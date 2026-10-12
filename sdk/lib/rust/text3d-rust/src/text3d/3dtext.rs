use crate::text3d::settings::LoadSettings;
use crate::text3d::settings::SaveSettings;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hRC: HGLRC = ::core::ptr::null::<HGLRC__>() as *mut HGLRC__;

static mut hDC: HDC = ::core::ptr::null::<HDC__>() as *mut HDC__;

static mut base: GLuint = 0;

static mut text_display_list: [GLuint; 20] = [0; 20];

static mut rot: GLfloat = 0.;

static mut extentX: GLfloat = 0.0f32;

static mut extentY: GLfloat = 0.0f32;

#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut fullscreen: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut uTimerID: UINT = 0;

// APPNAME (&[0x0033, 0x0044, 0x0054, 0x0065, 0x0078, 0x0074, 0x0000u16]).as_ptr() /* _T "3DText" */ -- wide string with terminating NUL
pub const APPNAME: [u16; 7] = [
    0x0033, // '3'
    0x0044, // 'D'
    0x0054, // 'T'
    0x0065, // 'e'
    0x0078, // 'x'
    0x0074, // 't'
    0x0000, // NUL
];

pub const APP_TIMER: UINT = 1; // Graphics Update Timer ID

pub const APP_TIMER_INTERVAL: UINT = USER_TIMER_MINIMUM * 5; // Graphics Update Interval

#[no_mangle]
pub unsafe extern "C" fn Build3DCharacters() {
    // Address Buffer For Character Storage
    let mut gmf: [GLYPHMETRICSFLOAT; (MAX_TEXT_LENGTH as _)] = core::mem::zeroed();
    // Windows Font Handle
    let mut font: HFONT;
    let mut i: size_t;
    let mut cellOriginX: GLfloat = 0.0f32;
    let mut stringOriginX: GLfloat;
    let mut stringExtentX: GLfloat = 0.0f32;
    let mut stringExtentY: GLfloat = 0.0f32;

    // Storage for MAX_TEXT_LENGTH number of characters
    base = glGenLists((MAX_TEXT_LENGTH as i32));

    font = CreateFont(-12,
                      0,                            // Width Of Font
                      0,                            // Angle Of Escapement
                      0,                            // Orientation Angle
                      (FW_BOLD as i32),                      // Font Weight
                      FALSE,                        // Italic
                      FALSE,                        // Underline
                      FALSE,                        // Strikeout
                      DEFAULT_CHARSET,              // Character Set Identifier
                      OUT_TT_PRECIS,                // Output Precision
                      CLIP_DEFAULT_PRECIS,          // Clipping Precision
                      ANTIALIASED_QUALITY,          // Output Quality
                      FF_DONTCARE|DEFAULT_PITCH,    // Family And Pitch
                      (&['T' as u16, 'a' as u16, 'h' as u16, 'o' as u16, 'm' as u16, 'a' as u16, 0u16]).as_ptr());                // Font Name

    // Selects The Font We Created
    SelectObject(hDC, (font).cast());

    let text_length: size_t = _tcslen(core::ptr::addr_of_mut!(g_Text) as *mut TCHAR) as size_t;

    // Calculate the string extent
    i = 0;
    while i < text_length {
        text_display_list[i] = (base as size_t + i) as GLuint;

        wglUseFontOutlines(hDC,                     // Select The Current DC
                           g_Text[i] as DWORD,      // Starting Character
                           1,                       // Number Of Display Lists To Build
                           (base as size_t + i) as GLuint,                // Starting Display Lists
                           0.0f32,                    // Deviation From The True Outlines
                           0.2f32,                    // Font Thickness In The Z Direction
                           WGL_FONT_POLYGONS as ::core::ffi::c_int, // Use Polygons, Not Lines
                           (gmf.as_mut_ptr()).add(i));                // Address Of Buffer To Receive Data

        stringOriginX = cellOriginX + gmf[i].gmfptGlyphOrigin.x;

        stringExtentX = stringOriginX + gmf[i].gmfBlackBoxX;
        if gmf[i].gmfBlackBoxY > stringExtentY {
            stringExtentY = gmf[i].gmfBlackBoxY;
        }

        cellOriginX = cellOriginX + gmf[i].gmfCellIncX;
        i += 1;
    }

    extentX = stringExtentX;
    extentY = stringExtentY;
}
#[no_mangle]
pub unsafe extern "C" fn Delete3DCharacters() {
    // Delete all MAX_TEXT_LENGTH characters
    glDeleteLists(base, (MAX_TEXT_LENGTH as i32));
}
#[no_mangle]
pub unsafe extern "C" fn glPrint() {
    // If there's no text, do nothing
    if g_Text[0 as usize] == 0 {
        return;
    }

    let mut text_length: size_t = _tcslen(core::ptr::addr_of_mut!(g_Text) as *mut TCHAR) as size_t;

    // Draws The Display List Text
    glCallLists(
        text_length as GLsizei,
        GL_UNSIGNED_INT as GLenum,
        core::ptr::addr_of_mut!(text_display_list) as *mut GLuint as *const core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn InitGL(Width: GLsizei, Height: GLsizei) {
    // Clear The Background Color To Black
    glClearColor(0.0f32, 0.0f32, 0.0f32, 0.0f32);

    // Enables Clearing Of The Depth Buffer
    glClearDepth(1.0f64);

    // The Type Of Depth Test To Do
    glDepthFunc(GL_LESS);

    // Enables Depth Testing
    glEnable(GL_DEPTH_TEST);

    // Enables Smooth Color Shading
    glShadeModel(GL_SMOOTH);

    // Select The Projection Matrix
    glMatrixMode(GL_PROJECTION);

    // Reset The Projection Matrix
    glLoadIdentity();

    // Calculate The Aspect Ratio Of The Window
    gluPerspective(45.0f64, (Width as GLfloat / Height as GLfloat) as GLdouble, 0.1f64, 100.0f64);

    // Select The Modelview Matrix
    glMatrixMode(GL_MODELVIEW);

    // Build The 3D Characters
    Build3DCharacters();

    // Enable Default Light (Quick And Dirty)
    glEnable(GL_LIGHT0);

    // Enable Lighting
    glEnable(GL_LIGHTING);

    // Enable Coloring Of Material
    glEnable(GL_COLOR_MATERIAL);
}
#[no_mangle]
pub unsafe extern "C" fn ReSizeGLScene(mut Width: GLsizei, mut Height: GLsizei) {
    // Is Window Too Small (Divide By Zero Error)
    if Height == 0 {
        // If So Make It One Pixel Tall
        Height = 1;
    }

    // Reset The Current Viewport And Perspective Transformation
    glViewport(0, 0, Width, Height);

    // Select The Projection Matrix
    glMatrixMode(GL_PROJECTION);

    // Reset The Projection Matrix
    glLoadIdentity();

    // Calculate The Aspect Ratio Of The Window
    // C promotes these floats to gluPerspective's GLdouble parameters.
    gluPerspective(45.0, ((Width as GLfloat) / (Height as GLfloat)) as GLdouble, 0.1, 100.0);

    // Select The Modelview Matrix
    glMatrixMode(GL_MODELVIEW);
}
#[no_mangle]
pub unsafe extern "C" fn DrawGLScene() {
    // Save ticks count of previous frame here
    static mut dwTicks: DWORD = 0;

    // Clear The Screen And The Depth Buffer
    glClear((GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT) as GLbitfield);

    // Reset The View
    glLoadIdentity();

    // Move One Unit Into The Screen
    glTranslatef(0.0f32, 0.0f32, -10.0f32);

    // Rotate On The X Axis
    glRotatef(rot, 1.0f32, 0.0f32, 0.0f32);

    // Rotate On The Y Axis
    glRotatef(rot * 1.2f32, 0.0f32, 1.0f32, 0.0f32);

    // Rotate On The Z Axis
    glRotatef(rot * 1.4f32, 0.0f32, 0.0f32, 1.0f32);

    // Move to the Left and Down before drawing
    glTranslatef(-(extentX / 2.0f32),
                 -(extentY / 2.0f32),
                 0.0f32);

    // Pulsing Colors Based On The Rotation
    glColor3f(
        (1.0f32 * (cos((rot / 20.0f32) as f64) as GLfloat)),
        (1.0f32 * (sin((rot / 25.0f32) as f64) as GLfloat)),
        (1.0f32 - 0.5f32 * (cos((rot / 17.0f32) as f64) as GLfloat)));

    // Print GL Text To The Screen
    glPrint();

    // Make The Text Blue
    glColor3f(0.0f32, 0.0f32, 1.0f32);

    // Increase The Rotation Variable
    if dwTicks != 0 {
        rot += (GetTickCount().wrapping_sub(dwTicks)) as GLfloat / 20.0f32;
    }
    dwTicks = GetTickCount();
}

#[no_mangle]
pub unsafe extern "system" fn ScreenSaverProc(
    hWnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    static ERR_NO_PIXELFORMAT: [u16; 35] = [
        'C' as u16, 'a' as u16, 'n' as u16, '\'' as u16, 't' as u16, ' ' as u16,
        'F' as u16, 'i' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'A' as u16,
        ' ' as u16, 'S' as u16, 'u' as u16, 'i' as u16, 't' as u16, 'a' as u16,
        'b' as u16, 'l' as u16, 'e' as u16, ' ' as u16, 'P' as u16, 'i' as u16,
        'x' as u16, 'e' as u16, 'l' as u16, 'F' as u16, 'o' as u16, 'r' as u16,
        'm' as u16, 'a' as u16, 't' as u16, '.' as u16, 0,
    ];
    static ERR_TITLE: [u16; 6] = [
        'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, 0,
    ];
    static ERR_NO_SET: [u16; 27] = [
        'C' as u16, 'a' as u16, 'n' as u16, '\'' as u16, 't' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 't' as u16, ' ' as u16, 'T' as u16, 'h' as u16,
        'e' as u16, ' ' as u16, 'P' as u16, 'i' as u16, 'x' as u16, 'e' as u16,
        'l' as u16, 'F' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16,
        't' as u16, '.' as u16, 0,
    ];
    static ERR_NO_RC: [u16; 37] = [
        'C' as u16, 'a' as u16, 'n' as u16, '\'' as u16, 't' as u16, ' ' as u16,
        'C' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16,
        ' ' as u16, 'A' as u16, ' ' as u16, 'G' as u16, 'L' as u16, ' ' as u16,
        'R' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'r' as u16,
        'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, 'C' as u16, 'o' as u16,
        'n' as u16, 't' as u16, 'e' as u16, 'x' as u16, 't' as u16, '.' as u16,
        0,
    ];
    static ERR_NO_ACTIVATE: [u16; 21] = [
        'C' as u16, 'a' as u16, 'n' as u16, '\'' as u16, 't' as u16, ' ' as u16,
        'A' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'a' as u16,
        't' as u16, 'e' as u16, ' ' as u16, 'G' as u16, 'L' as u16, 'R' as u16,
        'C' as u16, '.' as u16, 0,
    ];

    let mut Screen: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut PixelFormat: GLuint = 0;
    static mut pfd: PIXELFORMATDESCRIPTOR = PIXELFORMATDESCRIPTOR {
        nSize: core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD,
        nVersion: 1 as WORD,
        dwFlags: (PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER) as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 0 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 0 as BYTE,
        cGreenShift: 0 as BYTE,
        cBlueBits: 0 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
        cStencilBits: 0 as BYTE,
        cAuxBuffers: 0 as BYTE,
        iLayerType: PFD_MAIN_PLANE as BYTE,
        bReserved: 0 as BYTE,
        dwLayerMask: 0 as DWORD,
        dwVisibleMask: 0 as DWORD,
        dwDamageMask: 0 as DWORD,
    };

    'sw: {
        match message {
            WM_CREATE => {
                LoadSettings();

                // Gets A Device Context For The Window
                hDC = GetDC(hWnd);

                // Finds The Closest Match To The Pixel Format We Set Above
                PixelFormat = ChoosePixelFormat(hDC, core::ptr::addr_of_mut!(pfd)) as GLuint;

                // No Matching Pixel Format?
                if PixelFormat == 0 {
                    MessageBox(
                        core::ptr::null_mut(),
                        ERR_NO_PIXELFORMAT.as_ptr(),
                        ERR_TITLE.as_ptr(),
                        (MB_OK | MB_ICONERROR) as UINT,
                    );

                    // This Sends A 'Message' Telling The Program To Quit
                    PostQuitMessage(0);
                    break 'sw;
                }

                // Can We Set The Pixel Mode?
                if SetPixelFormat(hDC, PixelFormat as core::ffi::c_int, core::ptr::addr_of_mut!(pfd)) == 0 {
                    MessageBox(
                        core::ptr::null_mut(),
                        ERR_NO_SET.as_ptr(),
                        ERR_TITLE.as_ptr(),
                        (MB_OK | MB_ICONERROR) as UINT,
                    );

                    // This Sends A 'Message' Telling The Program To Quit
                    PostQuitMessage(0);
                    break 'sw;
                }

                // Grab A Rendering Context
                hRC = wglCreateContext(hDC);

                // Did We Get One?
                if hRC.is_null() {
                    MessageBox(
                        core::ptr::null_mut(),
                        ERR_NO_RC.as_ptr(),
                        ERR_TITLE.as_ptr(),
                        (MB_OK | MB_ICONERROR) as UINT,
                    );

                    // This Sends A 'Message' Telling The Program To Quit
                    PostQuitMessage(0);
                    break 'sw;
                }

                // Can We Make The RC Active?
                if wglMakeCurrent(hDC, hRC) == 0 {
                    MessageBox(
                        core::ptr::null_mut(),
                        ERR_NO_ACTIVATE.as_ptr(),
                        ERR_TITLE.as_ptr(),
                        (MB_OK | MB_ICONERROR) as UINT,
                    );

                    // This Sends A 'Message' Telling The Program To Quit
                    PostQuitMessage(0);
                    break 'sw;
                }

                // Grab Screen Info For The Current Window
                GetClientRect(hWnd, core::ptr::addr_of_mut!(Screen));

                // Initialize The GL Screen Using Screen Info
                InitGL(Screen.right as GLsizei, Screen.bottom as GLsizei);

                // Create Graphics update timer
                uTimerID = SetTimer(
                    hWnd,
                    (APP_TIMER) as UINT_PTR,
                    APP_TIMER_INTERVAL,
                    None,
                ) as UINT;
                break 'sw;
            }

            WM_DESTROY => {
                // Disable Fullscreen Mode
                ChangeDisplaySettings(core::ptr::null_mut(), 0);

                // Delete the Update Timer
                KillTimer(hWnd, uTimerID as UINT_PTR);

                // Deletes The Character Display Lists
                Delete3DCharacters();

                // Make The DC Current
                wglMakeCurrent(hDC, core::ptr::null_mut());

                // Kill The RC
                wglDeleteContext(hRC);

                // Free The DC
                ReleaseDC(hWnd, hDC);
                break 'sw;
            }

            WM_PAINT => {
                DrawGLScene();
                SwapBuffers(hDC);
                // Mark this window as updated, so the OS won't ask us to update it again.
                ValidateRect(hWnd, core::ptr::null());
                break 'sw;
            }

            WM_SIZE => {
                // Resizing The Screen
                // Resize To The New Window Size
                ReSizeGLScene(
                    LOWORD!(lParam as DWORD) as GLsizei,
                    HIWORD!(lParam as DWORD) as GLsizei,
                );
                break 'sw;
            }

            WM_TIMER => {
                // Used to update graphic based on timer udpate interval
                InvalidateRect(hWnd, core::ptr::null(), (TRUE) as BOOL);
                break 'sw;
            }

            _ => {
                // Pass Windows Messages to the default screensaver window procedure
                return DefScreenSaverProc(hWnd, message, wParam, lParam);
            }
        }
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ScreenSaverConfigureDialog(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    match uMsg {
        WM_INITDIALOG => {
            LoadSettings();
            SetDlgItemText(hDlg, (IDC_MESSAGE_TEXT as i32), core::ptr::addr_of_mut!(g_Text) as LPWSTR);
            SendDlgItemMessage(hDlg, (IDC_MESSAGE_TEXT as i32), EM_LIMITTEXT, (ARRAYSIZE!(g_Text) - 1) as _, 0);
            return (TRUE) as BOOL;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    GetDlgItemText(hDlg, (IDC_MESSAGE_TEXT as i32), core::ptr::addr_of_mut!(g_Text) as LPWSTR, (ARRAYSIZE!(g_Text)) as i32);
                    SaveSettings();
                    EndDialog(hDlg, (IDCANCEL) as INT_PTR);
                }
                IDCANCEL => {
                    EndDialog(hDlg, (IDCANCEL) as INT_PTR);
                }
                _ => {}
            }
            return (FALSE) as BOOL;
        }
        WM_CLOSE => {
            EndDialog(hDlg, 0);
        }
        _ => {
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterDialogClasses(hInst: HANDLE) -> BOOL {
    (TRUE as i32)
}
