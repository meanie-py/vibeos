use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
#[allow(unused_imports)]
use core::ffi::c_void;

// ── No-op debug macros (Wine debug-channel guard compiles them to nothing).

// ── NT_SUCCESS lives in crate::winbase; NtCurrentTeb in crate::internal.
use crate::internal::NtCurrentTeb;
use crate::winbase::NT_SUCCESS;
// These are inline ntwrapper wrappers (NtUserCallOneParam multiplexers), not external
// link symbols — import them instead of declaring them extern.
use crate::ntwrapper::{NtUserxRealizePalette, NtUserxCreateSystemThreads};

// winnt.h: STATUS_NO_MEMORY.
const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as NTSTATUS;

// csrmsg.h: CSR_CREATE_API_NUMBER(ServerId, ApiId) := (ServerId << 16) | ApiId.
#[inline]
fn CSR_CREATE_API_NUMBER(ServerId: ULONG, ApiId: ULONG) -> CSR_API_NUMBER {
    ((ServerId << 16) | ApiId) as CSR_API_NUMBER
}

// ntuser.h: #define GetWin32ClientInfo() ((PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo))
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO
}

extern "C" {
    fn memcpy( dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize, ) -> *mut core::ffi::c_void;
}
extern "system" {
    fn CreatePen(iStyle: i32, cWidth: i32, color: COLORREF) -> HPEN;
    fn DeleteObject(ho: LPVOID) -> i32;
    fn GetProcessHeap() -> LPVOID;
    fn HeapFree(hHeap: LPVOID, dwFlags: u32, lpMem: LPVOID) -> i32;
    pub fn ExitThread(code: DWORD) -> !; // DECLSPEC_NORETURN
    fn SetLastError(dwErrCode: DWORD);
    // ── ntdll CSR client (csr.h). WINAPI/NTAPI = stdcall on i386 → extern "system".
    fn CsrClientCallServer(
        ApiMessage: PCSR_API_MESSAGE,
        CaptureBuffer: PCSR_CAPTURE_BUFFER,
        ApiNumber: CSR_API_NUMBER,
        DataLength: ULONG,
    ) -> NTSTATUS;
    fn CsrAllocateCaptureBuffer(ArgumentCount: ULONG, BufferSize: ULONG) -> PCSR_CAPTURE_BUFFER;
    fn CsrCaptureMessageBuffer(
        CaptureBuffer: PCSR_CAPTURE_BUFFER,
        MessageBuffer: PVOID,
        MessageLength: ULONG,
        CapturedData: *mut PVOID,
    ) -> PVOID;
    fn CsrFreeCaptureBuffer(CaptureBuffer: PCSR_CAPTURE_BUFFER) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub static mut SysColorPens: [HPEN; (COLOR_MENUBAR + 1) as usize] = [core::ptr::null_mut(); (COLOR_MENUBAR + 1) as usize];
pub static mut SysColorBrushes: [HBRUSH; (COLOR_MENUBAR + 1) as usize] = [core::ptr::null_mut(); (COLOR_MENUBAR + 1) as usize];

#[no_mangle]
pub unsafe extern "system" fn WaitForInputIdle(
    hProcess: HANDLE,
    dwMilliseconds: DWORD,
) -> DWORD {
    NtUserWaitForInputIdle(hProcess, dwMilliseconds, 0u8 as _)
}

#[no_mangle]
pub unsafe extern "system" fn SetDebugErrorLevel(dwLevel: DWORD) {
    FIXME!("(%lu): stub\n", dwLevel);
}

#[no_mangle]
pub unsafe extern "system" fn GetAppCompatFlags(hTask: HTASK) -> DWORD {
    let pci: PCLIENTINFO = GetWin32ClientInfo();

    (*pci).dwCompatFlags
}

#[no_mangle]
pub unsafe extern "system" fn GetAppCompatFlags2(hTask: HTASK) -> u32 {
    let pci: PCLIENTINFO = GetWin32ClientInfo();

    (*pci).dwCompatFlags2
}

#[no_mangle]
pub unsafe extern "system" fn LoadLocalFonts() {
}

#[no_mangle]
pub unsafe extern "system" fn LoadRemoteFonts() {
}

#[no_mangle]
pub unsafe extern "system" fn RegisterSystemThread(flags: DWORD, reserved: DWORD) {
}
#[no_mangle]
pub unsafe extern "system" fn UserRealizePalette(hDC: HDC) -> UINT {
    NtUserxRealizePalette(hDC)
}

#[no_mangle]
pub unsafe extern "system" fn SetSysColorsTemp(
    pPens: *const COLORREF,
    pBrushes: *const HBRUSH,
    n: DWORD_PTR,
) -> DWORD_PTR {
    let mut i: u32;

    if !pPens.is_null() && !pBrushes.is_null() {
        let pOldCol: LPVOID = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<u32>()
                + (n as usize) * core::mem::size_of::<HPEN>()
                + (n as usize) * core::mem::size_of::<HBRUSH>()) as SIZE_T,
        );
        let mut p: LPVOID = pOldCol;
        *(p as *mut DWORD_PTR) = n;
        p = (p as *mut u8).add(core::mem::size_of::<u32>()) as LPVOID;
        memcpy(
            p,
            (*(&raw const SysColorPens)).as_ptr() as *const core::ffi::c_void,
            (n as usize) * core::mem::size_of::<HPEN>(),
        );
        p = (p as *mut u8).add((n as usize) * core::mem::size_of::<HPEN>()) as LPVOID;
        memcpy(
            p,
            (*(&raw const SysColorBrushes)).as_ptr() as *const core::ffi::c_void,
            (n as usize) * core::mem::size_of::<HBRUSH>(),
        );
        p = (p as *mut u8).add((n as usize) * core::mem::size_of::<HBRUSH>()) as LPVOID;
        let _ = p;

        i = 0;
        while (i as DWORD_PTR) < n {
            SysColorPens[i as usize] = CreatePen(PS_SOLID as i32, 1, *pPens.add(i as usize));
            SysColorBrushes[i as usize] = *pBrushes.add(i as usize);
            i += 1;
        }

        return pOldCol as DWORD_PTR;
    }
    if pPens.is_null() && pBrushes.is_null() {
        let pOldCol: LPVOID = n as LPVOID;
        let mut p: LPVOID = pOldCol;
        let nCount: u32 = *(p as *mut u32);
        p = (p as *mut u8).add(core::mem::size_of::<u32>()) as LPVOID;

        i = 0;
        while i < nCount {
            DeleteObject(SysColorPens[i as usize] as LPVOID);
            SysColorPens[i as usize] = *(p as *mut HPEN);
            p = (p as *mut u8).add(core::mem::size_of::<HPEN>()) as LPVOID;
            i += 1;
        }
        i = 0;
        while i < nCount {
            SysColorBrushes[i as usize] = *(p as *mut HBRUSH);
            p = (p as *mut u8).add(core::mem::size_of::<HBRUSH>()) as LPVOID;
            i += 1;
        }
        HeapFree(GetProcessHeap(), 0, pOldCol);

        return 1 as DWORD_PTR;
    }
    0 as DWORD_PTR
}

#[no_mangle]
pub unsafe extern "system" fn GetInputDesktop() -> HDESK {
    0 as HDESK
}

#[no_mangle]
pub unsafe extern "system" fn GetAccCursorInfo(pci: PCURSORINFO) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetRawInputDeviceInfoW(
    hDevice: HANDLE,
    uiCommand: UINT,
    pData: LPVOID,
    pcbSize: PUINT,
) -> UINT {
    0
}

#[no_mangle]
pub unsafe extern "system" fn CsrBroadcastSystemMessageExW(
    dwflags: DWORD,
    lpdwRecipients: LPDWORD,
    uiMessage: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    pBSMInfo: PBSMINFO,
) -> LONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetRawInputDeviceInfoA(
    hDevice: HANDLE,
    uiCommand: UINT,
    pData: LPVOID,
    pcbSize: PUINT,
) -> UINT {
    let mut pDataW = pData;
    let mut cbSize: UINT = *pcbSize;

    if uiCommand == RIDI_DEVICENAME as UINT && !pData.is_null() {
        pDataW = HeapAlloc(GetProcessHeap(), 0, (cbSize as usize * core::mem::size_of::<WCHAR>()) as SIZE_T);
        if pDataW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return UINT::MAX;
        }
    }

    let mut Ret = NtUserGetRawInputDeviceInfo(hDevice, uiCommand, pDataW, &mut cbSize) as UINT;
    // Divergence from stubs.c: its `Ret >= 0` is always true on a UINT, so a failed call
    // (UINT)-1 still "converted" the uninitialised heap block into the caller's buffer.
    // Convert only what the syscall actually filled.
    if Ret != UINT::MAX && uiCommand == RIDI_DEVICENAME as UINT && !pDataW.is_null() {
        if crate::winbase::WideCharToMultiByte(
            crate::winbase::CP_THREAD_ACP,
            0,
            pDataW as LPCWSTR,
            cbSize as i32,
            pData as LPSTR,
            *pcbSize as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) == 0
        {
            Ret = UINT::MAX;
        }
    }
    if pData != pDataW {
        HeapFree(GetProcessHeap(), 0, pDataW);
    }
    *pcbSize = cbSize;
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AlignRects(rect: LPRECT, b: DWORD, c: DWORD, d: DWORD) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DefRawInputProc(
    paRawInput: *mut PRAWINPUT,
    nInput: INT,
    cbSizeHeader: UINT,
) -> LRESULT {
    if cbSizeHeader as usize == core::mem::size_of::<RAWINPUTHEADER>() {
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn GetRawInputBuffer(
    pData: PRAWINPUT,
    pcbSize: PUINT,
    cbSizeHeader: UINT,
) -> UINT {
    let pcti = (*GetWin32ClientInfo()).pClientThreadInfo;

    if pcbSize.is_null() || cbSizeHeader as usize != core::mem::size_of::<RAWINPUTHEADER>() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return UINT::MAX;
    }

    if pcti.is_null() || ((*pcti).fsWakeBits as u32 & QS_RAWINPUT) == 0 {
        *pcbSize = 0;
        return 0;
    }

    NtUserGetRawInputBuffer(pData, pcbSize, cbSizeHeader) as UINT
}

#[no_mangle]
pub unsafe extern "system" fn GetRawInputData(
    hRawInput: HRAWINPUT,
    uiCommand: UINT,
    pData: LPVOID,
    pcbSize: PUINT,
    cbSizeHeader: UINT,
) -> UINT {
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetRawInputDeviceList(
    pRawInputDeviceList: PRAWINPUTDEVICELIST,
    puiNumDevices: PUINT,
    cbSize: UINT,
) -> UINT {
    if !pRawInputDeviceList.is_null() {
        core::ptr::write_bytes(
            pRawInputDeviceList as *mut u8,
            0,
            core::mem::size_of::<PRAWINPUTDEVICELIST>(),
        );
    }
    if !puiNumDevices.is_null() {
        *puiNumDevices = 0;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn GetRegisteredRawInputDevices(
    pRawInputDevices: PRAWINPUTDEVICE,
    puiNumDevices: PUINT,
    cbSize: UINT,
) -> UINT {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RegisterRawInputDevices(
    pRawInputDevices: PCRAWINPUTDEVICE,
    uiNumDevices: UINT,
    cbSize: UINT,
) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DisplayExitWindowsWarnings(flags: ULONG) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn ReasonCodeNeedsBugID(reasoncode: ULONG) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn ReasonCodeNeedsComment(reasoncode: ULONG) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn CtxInitUser32() -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn EnterReaderModeHelper(hwnd: HWND) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn InitializeLpkHooks(hookfuncs: *mut FARPROC) {
}

#[no_mangle]
pub unsafe extern "system" fn InitializeWin32EntryTable(EntryTablePlus0x1000: *mut UCHAR) -> WORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IsServerSideWindow(wnd: HWND) -> BOOL {
    /* Upstream f7291343167 (CORE-19946): undocumented — TRUE when the window's
     * procedure lives in kernel mode. uxtheme from Windows calls it heavily. */
    let Wnd = crate::internal::ValidateHwnd(wnd);
    (!Wnd.is_null() && ((*Wnd).state & WNDS_SERVERSIDEWINDOWPROC) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn AllowForegroundActivation() {
}

#[no_mangle]
pub unsafe extern "system" fn ShowStartGlass(unknown: DWORD) {
    let _ = unknown;
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuIndex(hMenu: HMENU, hSubMenu: HMENU) -> u32 {
    NtUserGetMenuIndex(hMenu, hSubMenu)
}

#[no_mangle]
pub unsafe extern "system" fn UserRegisterWowHandlers(
    Unknown1: PVOID,
    Unknown2: PVOID,
) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn BuildReasonArray(Pointer: PVOID) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn CreateSystemThreads(Unused: DWORD) {
    NtUserxCreateSystemThreads(1 as BOOL);
    ExitThread(0);
}

#[no_mangle]
pub unsafe extern "system" fn DestroyReasons(Pointer: PVOID) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DeviceEventWorker(
    hwnd: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
    Data: DWORD,
    uResult: *mut ULONG_PTR,
) -> NTSTATUS {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let pusem: PUSER_DEVICE_EVENT_MSG =
        core::ptr::addr_of_mut!(ApiMessage.Data.DeviceEventMsg);

    (*pusem).hwnd = hwnd;
    (*pusem).wParam = wParam;
    (*pusem).lParam = lParam;
    (*pusem).Data = Data;
    (*pusem).Result = 0;

    TRACE!(
        b"DeviceEventWorker : hwnd %p, wParam %d, lParam %d, Data %d, uResult %p\n\0".as_ptr()
            as *const i8,
        hwnd,
        wParam,
        lParam,
        Data,
        uResult,
    );

    if lParam == 0 {
        CsrClientCallServer(
            core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
            core::ptr::null_mut(),
            CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpDeviceEvent),
            core::mem::size_of_val(&*pusem) as ULONG,
        );
    } else {
        let pdev_br: PDEV_BROADCAST_HDR = lParam as PDEV_BROADCAST_HDR;
        let BufferSize: ULONG = (*pdev_br).dbch_size;

        let pcsrcb: PCSR_CAPTURE_BUFFER = CsrAllocateCaptureBuffer(1, BufferSize);

        if pcsrcb.is_null() {
            return STATUS_NO_MEMORY;
        }

        CsrCaptureMessageBuffer(
            pcsrcb,
            lParam as LPVOID,
            BufferSize,
            core::ptr::addr_of_mut!((*pusem).lParam) as *mut LPVOID,
        );

        CsrClientCallServer(
            core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
            pcsrcb,
            CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpDeviceEvent),
            core::mem::size_of_val(&*pusem) as ULONG,
        );

        CsrFreeCaptureBuffer(pcsrcb);
    }

    if NT_SUCCESS(ApiMessage.Status) {
        *uResult = (*pusem).Result;
    }

    ApiMessage.Status
}

#[no_mangle]
pub unsafe extern "system" fn GetReasonTitleFromReasonCode(dw1: DWORD, dw2: DWORD, dw3: DWORD) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IsSETEnabled() -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RecordShutdownReason(dw0: DWORD) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn UserLpkTabbedTextOut(
    dw1: DWORD,
    dw2: DWORD,
    dw3: DWORD,
    dw4: DWORD,
    dw5: DWORD,
    dw6: DWORD,
    dw7: DWORD,
    dw8: DWORD,
    dw9: DWORD,
    dw10: DWORD,
    dw11: DWORD,
    dw12: DWORD,
) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn Win32PoolAllocationStats(
    dw1: DWORD,
    dw2: DWORD,
    dw3: DWORD,
    dw4: DWORD,
    dw5: DWORD,
) -> BOOL {
    0
}
