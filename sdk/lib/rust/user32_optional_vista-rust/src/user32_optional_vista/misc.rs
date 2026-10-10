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

#[no_mangle]
pub unsafe extern "system" fn RegisterPowerSettingNotification(recipient: HANDLE, guid: *const GUID, flags: DWORD) -> HPOWERNOTIFY {
    return 0xdeadbeef as usize as HPOWERNOTIFY;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterPowerSettingNotification(handle: HPOWERNOTIFY) -> BOOL {
    // FIXME("(%p): stub\n", handle);
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn RegisterSuspendResumeNotification(recipient: HANDLE, flags: DWORD) -> HPOWERNOTIFY {
    return 0xdeadbeef as HPOWERNOTIFY;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterSuspendResumeNotification(
    handle: HPOWERNOTIFY,
) -> BOOL {
    // FIXME("%p: stub.\n", handle);
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

    if device_count.is_null() {
        return (FALSE) as BOOL;
    }

    if !devices.is_null() {
        return (FALSE) as BOOL;
    }

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
    if id == 0 || type_0.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    *type_0 = PT_MOUSE as POINTER_INPUT_TYPE;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPointerInfo(id: UINT32, info: *mut POINTER_INFO) -> BOOL {
    SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    return (FALSE) as BOOL;
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
    // FIXME( "%p %x %ld %p\n", hwnd, message, action, changefilter );
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
