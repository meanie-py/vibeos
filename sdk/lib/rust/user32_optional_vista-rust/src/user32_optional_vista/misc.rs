use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const POINTER_DEVICE_PRODUCT_STRING_MAX: usize = 520;

pub const PT_POINTER: u32 = 1;
pub const PT_TOUCH: u32 = 2;
pub const PT_PEN: u32 = 3;
pub const PT_MOUSE: u32 = 4;
pub const PT_TOUCHPAD: u32 = 5;

pub type HPOWERNOTIFY = PVOID;
pub type PHPOWERNOTIFY = *mut PVOID;
pub type POINTER_FLAGS = UINT32;
pub type POINTER_INPUT_TYPE = DWORD;

#[allow(non_snake_case)]
#[repr(C)]
pub struct tagTOUCH_HIT_TESTING_PROXIMITY_EVALUATION {
    pub score: UINT16,
    pub adjustedPoint: POINT,
}
pub type TOUCH_HIT_TESTING_PROXIMITY_EVALUATION = tagTOUCH_HIT_TESTING_PROXIMITY_EVALUATION;
pub type PTOUCH_HIT_TESTING_PROXIMITY_EVALUATION = *mut tagTOUCH_HIT_TESTING_PROXIMITY_EVALUATION;

#[allow(non_snake_case)]
#[repr(C)]
pub struct tagTOUCH_HIT_TESTING_INPUT {
    pub pointerId: UINT32,
    pub point: POINT,
    pub boundingBox: RECT,
    pub nonOccludedBoundingBox: RECT,
    pub orientation: UINT32,
}
pub type TOUCH_HIT_TESTING_INPUT = tagTOUCH_HIT_TESTING_INPUT;
pub type PTOUCH_HIT_TESTING_INPUT = *mut tagTOUCH_HIT_TESTING_INPUT;

pub type POINTER_DEVICE_TYPE = u32;
pub const POINTER_DEVICE_TYPE_INTEGRATED_PEN: u32 = 0x00000001;
pub const POINTER_DEVICE_TYPE_EXTERNAL_PEN: u32 = 0x00000002;
pub const POINTER_DEVICE_TYPE_TOUCH: u32 = 0x00000003;
pub const POINTER_DEVICE_TYPE_TOUCH_PAD: u32 = 0x00000004;
pub const POINTER_DEVICE_TYPE_MAX: u32 = 0xFFFFFFFF;

#[allow(non_snake_case)]
#[repr(C)]
pub struct tagPOINTER_DEVICE_INFO {
    pub displayOrientation: DWORD,
    pub device: HANDLE,
    pub pointerDeviceType: POINTER_DEVICE_TYPE,
    pub monitor: HMONITOR,
    pub startingCursorId: ULONG,
    pub maxActiveContacts: USHORT,
    pub productString: [WCHAR; POINTER_DEVICE_PRODUCT_STRING_MAX],
}
pub type POINTER_DEVICE_INFO = tagPOINTER_DEVICE_INFO;

pub type POINTER_BUTTON_CHANGE_TYPE = u32;
pub const POINTER_CHANGE_NONE: u32 = 0;
pub const POINTER_CHANGE_FIRSTBUTTON_DOWN: u32 = 1;
pub const POINTER_CHANGE_FIRSTBUTTON_UP: u32 = 2;
pub const POINTER_CHANGE_SECONDBUTTON_DOWN: u32 = 3;
pub const POINTER_CHANGE_SECONDBUTTON_UP: u32 = 4;
pub const POINTER_CHANGE_THIRDBUTTON_DOWN: u32 = 5;
pub const POINTER_CHANGE_THIRDBUTTON_UP: u32 = 6;
pub const POINTER_CHANGE_FOURTHBUTTON_DOWN: u32 = 7;
pub const POINTER_CHANGE_FOURTHBUTTON_UP: u32 = 8;
pub const POINTER_CHANGE_FIFTHBUTTON_DOWN: u32 = 9;
pub const POINTER_CHANGE_FIFTHBUTTON_UP: u32 = 10;

#[allow(non_snake_case)]
#[repr(C)]
pub struct tagPOINTER_INFO {
    pub pointerType: POINTER_INPUT_TYPE,
    pub pointerId: UINT32,
    pub frameId: UINT32,
    pub pointerFlags: POINTER_FLAGS,
    pub sourceDevice: HANDLE,
    pub hwndTarget: HWND,
    pub ptPixelLocation: POINT,
    pub ptHimetricLocation: POINT,
    pub ptPixelLocationRaw: POINT,
    pub ptHimetricLocationRaw: POINT,
    pub dwTime: DWORD,
    pub historyCount: UINT32,
    pub InputData: INT32,
    pub dwKeyStates: DWORD,
    pub PerformanceCount: UINT64,
    pub ButtonChangeType: POINTER_BUTTON_CHANGE_TYPE,
}
pub type POINTER_INFO = tagPOINTER_INFO;

#[allow(non_snake_case)]
#[repr(C)]
pub struct tagCHANGEFILTERSTRUCT {
    pub cbSize: DWORD,
    pub ExtStatus: DWORD,
}
pub type CHANGEFILTERSTRUCT = tagCHANGEFILTERSTRUCT;
pub type PCHANGEFILTERSTRUCT = *mut tagCHANGEFILTERSTRUCT;

// No power-setting / suspend-resume notification is ever delivered on this machine (there is
// no power manager to raise one); the registration is only bookkeeping, so the handle is a
// unique opaque token and nothing is remembered behind it.
static NEXT_POWER_NOTIFY: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(1);

fn new_power_notify_handle() -> HPOWERNOTIFY {
    // Stride 16 keeps the token clear of small integers that callers treat as "failure".
    let n = NEXT_POWER_NOTIFY.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    (n << 4) as usize as HPOWERNOTIFY
}

#[no_mangle]
pub unsafe extern "system" fn RegisterPowerSettingNotification(recipient: HANDLE, guid: *const GUID, flags: DWORD) -> HPOWERNOTIFY {
    if recipient.is_null() || guid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    new_power_notify_handle()
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterPowerSettingNotification(handle: HPOWERNOTIFY) -> BOOL {
    if handle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as i32;
    }
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn RegisterSuspendResumeNotification(recipient: HANDLE, flags: DWORD) -> HPOWERNOTIFY {
    if recipient.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    new_power_notify_handle()
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterSuspendResumeNotification(
    handle: HPOWERNOTIFY,
) -> BOOL {
    if handle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as i32;
    }
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn IsWindowRedirectedForPrint(hwnd: HWND) -> BOOL {
    // FIXME("(%p): stub\n", hwnd);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn RegisterPointerDeviceNotifications(
    hwnd: HWND,
    notifyrange: BOOL,
) -> BOOL {
    // FIXME("(%p %d): stub\n", hwnd, notifyrange);
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn GetPointerDevices(device_count: *mut UINT32, devices: *mut POINTER_DEVICE_INFO) -> BOOL {
    // No pointer (touch/pen) device exists; the mouse is not a "pointer device" here.
    if device_count.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    let _ = devices;
    *device_count = 0;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterTouchHitTestingWindow(hwnd: HWND, value: ULONG) -> BOOL {
    // FIXME("(%p %ld): stub\n", hwnd, value);
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn EvaluateProximityToRect(
    box_0: *const RECT,
    input: *const TOUCH_HIT_TESTING_INPUT,
    proximity: *mut TOUCH_HIT_TESTING_PROXIMITY_EVALUATION,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PackTouchHitTestingProximityEvaluation(input: *const TOUCH_HIT_TESTING_INPUT, proximity: *const TOUCH_HIT_TESTING_PROXIMITY_EVALUATION) -> LRESULT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetPointerType(id: UINT32, type_0: *mut POINTER_INPUT_TYPE) -> BOOL {
    // The only pointer is the mouse, id 1.
    if id != MOUSE_POINTER_ID || type_0.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    *type_0 = PT_MOUSE as POINTER_INPUT_TYPE;
    return (TRUE) as BOOL;
}

const MOUSE_POINTER_ID: UINT32 = 1;
const POINTER_FLAG_INRANGE: u32 = 0x00000002;
const POINTER_FLAG_INCONTACT: u32 = 0x00000004;
const POINTER_FLAG_FIRSTBUTTON: u32 = 0x00000010;
const POINTER_FLAG_PRIMARY: u32 = 0x00002000;
const POINTER_FLAG_UPDATE: u32 = 0x00020000;
const VK_LBUTTON_KEY: i32 = 0x01;

extern "system" {
    fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    fn GetAsyncKeyState(vKey: i32) -> i16;
    fn GetTickCount() -> DWORD;
    fn QueryPerformanceCounter(lpPerformanceCount: *mut i64) -> BOOL;
    fn WindowFromPoint(point: POINT) -> HWND;
    fn IsWindow(hWnd: HWND) -> BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPointerInfo(id: UINT32, info: *mut POINTER_INFO) -> BOOL {
    if id != MOUSE_POINTER_ID || info.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    let mut pt: POINT = core::mem::zeroed();
    if GetCursorPos(&mut pt) == 0 {
        return (FALSE) as BOOL;
    }
    core::ptr::write_bytes(info, 0, 1);
    let i = &mut *info;
    i.pointerType = PT_MOUSE as POINTER_INPUT_TYPE;
    i.pointerId = MOUSE_POINTER_ID;
    i.pointerFlags = POINTER_FLAG_INRANGE | POINTER_FLAG_PRIMARY | POINTER_FLAG_UPDATE;
    if GetAsyncKeyState(VK_LBUTTON_KEY) < 0 {
        i.pointerFlags |= POINTER_FLAG_INCONTACT | POINTER_FLAG_FIRSTBUTTON;
    }
    i.hwndTarget = WindowFromPoint(pt);
    i.ptPixelLocation = pt;
    i.ptPixelLocationRaw = pt;
    // HIMETRIC is 0.01 mm: 2540 per inch, and this machine is 96 DPI.
    let himetric = |v: i32| ((v as i64 * 2540 + 48).div_euclid(96)) as i32;
    i.ptHimetricLocation = POINT { x: himetric(pt.x), y: himetric(pt.y) };
    i.ptHimetricLocationRaw = i.ptHimetricLocation;
    i.dwTime = GetTickCount();
    i.historyCount = 1;
    let mut pc: i64 = 0;
    QueryPerformanceCounter(&mut pc);
    i.PerformanceCount = pc as UINT64;
    i.ButtonChangeType = POINTER_CHANGE_NONE as _;
    (TRUE) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn ChangeWindowMessageFilter(message: UINT, flag: DWORD) -> BOOL {
    // FIXME( "%x %08lx\n", message, flag );
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn ChangeWindowMessageFilterEx(
    hwnd: HWND,
    message: UINT,
    action: DWORD,
    changefilter: *mut CHANGEFILTERSTRUCT,
) -> BOOL {
    // There is no UIPI here: every message is already allowed, so any change succeeds.
    if hwnd.is_null() || IsWindow(hwnd) == 0 {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if !changefilter.is_null() {
        if (*changefilter).cbSize as usize != core::mem::size_of::<CHANGEFILTERSTRUCT>() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return (FALSE) as BOOL;
        }
        (*changefilter).ExtStatus = 0; // MSGFLTINFO_NONE
    }
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn ShutdownBlockReasonDestroy(hwnd: HWND) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ShutdownBlockReasonCreate(hwnd: HWND, reason: LPCWSTR) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

// Measured from the mingw-w64 SDK headers (sizeof): 96 bytes on 64-bit, 88 on i386.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<POINTER_INFO>() == 96);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<POINTER_INFO>() == 88);
