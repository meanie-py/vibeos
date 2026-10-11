use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type HSYNTHETICPOINTERDEVICE = HANDLE;

pub const POINTER_FEEDBACK_DEFAULT: i32 = 1;
pub const POINTER_FEEDBACK_INDIRECT: i32 = 2;
pub const POINTER_FEEDBACK_NONE: i32 = 3;
pub type POINTER_FEEDBACK_MODE = i32;

pub const ORIENTATION_PREFERENCE_NONE: i32 = 0x0;
pub const ORIENTATION_PREFERENCE_LANDSCAPE: i32 = 0x1;
pub const ORIENTATION_PREFERENCE_PORTRAIT: i32 = 0x2;
pub const ORIENTATION_PREFERENCE_LANDSCAPE_FLIPPED: i32 = 0x4;
pub const ORIENTATION_PREFERENCE_PORTRAIT_FLIPPED: i32 = 0x8;
pub type ORIENTATION_PREFERENCE = i32;

pub type HTOUCHINPUT = HANDLE;

#[repr(C)]
pub struct tagTOUCHINPUT {
    pub x: LONG,
    pub y: LONG,
    pub hSource: HANDLE,
    pub dwID: DWORD,
    pub dwFlags: DWORD,
    pub dwMask: DWORD,
    pub dwTime: DWORD,
    pub dwExtraInfo: ULONG_PTR,
    pub cxContact: DWORD,
    pub cyContact: DWORD,
}
pub type TOUCHINPUT = tagTOUCHINPUT;
pub type PTOUCHINPUT = *mut TOUCHINPUT;
pub type PCTOUCHINPUT = *const TOUCHINPUT;

pub type HGESTUREINFO = HANDLE;

pub const GF_BEGIN: u32 = 0x00000001;
pub const GF_INERTIA: u32 = 0x00000002;
pub const GF_END: u32 = 0x00000004;

#[repr(C)]
pub struct tagGESTURECONFIG {
    pub dwID: DWORD,
    pub dwWant: DWORD,
    pub dwBlock: DWORD,
}
pub type GESTURECONFIG = tagGESTURECONFIG;
pub type PGESTURECONFIG = *mut GESTURECONFIG;

#[repr(C)]
pub struct tagGESTUREINFO {
    pub cbSize: UINT,
    pub dwFlags: DWORD,
    pub dwID: DWORD,
    pub hwndTarget: HWND,
    pub ptsLocation: POINTS,
    pub dwInstanceID: DWORD,
    pub dwSequenceID: DWORD,
    pub ullArguments: ULONGLONG,
    pub cbExtraArgs: UINT,
}
pub type GESTUREINFO = tagGESTUREINFO;
pub type PGESTUREINFO = *mut GESTUREINFO;
pub type PCGESTUREINFO = *const GESTUREINFO;

pub const POINTER_CHANGE_NONE: i32 = 0;
pub const POINTER_CHANGE_FIRSTBUTTON_DOWN: i32 = 1;
pub const POINTER_CHANGE_FIRSTBUTTON_UP: i32 = 2;
pub const POINTER_CHANGE_SECONDBUTTON_DOWN: i32 = 3;
pub const POINTER_CHANGE_SECONDBUTTON_UP: i32 = 4;
pub const POINTER_CHANGE_THIRDBUTTON_DOWN: i32 = 5;
pub const POINTER_CHANGE_THIRDBUTTON_UP: i32 = 6;
pub const POINTER_CHANGE_FOURTHBUTTON_DOWN: i32 = 7;
pub const POINTER_CHANGE_FOURTHBUTTON_UP: i32 = 8;
pub const POINTER_CHANGE_FIFTHBUTTON_DOWN: i32 = 9;
pub const POINTER_CHANGE_FIFTHBUTTON_UP: i32 = 10;
pub type POINTER_BUTTON_CHANGE_TYPE = i32;

pub type HPOWERNOTIFY = PVOID;
pub type PHPOWERNOTIFY = *mut HPOWERNOTIFY;
pub type POINTER_FLAGS = UINT32;
pub type POINTER_INPUT_TYPE = DWORD;

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

pub type TOUCH_FLAGS = UINT32;
pub const TOUCH_FLAG_NONE: u32 = 0x00000000;

pub type TOUCH_MASK = UINT32;
pub const TOUCH_MASK_NONE: u32 = 0x00000000;
pub const TOUCH_MASK_CONTACTAREA: u32 = 0x00000001;
pub const TOUCH_MASK_ORIENTATION: u32 = 0x00000002;
pub const TOUCH_MASK_PRESSURE: u32 = 0x00000004;

#[repr(C)]
pub struct tagPOINTER_TOUCH_INFO {
    pub pointerInfo: POINTER_INFO,
    pub touchFlags: TOUCH_FLAGS,
    pub touchMask: TOUCH_MASK,
    pub rcContact: RECT,
    pub rcContactRaw: RECT,
    pub orientation: UINT32,
    pub pressure: UINT32,
}
pub type POINTER_TOUCH_INFO = tagPOINTER_TOUCH_INFO;

#[no_mangle]
pub unsafe extern "system" fn CloseTouchInputHandle(handle: HTOUCHINPUT) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetTouchInputInfo(handle: HTOUCHINPUT, count: UINT, ptr: *mut TOUCHINPUT, size: i32) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IsTouchWindow(hwnd: HWND, flags: *mut ULONG) -> BOOL {
    // FIXME( "hwnd %p, flags %p stub!\n", hwnd, flags );
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn RegisterTouchWindow(hwnd: HWND, flags: ULONG) -> BOOL {
    // FIXME( "hwnd %p, flags %#lx stub!\n", hwnd, flags );
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterTouchWindow(hwnd: HWND) -> BOOL {
    // FIXME( "hwnd %p stub!\n", hwnd );
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn CloseGestureInfoHandle(handle: HGESTUREINFO) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetGestureExtraArgs(handle: HGESTUREINFO, count: UINT, args: *mut BYTE) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetGestureInfo(handle: HGESTUREINFO, ptr: *mut GESTUREINFO) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetGestureConfig(
    hwnd: HWND,
    reserved: DWORD,
    flags: DWORD,
    count: *mut UINT,
    config: *mut GESTURECONFIG,
    size: UINT,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetGestureConfig(hwnd: HWND, reserved: DWORD, count: UINT, config: *mut GESTURECONFIG, size: UINT) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPointerTouchInfo(id: UINT32, info: *mut POINTER_TOUCH_INFO) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPointerTouchInfoHistory(id: UINT32, count: *mut UINT32, info: *mut POINTER_TOUCH_INFO) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CreateSyntheticPointerDevice(type_0: POINTER_INPUT_TYPE, max_count: ULONG, mode: POINTER_FEEDBACK_MODE) -> HSYNTHETICPOINTERDEVICE {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return core::ptr::null_mut();
}
