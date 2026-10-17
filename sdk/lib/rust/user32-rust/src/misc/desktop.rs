use win32k_sys::win32k::*;
// The crate's own functions: the definition is the declaration.
use crate::windows::cursoricon::LoadCursorW;
use crate::internal::*;   // ntdll Rtl* + LoadUserApiHook/BeginIf*/End*/guah/gpsi + NT_SUCCESS not here
use crate::winbase::*;    // MultiByteToWideChar/WideCharToMultiByte + CP_THREAD_ACP + NT_SUCCESS
use crate::ntwrapper::*;  // GetWindowDC, SetCursor, SetShellWindowEx (spec forwarders)
#[allow(unused_imports)]
use core::ffi::c_void;

// bindgen opaqued `_SECURITY_ATTRIBUTES` (`_unused: [u8;0]`) → hand-declare the real
// winbase.h layout for the two field reads in CreateDesktop's lpsa handling.
#[repr(C)]
struct SaReal {
    nLength: DWORD,
    lpSecurityDescriptor: PVOID,
    bInheritHandle: BOOL,
}

// ---- internal C symbols (cohab) + ntdll/kernel32/gdi32 + sibling user32 exports ----
//
// EnumNamesA/EnumNamesW: `BOOL FASTCALL` in user32p.h (defined misc/winsta.c) — on
// i386 FASTCALL is `@Name@N`, so they must be `extern "fastcall"` (NOT "system").
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn EnumNamesA(WindowStation: HWINSTA, EnumFunc: DESKTOPENUMPROCA, Context: LPARAM, Desktops: BOOL) -> BOOL;
    fn EnumNamesW(WindowStation: HWINSTA, EnumFunc: DESKTOPENUMPROCW, Context: LPARAM, Desktops: BOOL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn EnumNamesA(WindowStation: HWINSTA, EnumFunc: DESKTOPENUMPROCA, Context: LPARAM, Desktops: BOOL) -> BOOL;
    fn EnumNamesW(WindowStation: HWINSTA, EnumFunc: DESKTOPENUMPROCW, Context: LPARAM, Desktops: BOOL) -> BOOL;
}

extern "system" {
    // misc/misc.c: `VOID WINAPI UserSetLastNTError(NTSTATUS)` — internal user32 C symbol.
    fn UserSetLastNTError(Status: NTSTATUS);
    // ntdll export (NTAPI): CSR client transport.
    fn CsrClientCallServer(
        ApiMessage: PCSR_API_MESSAGE,
        CaptureBuffer: PCSR_CAPTURE_BUFFER,
        ApiNumber: CSR_API_NUMBER,
        DataLength: ULONG,
    ) -> NTSTATUS;
    // kernel32 / gdi32 (resolved at link against the real DLLs).
    fn SetLastError(dwErrCode: u32);
    fn GdiConvertToDevmodeW(dm: LPDEVMODEA) -> LPDEVMODEW;
    // user32's OWN exports, defined in sibling modules of this crate (dc.rs / defwnd.rs /
    // controls). Called here via a local decl, resolved at link to those `#[no_mangle]`s.
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

// winuser.h `IDC_ARROW == MAKEINTRESOURCEW(32512)` — not in win32k-sys; reproduced
// locally (matching the static_.rs / menu.rs / combo.rs pattern).
const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR;

// `ERR(...)` — ReactOS debug print; a no-op in the cohab build (DUMMY debug).

// csrmsg.h: `CSR_CREATE_API_NUMBER(ServerId, ApiId) := (CSR_API_NUMBER)((ServerId<<16)|ApiId)`.
#[inline]
fn CSR_CREATE_API_NUMBER(ServerId: u32, ApiId: u32) -> CSR_API_NUMBER {
    ((ServerId << 16) | ApiId) as CSR_API_NUMBER
}

// user_x.h: `static __inline PDESKTOPINFO GetThreadDesktopInfo(VOID)` and the two
// TEB inlines it depends on (ntuser.h / user_x.h FORCEINLINE macros — not exports).
// Reproduced here exactly as in windows/defwnd.rs.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as *mut _ as PCLIENTINFO
}
use crate::windows::message::GetW32ThreadInfo;
#[inline]
unsafe fn GetThreadDesktopInfo() -> PDESKTOPINFO {
    let ti = GetW32ThreadInfo();
    if !ti.is_null() {
        (*GetWin32ClientInfo()).pDeskInfo
    } else {
        core::ptr::null_mut()
    }
}

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(user32) - debug channel macro, no Rust equivalent needed
//
// `DESKTOP_builtin_class` (desktop.c:19) is inside an `#if 0 // Kept for referencing`
// block — it is NOT compiled in C and is NOT referenced cross-TU (regcontrol.c's
// `controls[]` table omits it), so it is intentionally not emitted here.

#[no_mangle]
pub unsafe extern "system" fn DesktopWndProcW(
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match Msg {
        WM_ERASEBKGND | WM_NCCREATE | WM_CREATE | WM_CLOSE | WM_DISPLAYCHANGE | WM_PAINT
        | WM_SYSCOLORCHANGE => {
            let mut lResult: LRESULT = 0;
            NtUserMessageCall(
                Wnd,
                Msg,
                wParam,
                lParam,
                core::ptr::addr_of_mut!(lResult) as ULONG_PTR,
                FNID_DESKTOP,
                0u8 as BOOL,
            );
            lResult
        }

        WM_PALETTECHANGED => {
            if Wnd == wParam as HWND {
                return 0;
            }
            let hdc = GetWindowDC(Wnd);
            PaintDesktop(hdc);
            ReleaseDC(Wnd, hdc);
            0
        }

        WM_QUERYNEWPALETTE => {
            let hdc = GetWindowDC(Wnd);
            PaintDesktop(hdc);
            ReleaseDC(Wnd, hdc);
            0
        }

        WM_SETCURSOR => SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_ARROW as LPCWSTR)) as LRESULT,

        _ => DefWindowProcW(Wnd, Msg, wParam, lParam),
    }
}

unsafe extern "system" fn LogFontA2W(pW: LPLOGFONTW, pA: *const LOGFONTA) {
    (*pW).lfHeight = (*pA).lfHeight;
    (*pW).lfWidth = (*pA).lfWidth;
    (*pW).lfEscapement = (*pA).lfEscapement;
    (*pW).lfOrientation = (*pA).lfOrientation;
    (*pW).lfWeight = (*pA).lfWeight;
    (*pW).lfItalic = (*pA).lfItalic;
    (*pW).lfUnderline = (*pA).lfUnderline;
    (*pW).lfStrikeOut = (*pA).lfStrikeOut;
    (*pW).lfCharSet = (*pA).lfCharSet;
    (*pW).lfOutPrecision = (*pA).lfOutPrecision;
    (*pW).lfClipPrecision = (*pA).lfClipPrecision;
    (*pW).lfQuality = (*pA).lfQuality;
    (*pW).lfPitchAndFamily = (*pA).lfPitchAndFamily;
    MultiByteToWideChar(
        CP_THREAD_ACP,
        0,
        core::ptr::addr_of!((*pA).lfFaceName) as LPCSTR,
        LF_FACESIZE as i32,
        core::ptr::addr_of_mut!((*pW).lfFaceName) as LPWSTR,
        LF_FACESIZE as i32,
    );
}

unsafe extern "system" fn LogFontW2A(pA: LPLOGFONTA, pW: *const LOGFONTW) {
    (*pA).lfHeight = (*pW).lfHeight;
    (*pA).lfWidth = (*pW).lfWidth;
    (*pA).lfEscapement = (*pW).lfEscapement;
    (*pA).lfOrientation = (*pW).lfOrientation;
    (*pA).lfWeight = (*pW).lfWeight;
    (*pA).lfItalic = (*pW).lfItalic;
    (*pA).lfUnderline = (*pW).lfUnderline;
    (*pA).lfStrikeOut = (*pW).lfStrikeOut;
    (*pA).lfCharSet = (*pW).lfCharSet;
    (*pA).lfOutPrecision = (*pW).lfOutPrecision;
    (*pA).lfClipPrecision = (*pW).lfClipPrecision;
    (*pA).lfQuality = (*pW).lfQuality;
    (*pA).lfPitchAndFamily = (*pW).lfPitchAndFamily;
    WideCharToMultiByte(
        CP_THREAD_ACP,
        0,
        (*pW).lfFaceName.as_ptr(),
        LF_FACESIZE as i32,
        (*pA).lfFaceName.as_mut_ptr(),
        LF_FACESIZE as i32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn RealGetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int {
    if nIndex < 0 || nIndex >= SM_CMETRICS as core::ffi::c_int || gpsi.is_null() {
        return 0;
    }
    crate::wow64::psi::sys_met(nIndex as usize)
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int {
    let Hook: i32;
    let mut Ret: core::ffi::c_int = 0;

    if gpsi.is_null() {
        return RealGetSystemMetrics(nIndex);
    }

    LoadUserApiHook();

    Hook = BeginIfHookedUserApiHook();

    if Hook == 0 {
        return RealGetSystemMetrics(nIndex);
    }

    Ret = guah.GetSystemMetrics.unwrap()(nIndex);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SetDeskWallpaper(filename: LPCSTR) -> BOOL {
    SystemParametersInfoA(SPI_SETDESKWALLPAPER, 0, filename as PVOID, 1)
}

#[no_mangle]
pub unsafe extern "system" fn RealSystemParametersInfoA(
    uiAction: UINT,
    uiParam: UINT,
    mut pvParam: PVOID,
    fWinIni: UINT,
) -> BOOL {
    match uiAction {
        SPI_GETNONCLIENTMETRICS => {
            let pnclma = pvParam as LPNONCLIENTMETRICSA;
            let mut nclmw: NONCLIENTMETRICSW = core::mem::zeroed();
            if (*pnclma).cbSize != core::mem::size_of::<NONCLIENTMETRICSA>() as UINT {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            nclmw.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;

            if SystemParametersInfoW(
                uiAction,
                core::mem::size_of::<NONCLIENTMETRICSW>() as UINT,
                core::ptr::addr_of_mut!(nclmw) as PVOID,
                fWinIni,
            ) == 0
            {
                return 0;
            }

            (*pnclma).iBorderWidth = nclmw.iBorderWidth;
            (*pnclma).iScrollWidth = nclmw.iScrollWidth;
            (*pnclma).iScrollHeight = nclmw.iScrollHeight;
            (*pnclma).iCaptionWidth = nclmw.iCaptionWidth;
            (*pnclma).iCaptionHeight = nclmw.iCaptionHeight;
            (*pnclma).iSmCaptionWidth = nclmw.iSmCaptionWidth;
            (*pnclma).iSmCaptionHeight = nclmw.iSmCaptionHeight;
            (*pnclma).iMenuWidth = nclmw.iMenuWidth;
            (*pnclma).iMenuHeight = nclmw.iMenuHeight;
            LogFontW2A(
                core::ptr::addr_of_mut!((*pnclma).lfCaptionFont),
                core::ptr::addr_of!(nclmw.lfCaptionFont),
            );
            LogFontW2A(
                core::ptr::addr_of_mut!((*pnclma).lfSmCaptionFont),
                core::ptr::addr_of!(nclmw.lfSmCaptionFont),
            );
            LogFontW2A(
                core::ptr::addr_of_mut!((*pnclma).lfMenuFont),
                core::ptr::addr_of!(nclmw.lfMenuFont),
            );
            LogFontW2A(
                core::ptr::addr_of_mut!((*pnclma).lfStatusFont),
                core::ptr::addr_of!(nclmw.lfStatusFont),
            );
            LogFontW2A(
                core::ptr::addr_of_mut!((*pnclma).lfMessageFont),
                core::ptr::addr_of!(nclmw.lfMessageFont),
            );
            return 1;
        }
        SPI_SETNONCLIENTMETRICS => {
            let pnclma = pvParam as LPNONCLIENTMETRICSA;
            let mut nclmw: NONCLIENTMETRICSW = core::mem::zeroed();
            if (*pnclma).cbSize != core::mem::size_of::<NONCLIENTMETRICSA>() as UINT {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            nclmw.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
            nclmw.iBorderWidth = (*pnclma).iBorderWidth;
            nclmw.iScrollWidth = (*pnclma).iScrollWidth;
            nclmw.iScrollHeight = (*pnclma).iScrollHeight;
            nclmw.iCaptionWidth = (*pnclma).iCaptionWidth;
            nclmw.iCaptionHeight = (*pnclma).iCaptionHeight;
            nclmw.iSmCaptionWidth = (*pnclma).iSmCaptionWidth;
            nclmw.iSmCaptionHeight = (*pnclma).iSmCaptionHeight;
            nclmw.iMenuWidth = (*pnclma).iMenuWidth;
            nclmw.iMenuHeight = (*pnclma).iMenuHeight;
            LogFontA2W(
                core::ptr::addr_of_mut!(nclmw.lfCaptionFont),
                core::ptr::addr_of!((*pnclma).lfCaptionFont),
            );
            LogFontA2W(
                core::ptr::addr_of_mut!(nclmw.lfSmCaptionFont),
                core::ptr::addr_of!((*pnclma).lfSmCaptionFont),
            );
            LogFontA2W(
                core::ptr::addr_of_mut!(nclmw.lfMenuFont),
                core::ptr::addr_of!((*pnclma).lfMenuFont),
            );
            LogFontA2W(
                core::ptr::addr_of_mut!(nclmw.lfStatusFont),
                core::ptr::addr_of!((*pnclma).lfStatusFont),
            );
            LogFontA2W(
                core::ptr::addr_of_mut!(nclmw.lfMessageFont),
                core::ptr::addr_of!((*pnclma).lfMessageFont),
            );

            return SystemParametersInfoW(
                uiAction,
                core::mem::size_of::<NONCLIENTMETRICSW>() as UINT,
                core::ptr::addr_of_mut!(nclmw) as PVOID,
                fWinIni,
            );
        }
        SPI_GETICONMETRICS => {
            let picma = pvParam as LPICONMETRICSA;
            let mut icmw: ICONMETRICSW = core::mem::zeroed();
            if (*picma).cbSize != core::mem::size_of::<ICONMETRICSA>() as UINT {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            icmw.cbSize = core::mem::size_of::<ICONMETRICSW>() as UINT;
            if SystemParametersInfoW(
                uiAction,
                core::mem::size_of::<ICONMETRICSW>() as UINT,
                core::ptr::addr_of_mut!(icmw) as PVOID,
                fWinIni,
            ) == 0
            {
                return 0;
            }

            (*picma).iHorzSpacing = icmw.iHorzSpacing;
            (*picma).iVertSpacing = icmw.iVertSpacing;
            (*picma).iTitleWrap = icmw.iTitleWrap;
            LogFontW2A(
                core::ptr::addr_of_mut!((*picma).lfFont),
                core::ptr::addr_of!(icmw.lfFont),
            );
            return 1;
        }
        SPI_SETICONMETRICS => {
            let picma = pvParam as LPICONMETRICSA;
            let mut icmw: ICONMETRICSW = core::mem::zeroed();
            if (*picma).cbSize != core::mem::size_of::<ICONMETRICSA>() as UINT {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            icmw.cbSize = core::mem::size_of::<ICONMETRICSW>() as UINT;
            icmw.iHorzSpacing = (*picma).iHorzSpacing;
            icmw.iVertSpacing = (*picma).iVertSpacing;
            icmw.iTitleWrap = (*picma).iTitleWrap;
            LogFontA2W(
                core::ptr::addr_of_mut!(icmw.lfFont),
                core::ptr::addr_of!((*picma).lfFont),
            );

            return SystemParametersInfoW(
                uiAction,
                core::mem::size_of::<ICONMETRICSW>() as UINT,
                core::ptr::addr_of_mut!(icmw) as PVOID,
                fWinIni,
            );
        }
        SPI_GETICONTITLELOGFONT => {
            let mut lfw: LOGFONTW = core::mem::zeroed();
            if SystemParametersInfoW(
                uiAction,
                0,
                core::ptr::addr_of_mut!(lfw) as PVOID,
                fWinIni,
            ) == 0
            {
                return 0;
            }
            LogFontW2A(pvParam as LPLOGFONTA, core::ptr::addr_of!(lfw));
            return 1;
        }
        SPI_SETICONTITLELOGFONT => {
            let plfa = pvParam as LPLOGFONTA;
            let mut lfw: LOGFONTW = core::mem::zeroed();
            LogFontA2W(core::ptr::addr_of_mut!(lfw), plfa);
            return SystemParametersInfoW(
                uiAction,
                0,
                core::ptr::addr_of_mut!(lfw) as PVOID,
                fWinIni,
            );
        }
        SPI_GETDESKWALLPAPER => {
            let mut awc: [WCHAR; MAX_PATH as usize] = core::mem::zeroed();
            let mut ustrWallpaper: _UNICODE_STRING = core::mem::zeroed();
            let mut astrWallpaper: ANSI_STRING = core::mem::zeroed();

            let Ret = NtUserSystemParametersInfo(
                SPI_GETDESKWALLPAPER,
                MAX_PATH,
                awc.as_mut_ptr() as PVOID,
                fWinIni,
            );
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(ustrWallpaper),
                awc.as_ptr(),
            );
            RtlInitEmptyAnsiString(
                core::ptr::addr_of_mut!(astrWallpaper),
                pvParam as PCHAR,
                uiParam as u16,
            );
            RtlUnicodeStringToAnsiString(
                core::ptr::addr_of_mut!(astrWallpaper),
                core::ptr::addr_of!(ustrWallpaper),
                0,
            );
            return Ret;
        }
        SPI_SETDESKWALLPAPER => {
            let mut ustrWallpaper: _UNICODE_STRING = core::mem::zeroed();

            if !pvParam.is_null() {
                if RtlCreateUnicodeStringFromAsciiz(
                    core::ptr::addr_of_mut!(ustrWallpaper),
                    pvParam as PCSZ,
                ) == 0
                {
                    ERR!("RtlCreateUnicodeStringFromAsciiz failed\n");
                    return 0;
                }
                pvParam = core::ptr::addr_of_mut!(ustrWallpaper) as PVOID;
            }

            let Ret = NtUserSystemParametersInfo(SPI_SETDESKWALLPAPER, uiParam, pvParam, fWinIni);

            if !pvParam.is_null() {
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(ustrWallpaper));
            }

            return Ret;
        }
        _ => {}
    }
    NtUserSystemParametersInfo(uiAction, uiParam, pvParam, fWinIni)
}

#[no_mangle]
pub unsafe extern "system" fn RealSystemParametersInfoW(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    fWinIni: UINT,
) -> BOOL {
    match uiAction {
        SPI_SETDESKWALLPAPER => {
            let mut ustrWallpaper: _UNICODE_STRING = core::mem::zeroed();
            RtlInitUnicodeString(&mut ustrWallpaper, pvParam as PCWSTR);
            return NtUserSystemParametersInfo(
                SPI_SETDESKWALLPAPER,
                uiParam,
                core::ptr::addr_of_mut!(ustrWallpaper) as PVOID,
                fWinIni,
            );
        }
        _ => {}
    }
    NtUserSystemParametersInfo(uiAction, uiParam, pvParam, fWinIni)
}

#[no_mangle]
pub unsafe extern "system" fn SystemParametersInfoA(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    fWinIni: UINT,
) -> BOOL {
    let Hook: BOOL;
    let mut Ret: BOOL = 0;

    LoadUserApiHook();

    Hook = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        return RealSystemParametersInfoA(uiAction, uiParam, pvParam, fWinIni);
    }

    Ret = guah.SystemParametersInfoA.unwrap()(uiAction, uiParam, pvParam, fWinIni);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SystemParametersInfoW(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    fWinIni: UINT,
) -> BOOL {
    let mut Ret: BOOL = 0;

    LoadUserApiHook();

    let Hook: BOOL = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        return RealSystemParametersInfoW(uiAction, uiParam, pvParam, fWinIni);
    }

    Ret = guah.SystemParametersInfoW.unwrap()(uiAction, uiParam, pvParam, fWinIni);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn CreateDesktopA(
    lpszDesktop: LPCSTR,
    lpszDevice: LPCSTR,
    pDevmode: LPDEVMODEA,
    dwFlags: u32,
    dwDesiredAccess: ACCESS_MASK,
    lpsa: LPSECURITY_ATTRIBUTES,
) -> HDESK {
    let mut DesktopNameU: _UNICODE_STRING = core::mem::zeroed();
    let hDesktop: HDESK;
    let mut DevmodeW: LPDEVMODEW = core::ptr::null_mut();

    if !lpszDesktop.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut DesktopNameU, lpszDesktop as PCSZ);
    } else {
        RtlInitUnicodeString(&mut DesktopNameU, core::ptr::null_mut());
    }

    if !pDevmode.is_null() {
        DevmodeW = GdiConvertToDevmodeW(pDevmode);
    }

    hDesktop = CreateDesktopW(
        DesktopNameU.Buffer,
        core::ptr::null_mut(),
        DevmodeW,
        dwFlags,
        dwDesiredAccess,
        lpsa,
    );

    if !lpszDesktop.is_null() {
        RtlFreeUnicodeString(&mut DesktopNameU);
    }

    hDesktop
}

#[no_mangle]
pub unsafe extern "system" fn CreateDesktopW(
    lpszDesktop: LPCWSTR,
    lpszDevice: LPCWSTR,
    pDevmode: LPDEVMODEW,
    dwFlags: u32,
    dwDesiredAccess: ACCESS_MASK,
    lpsa: LPSECURITY_ATTRIBUTES,
) -> HDESK {
    let mut oas: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DesktopName: _UNICODE_STRING = core::mem::zeroed();
    let mut DesktopDevice: _UNICODE_STRING = core::mem::zeroed();
    let hWinSta: HWINSTA;
    let hDesktop: HDESK;
    let mut Attributes: u32 = OBJ_OPENIF | OBJ_CASE_INSENSITIVE;

    hWinSta = NtUserGetProcessWindowStation();

    RtlInitUnicodeString(&mut DesktopName, lpszDesktop);
    RtlInitUnicodeString(&mut DesktopDevice, lpszDevice);

    if !lpsa.is_null() && (*(lpsa as *const SaReal)).bInheritHandle != 0 {
        Attributes |= OBJ_INHERIT;
    }

    oas.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    oas.RootDirectory = hWinSta as HANDLE;
    oas.Attributes = Attributes;
    oas.ObjectName = &mut DesktopName;
    oas.SecurityDescriptor = if !lpsa.is_null() {
        (*(lpsa as *const SaReal)).lpSecurityDescriptor
    } else {
        core::ptr::null_mut()
    };
    oas.SecurityQualityOfService = core::ptr::null_mut();

    hDesktop = NtUserCreateDesktop(
        &mut oas,
        &mut DesktopDevice,
        pDevmode,
        dwFlags,
        dwDesiredAccess,
    );

    hDesktop
}

#[no_mangle]
pub unsafe extern "system" fn EnumDesktopsA(
    WindowStation: HWINSTA,
    EnumFunc: DESKTOPENUMPROCA,
    Context: LPARAM,
) -> i32 {
    EnumNamesA(WindowStation, EnumFunc, Context, 1)
}

#[no_mangle]
pub unsafe extern "system" fn EnumDesktopsW(
    WindowStation: HWINSTA,
    EnumFunc: DESKTOPENUMPROCW,
    Context: LPARAM,
) -> i32 {
    EnumNamesW(WindowStation, EnumFunc, Context, 1)
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadDesktop(dwThreadId: DWORD) -> HDESK {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let GetThreadConsoleDesktopRequest: PUSER_GET_THREAD_CONSOLE_DESKTOP =
        core::ptr::addr_of_mut!(ApiMessage.Data.GetThreadConsoleDesktopRequest);

    (*GetThreadConsoleDesktopRequest).ThreadId = dwThreadId as ULONG_PTR;

    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpGetThreadConsoleDesktop),
        core::mem::size_of_val(&*GetThreadConsoleDesktopRequest) as u32,
    );

    if NT_SUCCESS(ApiMessage.Status) == false {
        UserSetLastNTError(ApiMessage.Status);
        return core::ptr::null_mut();
    }

    NtUserGetThreadDesktop(dwThreadId, (*GetThreadConsoleDesktopRequest).ConsoleDesktop)
}

#[no_mangle]
pub unsafe extern "system" fn OpenDesktopA(
    lpszDesktop: LPCSTR,
    dwFlags: u32,
    fInherit: i32,
    dwDesiredAccess: ACCESS_MASK,
) -> HDESK {
    let mut DesktopNameU: _UNICODE_STRING = core::mem::zeroed();

    if !lpszDesktop.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut DesktopNameU, lpszDesktop as PCSZ);
    } else {
        RtlInitUnicodeString(&mut DesktopNameU, core::ptr::null_mut());
    }

    let hDesktop = OpenDesktopW(
        DesktopNameU.Buffer,
        dwFlags,
        fInherit,
        dwDesiredAccess,
    );

    if !lpszDesktop.is_null() {
        RtlFreeUnicodeString(&mut DesktopNameU);
    }

    hDesktop
}

#[no_mangle]
pub unsafe extern "system" fn OpenDesktopW(
    lpszDesktop: LPCWSTR,
    dwFlags: u32,
    fInherit: i32,
    dwDesiredAccess: ACCESS_MASK,
) -> HDESK {
    let mut DesktopName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    RtlInitUnicodeString(&mut DesktopName, lpszDesktop);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = GetProcessWindowStation() as HANDLE;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut DesktopName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if fInherit != 0 {
        ObjectAttributes.Attributes |= OBJ_INHERIT;
    }

    NtUserOpenDesktop(&mut ObjectAttributes, dwFlags, dwDesiredAccess)
}

#[no_mangle]
pub unsafe extern "system" fn SetShellWindow(hwndShell: HWND) -> i32 {
    SetShellWindowEx(hwndShell, hwndShell)
}

#[no_mangle]
pub unsafe extern "system" fn GetShellWindow() -> HWND {
    let pdi: PDESKTOPINFO = GetThreadDesktopInfo();
    if !pdi.is_null() {
        return crate::wow64::desktop_info_shell_window(pdi);
    }
    core::ptr::null_mut()
}
