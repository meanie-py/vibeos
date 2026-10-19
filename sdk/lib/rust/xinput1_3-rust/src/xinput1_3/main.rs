// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0A6804564E14: [u16; 23] = [
    0x005Cu16, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x0049, 0x006E, 0x0070,
    0x0075, 0x0074, 0x005C, 0x004A, 0x006F, 0x0079, 0x0073, 0x0074, 0x0069, 0x0063,
    0x006B, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_52B0C056F710: [u16; 9] = [0x0064u16, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_7B5EDCF11682: [u16; 26] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0065, 0x005C, 0x0041, 0x0070, 0x0070, 0x0044, 0x0065, 0x0066,
    0x0061, 0x0075, 0x006C, 0x0074, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_AD1FFBB38714: [u16; 9] = [0x006Fu16, 0x0076, 0x0065, 0x0072, 0x0072, 0x0069, 0x0064, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_C4B1D9B41A0C: [u16; 23] = [
    0x0077u16, 0x0069, 0x006E, 0x0065, 0x005F, 0x0078, 0x0069, 0x006E, 0x0070, 0x0075,
    0x0074, 0x005F, 0x0068, 0x0069, 0x0064, 0x005F, 0x0075, 0x0070, 0x0064, 0x0061,
    0x0074, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_F1850E60EE4D: [u16; 36] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0065, 0x005C, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074,
    0x0049, 0x006E, 0x0070, 0x0075, 0x0074, 0x005C, 0x004A, 0x006F, 0x0079, 0x0073,
    0x0074, 0x0069, 0x0063, 0x006B, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_FEB45BBE340C: [u16; 24] = [
    0x005Fu16, 0x005F, 0x0077, 0x0069, 0x006E, 0x0065, 0x005F, 0x0078, 0x0069, 0x006E,
    0x0070, 0x0075, 0x0074, 0x005F, 0x0064, 0x0065, 0x0076, 0x006E, 0x006F, 0x0074,
    0x0069, 0x0066, 0x0079, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_10 {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_11,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_16 {
    pub c2rust_unnamed: C2RustUnnamed_17,
    pub Pointer: PVOID,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub Offset: DWORD,
    pub OffsetHigh: DWORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub Usage: USAGE,
    pub Reserved1: USAGE,
    pub StringIndex: USHORT,
    pub Reserved2: USHORT,
    pub DesignatorIndex: USHORT,
    pub Reserved3: USHORT,
    pub DataIndex: USHORT,
    pub Reserved4: USHORT,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_21 {
    pub Range: C2RustUnnamed_23,
    pub NotRange: C2RustUnnamed_22,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
    pub UsageMin: USAGE,
    pub UsageMax: USAGE,
    pub StringMin: USHORT,
    pub StringMax: USHORT,
    pub DesignatorMin: USHORT,
    pub DesignatorMax: USHORT,
    pub DataIndexMin: USHORT,
    pub DataIndexMax: USHORT,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
    pub preparsed: PHIDP_PREPARSED_DATA,
    pub caps: HIDP_CAPS,
    pub lx_caps: HIDP_VALUE_CAPS,
    pub ly_caps: HIDP_VALUE_CAPS,
    pub lt_caps: HIDP_VALUE_CAPS,
    pub rx_caps: HIDP_VALUE_CAPS,
    pub ry_caps: HIDP_VALUE_CAPS,
    pub rt_caps: HIDP_VALUE_CAPS,
    pub read_ovl: OVERLAPPED,
    pub input_report_buf: *mut ::core::ffi::c_char,
    pub output_report_buf: *mut ::core::ffi::c_char,
    pub feature_report_buf: *mut ::core::ffi::c_char,
    pub haptics_report: BYTE,
    pub haptics_rumble_caps: HIDP_VALUE_CAPS,
    pub haptics_buzz_caps: HIDP_VALUE_CAPS,
}

#[no_mangle]
pub static mut GUID_DEVINTERFACE_WINEXINPUT: GUID = _GUID {
    Data1: 0x6c53d5fd as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6480 as ::core::ffi::c_ushort,
    Data3: 0x440f as ::core::ffi::c_ushort,
    Data4: [
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut xinput_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const xinput_cs_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut xinput_cs_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut xinput_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut xinput_cs_debug.ProcessLocksList,
            Blink: &raw mut xinput_cs_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

// C: `static struct xinput_controller controllers[XUSER_MAX_COUNT];` -- zero-initialized.
static mut controllers: [xinput_controller; 4] = unsafe { core::mem::zeroed() };

static mut xinput_instance: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut start_event: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut update_event: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut state_lock: SRWLOCK = _RTL_SRWLOCK {
    Ptr: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
};

static mut current_state: [XINPUT_STATE; 4] = [_XINPUT_STATE {
    dwPacketNumber: 0,
    Gamepad: _XINPUT_GAMEPAD {
        wButtons: 0,
        bLeftTrigger: 0,
        bRightTrigger: 0,
        sThumbLX: 0,
        sThumbLY: 0,
        sThumbRX: 0,
        sThumbRY: 0,
    },
}; 4];

static mut JS_STATE_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

static mut JS_STATE_LOW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

static mut JS_STATE_HIGH: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const XINPUT_GAMEPAD_GUIDE: u32 = 0x0400;


#[repr(C)]
pub struct xinput_controller_hid {
    pub preparsed: PHIDP_PREPARSED_DATA,
    pub caps: HIDP_CAPS,
    pub lx_caps: HIDP_VALUE_CAPS,
    pub ly_caps: HIDP_VALUE_CAPS,
    pub lt_caps: HIDP_VALUE_CAPS,
    pub rx_caps: HIDP_VALUE_CAPS,
    pub ry_caps: HIDP_VALUE_CAPS,
    pub rt_caps: HIDP_VALUE_CAPS,

    pub read_ovl: OVERLAPPED,

    pub input_report_buf: *mut i8,
    pub output_report_buf: *mut i8,
    pub feature_report_buf: *mut i8,

    pub haptics_report: BYTE,
    pub haptics_rumble_caps: HIDP_VALUE_CAPS,
    pub haptics_buzz_caps: HIDP_VALUE_CAPS,
}

#[repr(C)]
pub struct xinput_controller {
    pub caps: XINPUT_CAPABILITIES,
    pub vibration: XINPUT_VIBRATION,
    pub device: HANDLE,
    pub device_path: [WCHAR; (MAX_PATH) as _],
    pub read_event: HANDLE,
    pub enabled: BOOL,

    pub hid: xinput_controller_hid,
}

// WINE_DEFAULT_DEBUG_CHANNEL(xinput): debug channel is provided by the crate's
// trace/debug macros; no declaration emitted here.

unsafe extern "C" fn set_current_state(index: UINT, state: *const XINPUT_STATE) {
    AcquireSRWLockExclusive(core::ptr::addr_of_mut!(state_lock));
    if state.is_null() {
        memset(
            (core::ptr::addr_of_mut!(current_state) as *mut XINPUT_STATE).add(index as usize)
                as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<XINPUT_STATE>(),
        );
    } else {
        memcpy(
            (core::ptr::addr_of_mut!(current_state) as *mut XINPUT_STATE).add(index as usize)
                as *mut core::ffi::c_void,
            state as *const core::ffi::c_void,
            core::mem::size_of::<XINPUT_STATE>(),
        );
        let p: *mut XINPUT_STATE =
            (core::ptr::addr_of_mut!(current_state) as *mut XINPUT_STATE).add(index as usize);
        (*p).dwPacketNumber = (*p).dwPacketNumber.wrapping_add(1);
        if (*p).dwPacketNumber == 0 {
            (*p).dwPacketNumber = (*p).dwPacketNumber.wrapping_add(1);
        }
    }
    ReleaseSRWLockExclusive(core::ptr::addr_of_mut!(state_lock));
}

unsafe extern "C" fn get_current_state(mut index: UINT, mut state: *mut XINPUT_STATE) -> BOOL {
    AcquireSRWLockShared(core::ptr::addr_of_mut!(state_lock));
    memcpy(
        state as *mut core::ffi::c_void,
        (core::ptr::addr_of_mut!(current_state) as *mut XINPUT_STATE).offset(index as isize)
            as *const core::ffi::c_void,
        core::mem::size_of::<XINPUT_STATE>(),
    );
    ReleaseSRWLockShared(core::ptr::addr_of_mut!(state_lock));

    return (*state).dwPacketNumber as BOOL;
}

unsafe extern "C" fn check_value_caps(
    mut controller: *mut xinput_controller,
    mut usage: USHORT,
    mut caps: *mut HIDP_VALUE_CAPS,
) {
    match usage {
        HID_USAGE_GENERIC_X => {
            (*controller).hid.lx_caps = *caps;
        }
        HID_USAGE_GENERIC_Y => {
            (*controller).hid.ly_caps = *caps;
        }
        HID_USAGE_GENERIC_Z => {
            (*controller).hid.lt_caps = *caps;
        }
        HID_USAGE_GENERIC_RX => {
            (*controller).hid.rx_caps = *caps;
        }
        HID_USAGE_GENERIC_RY => {
            (*controller).hid.ry_caps = *caps;
        }
        HID_USAGE_GENERIC_RZ => {
            (*controller).hid.rt_caps = *caps;
        }
        _ => {}
    }
}

unsafe extern "C" fn check_waveform_caps(
    controller: *mut xinput_controller,
    device: HANDLE,
    preparsed: PHIDP_PREPARSED_DATA,
    collections: *mut HIDP_LINK_COLLECTION_NODE,
    caps: *mut HIDP_VALUE_CAPS,
) {
    let mut count: USHORT = 0;
    let mut report_len: USHORT = (*controller).hid.caps.FeatureReportByteLength;
    let mut report_buf: *mut ::core::ffi::c_char = (*controller).hid.feature_report_buf;
    let mut parent: ULONG = (*caps).LinkCollection as ULONG;
    let mut waveform: ULONG = 0 as ULONG;
    let mut value_caps: HIDP_VALUE_CAPS = core::mem::zeroed();
    let mut phy_usages: USAGE_AND_PAGE = core::mem::zeroed();
    let mut status: NTSTATUS = 0;

    while (*collections.offset(parent as isize)).LinkUsagePage as ::core::ffi::c_int
        != HID_USAGE_PAGE_HAPTICS as ::core::ffi::c_int
        || (*collections.offset(parent as isize)).LinkUsage as ::core::ffi::c_int
            != HID_USAGE_HAPTICS_SIMPLE_CONTROLLER as ::core::ffi::c_int
    {
        parent = (*collections.offset(parent as isize)).Parent as ULONG;
        if parent == 0 {
            break;
        }
    }

    if (*collections.offset(parent as isize)).LinkUsagePage as ::core::ffi::c_int
        != HID_USAGE_PAGE_HAPTICS as ::core::ffi::c_int
        || (*collections.offset(parent as isize)).LinkUsage as ::core::ffi::c_int
            != HID_USAGE_HAPTICS_SIMPLE_CONTROLLER as ::core::ffi::c_int
    {
        WARN!("Failed to find haptics simple controller collection\n");
        return;
    }
    phy_usages.UsagePage =
        (*collections.offset((*collections.offset(parent as isize)).Parent as isize)).LinkUsagePage;
    phy_usages.Usage =
        (*collections.offset((*collections.offset(parent as isize)).Parent as isize)).LinkUsage;

    status = HidP_InitializeReportForID(
        HidP_Feature,
        (*caps).ReportID,
        preparsed,
        report_buf as PCHAR,
        report_len as ULONG,
    );
    if status != HIDP_STATUS_SUCCESS as NTSTATUS {
        WARN!("HidP_InitializeReportForID returned %#lx\n", status);
    }
    if HidD_GetFeature(device, report_buf as PVOID, report_len as ULONG) == 0 {
        WARN!(
            "Failed to get waveform list report, error %lu\n",
            GetLastError()
        );
        return;
    }

    status = HidP_GetUsageValue(
        HidP_Feature,
        (*caps).UsagePage,
        (*caps).LinkCollection,
        (*caps).__bindgen_anon_1.NotRange.Usage,
        core::ptr::addr_of_mut!(waveform),
        preparsed,
        report_buf as PCHAR,
        report_len as ULONG,
    );
    if status != HIDP_STATUS_SUCCESS as NTSTATUS {
        WARN!("HidP_GetUsageValue returned %#lx\n", status);
    }

    count = 1 as USHORT;
    status = HidP_GetSpecificValueCaps(
        HidP_Output,
        HID_USAGE_PAGE_HAPTICS,
        parent as USHORT,
        HID_USAGE_HAPTICS_INTENSITY,
        core::ptr::addr_of_mut!(value_caps),
        core::ptr::addr_of_mut!(count),
        preparsed,
    );
    if status != HIDP_STATUS_SUCCESS as NTSTATUS || count == 0 {
        WARN!(
            "Failed to get waveform intensity caps, status %#lx\n",
            status
        );
    } else if phy_usages.UsagePage as ::core::ffi::c_int
        == HID_USAGE_PAGE_GENERIC as ::core::ffi::c_int
        && phy_usages.Usage as ::core::ffi::c_int == HID_USAGE_GENERIC_Z as ::core::ffi::c_int
    {
        TRACE!("Ignoring left rumble caps\n");
    } else if phy_usages.UsagePage as ::core::ffi::c_int
        == HID_USAGE_PAGE_GENERIC as ::core::ffi::c_int
        && phy_usages.Usage as ::core::ffi::c_int == HID_USAGE_GENERIC_RZ as ::core::ffi::c_int
    {
        TRACE!("Ignoring right rumble caps\n");
    } else if waveform == HID_USAGE_HAPTICS_WAVEFORM_RUMBLE as ULONG {
        (*controller).hid.haptics_report = value_caps.ReportID as BYTE;
        (*controller).hid.haptics_rumble_caps = value_caps;
    } else if waveform == HID_USAGE_HAPTICS_WAVEFORM_BUZZ as ULONG {
        (*controller).hid.haptics_report = value_caps.ReportID as BYTE;
        (*controller).hid.haptics_buzz_caps = value_caps;
    } else {
        FIXME!("Unsupported waveform type %#lx\n", waveform);
    }
}

unsafe extern "C" fn controller_check_caps(
    mut controller: *mut xinput_controller,
    mut device: HANDLE,
    mut preparsed: PHIDP_PREPARSED_DATA,
) -> BOOL {
    let mut caps_count: USHORT = 0 as USHORT;
    let mut waveform_caps_count: USHORT = 0 as USHORT;
    let mut caps: *mut XINPUT_CAPABILITIES = &raw mut (*controller).caps;
    let mut collections: *mut HIDP_LINK_COLLECTION_NODE = core::ptr::null_mut();
    let mut waveform_caps: [HIDP_VALUE_CAPS; 8] = core::mem::zeroed();
    let mut button_caps: *mut HIDP_BUTTON_CAPS = core::ptr::null_mut();
    let mut collections_count: ULONG = 0 as ULONG;
    let mut value_caps: *mut HIDP_VALUE_CAPS = core::ptr::null_mut();
    let mut i: ::core::ffi::c_int = 0;
    let mut u: ::core::ffi::c_int = 0;
    let mut button_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut status: NTSTATUS = 0;

    /* Count buttons */
    memset(
        caps as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<XINPUT_CAPABILITIES>() as size_t,
    );

    button_caps = malloc(
        (core::mem::size_of::<HIDP_BUTTON_CAPS>() as size_t)
            .wrapping_mul((*controller).hid.caps.NumberInputButtonCaps as size_t),
    ) as *mut HIDP_BUTTON_CAPS;
    if button_caps.is_null() {
        return FALSE as BOOL;
    }
    status = HidP_GetButtonCaps!(
        HidP_Input,
        button_caps,
        &raw mut (*controller).hid.caps.NumberInputButtonCaps,
        preparsed,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!("HidP_GetButtonCaps returned %#lx\n", status);
    } else {
        i = 0 as ::core::ffi::c_int;
        while i < (*controller).hid.caps.NumberInputButtonCaps as ::core::ffi::c_int {
            if !((*button_caps.offset(i as isize)).UsagePage as ::core::ffi::c_int
                != HID_USAGE_PAGE_BUTTON as ::core::ffi::c_int)
            {
                if (*button_caps.offset(i as isize)).IsRange != 0 {
                    button_count = max!(
                        button_count,
                        (*button_caps.offset(i as isize)).__bindgen_anon_1.Range.UsageMax as ::core::ffi::c_int,
                    );
                } else {
                    button_count = max!(
                        button_count,
                        (*button_caps.offset(i as isize)).__bindgen_anon_1.NotRange.Usage as ::core::ffi::c_int,
                    );
                }
            }
            i += 1;
        }
    }
    free(button_caps as *mut ::core::ffi::c_void);
    if button_count < 11 as ::core::ffi::c_int {
        WARN!("Too few buttons, continuing anyway\n");
    }
    (*caps).Gamepad.wButtons = 0xffff as WORD;

    value_caps = malloc(
        (core::mem::size_of::<HIDP_VALUE_CAPS>() as size_t)
            .wrapping_mul((*controller).hid.caps.NumberInputValueCaps as size_t),
    ) as *mut HIDP_VALUE_CAPS;
    if value_caps.is_null() {
        return FALSE as BOOL;
    }
    status = HidP_GetValueCaps!(
        HidP_Input,
        value_caps,
        &raw mut (*controller).hid.caps.NumberInputValueCaps,
        preparsed,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!("HidP_GetValueCaps returned %#lx\n", status);
    } else {
        i = 0 as ::core::ffi::c_int;
        while i < (*controller).hid.caps.NumberInputValueCaps as ::core::ffi::c_int {
            let caps: *mut HIDP_VALUE_CAPS = value_caps.offset(i as isize);
            if !((*caps).UsagePage as ::core::ffi::c_int
                != HID_USAGE_PAGE_GENERIC as ::core::ffi::c_int)
            {
                if (*caps).IsRange == 0 {
                    check_value_caps(controller, (*caps).__bindgen_anon_1.NotRange.Usage as USHORT, caps);
                } else {
                    u = (*caps).__bindgen_anon_1.Range.UsageMin as ::core::ffi::c_int;
                    while u <= (*caps).__bindgen_anon_1.Range.UsageMax as ::core::ffi::c_int {
                        check_value_caps(controller, u as USHORT, value_caps.offset(i as isize));
                        u += 1;
                    }
                }
            }
            i += 1;
        }
    }
    free(value_caps as *mut ::core::ffi::c_void);

    if (*controller).hid.lt_caps.UsagePage == 0 {
        WARN!("Missing axis LeftTrigger\n");
    } else {
        (*caps).Gamepad.bLeftTrigger = (((1 as ::core::ffi::c_uint)
            << (core::mem::size_of::<BYTE>() as usize)
                .wrapping_add(1 as usize))
            .wrapping_sub(1 as ::core::ffi::c_uint)) as BYTE;
    }
    if (*controller).hid.rt_caps.UsagePage == 0 {
        WARN!("Missing axis RightTrigger\n");
    } else {
        (*caps).Gamepad.bRightTrigger = (((1 as ::core::ffi::c_uint)
            << (core::mem::size_of::<BYTE>() as usize)
                .wrapping_add(1 as usize))
            .wrapping_sub(1 as ::core::ffi::c_uint)) as BYTE;
    }
    if (*controller).hid.lx_caps.UsagePage == 0 {
        WARN!("Missing axis ThumbLX\n");
    } else {
        (*caps).Gamepad.sThumbLX = (((1 as ::core::ffi::c_uint)
            << (core::mem::size_of::<SHORT>() as usize)
                .wrapping_add(1 as usize))
            .wrapping_sub(1 as ::core::ffi::c_uint)) as SHORT;
    }
    if (*controller).hid.ly_caps.UsagePage == 0 {
        WARN!("Missing axis ThumbLY\n");
    } else {
        (*caps).Gamepad.sThumbLY = (((1 as ::core::ffi::c_uint)
            << (core::mem::size_of::<SHORT>() as usize)
                .wrapping_add(1 as usize))
            .wrapping_sub(1 as ::core::ffi::c_uint)) as SHORT;
    }
    if (*controller).hid.rx_caps.UsagePage == 0 {
        WARN!("Missing axis ThumbRX\n");
    } else {
        (*caps).Gamepad.sThumbRX = (((1 as ::core::ffi::c_uint)
            << (core::mem::size_of::<SHORT>() as usize)
                .wrapping_add(1 as usize))
            .wrapping_sub(1 as ::core::ffi::c_uint)) as SHORT;
    }
    if (*controller).hid.ry_caps.UsagePage == 0 {
        WARN!("Missing axis ThumbRY\n");
    } else {
        (*caps).Gamepad.sThumbRY = (((1 as ::core::ffi::c_uint)
            << (core::mem::size_of::<SHORT>() as usize)
                .wrapping_add(1 as usize))
            .wrapping_sub(1 as ::core::ffi::c_uint)) as SHORT;
    }

    (*caps).Type = XINPUT_DEVTYPE_GAMEPAD as BYTE;
    (*caps).SubType = XINPUT_DEVSUBTYPE_GAMEPAD as BYTE;

    collections_count = (*controller).hid.caps.NumberLinkCollectionNodes as ULONG;
    collections = malloc(
        (core::mem::size_of::<HIDP_LINK_COLLECTION_NODE>() as size_t)
            .wrapping_mul((*controller).hid.caps.NumberLinkCollectionNodes as size_t),
    ) as *mut HIDP_LINK_COLLECTION_NODE;
    if collections.is_null() {
        return FALSE as BOOL;
    }
    status = HidP_GetLinkCollectionNodes(
        collections,
        &raw mut collections_count,
        preparsed,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!("HidP_GetLinkCollectionNodes returned %#lx\n", status);
    } else {
        i = 0 as ::core::ffi::c_int;
        while (i as ULONG) < collections_count {
            if !((*collections.offset(i as isize)).LinkUsagePage as ::core::ffi::c_int
                != HID_USAGE_PAGE_HAPTICS as ::core::ffi::c_int)
            {
                if (*collections.offset(i as isize)).LinkUsage as ::core::ffi::c_int
                    == HID_USAGE_HAPTICS_WAVEFORM_LIST as ::core::ffi::c_int
                {
                    caps_count = (ARRAY_SIZE!(waveform_caps) as usize)
                        .wrapping_sub(waveform_caps_count as usize)
                        as USHORT;
                    value_caps = (&raw mut waveform_caps as *mut HIDP_VALUE_CAPS)
                        .offset(waveform_caps_count as ::core::ffi::c_int as isize);
                    status = HidP_GetSpecificValueCaps(
                        HidP_Feature,
                        HID_USAGE_PAGE_ORDINAL,
                        i as USHORT,
                        0,
                        value_caps,
                        &raw mut caps_count,
                        preparsed,
                    );
                    if status == HIDP_STATUS_SUCCESS {
                        waveform_caps_count = (waveform_caps_count as ::core::ffi::c_int
                            + caps_count as ::core::ffi::c_int)
                            as USHORT;
                    }
                }
            }
            i += 1;
        }
    }
    i = 0 as ::core::ffi::c_int;
    while i < waveform_caps_count as ::core::ffi::c_int {
        check_waveform_caps(
            controller,
            device,
            preparsed,
            collections,
            (&raw mut waveform_caps as *mut HIDP_VALUE_CAPS).offset(i as isize),
        );
        i += 1;
    }
    free(collections as *mut ::core::ffi::c_void);

    if (*controller).hid.haptics_rumble_caps.UsagePage as ::core::ffi::c_int != 0
        || (*controller).hid.haptics_buzz_caps.UsagePage as ::core::ffi::c_int != 0
    {
        (*caps).Flags |= XINPUT_CAPS_FFB_SUPPORTED as WORD;
        (*caps).Vibration.wLeftMotorSpeed = 255 as WORD;
        (*caps).Vibration.wRightMotorSpeed = 255 as WORD;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn HID_set_state(
    controller: *mut xinput_controller,
    state: *mut XINPUT_VIBRATION,
) -> DWORD {
    let mut report_len: ULONG = (*controller).hid.caps.OutputReportByteLength as ULONG;
    let mut preparsed: PHIDP_PREPARSED_DATA = (*controller).hid.preparsed;
    let mut report_buf: *mut c_char = (*controller).hid.output_report_buf;
    let mut ret: BOOL = 0;
    let mut update_rumble: BOOL = 0;
    let mut update_buzz: BOOL = 0;
    let mut collection: USHORT = 0;
    let mut status: NTSTATUS = 0;
    let mut report_id: BYTE = 0;

    if ((*controller).caps.Flags & (XINPUT_CAPS_FFB_SUPPORTED as WORD)) == 0 {
        return ERROR_SUCCESS as DWORD;
    }

    update_rumble = ((*controller).vibration.wLeftMotorSpeed != (*state).wLeftMotorSpeed) as BOOL;
    (*controller).vibration.wLeftMotorSpeed = (*state).wLeftMotorSpeed;
    update_buzz = ((*controller).vibration.wRightMotorSpeed != (*state).wRightMotorSpeed) as BOOL;
    (*controller).vibration.wRightMotorSpeed = (*state).wRightMotorSpeed;

    if (*controller).enabled == 0 {
        return ERROR_SUCCESS as DWORD;
    }
    if update_rumble == 0 && update_buzz == 0 {
        return ERROR_SUCCESS as DWORD;
    }

    report_id = (*controller).hid.haptics_report;
    status = HidP_InitializeReportForID(
        HidP_Output,
        report_id as UCHAR,
        preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!("HidP_InitializeReportForID returned %#lx\n", status);
    }

    collection = (*controller).hid.haptics_rumble_caps.LinkCollection;
    status = HidP_SetUsageValue(
        HidP_Output,
        HID_USAGE_PAGE_HAPTICS,
        collection,
        HID_USAGE_HAPTICS_INTENSITY,
        (*state).wLeftMotorSpeed as ULONG,
        preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!("HidP_SetUsageValue INTENSITY returned %#lx\n", status);
    }

    collection = (*controller).hid.haptics_buzz_caps.LinkCollection;
    status = HidP_SetUsageValue(
        HidP_Output,
        HID_USAGE_PAGE_HAPTICS,
        collection,
        HID_USAGE_HAPTICS_INTENSITY,
        (*state).wRightMotorSpeed as ULONG,
        preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!("HidP_SetUsageValue INTENSITY returned %#lx\n", status);
    }

    ret = HidD_SetOutputReport(
        (*controller).device,
        report_buf as *mut c_void,
        report_len,
    ) as BOOL;
    if ret == 0 {
        WARN!("HidD_SetOutputReport failed with error %lu\n", GetLastError());
    }

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn controller_enable(mut controller: *mut xinput_controller) -> BOOL {
    let mut report_len: ULONG = (*controller).hid.caps.InputReportByteLength as ULONG;
    let mut report_buf: *mut ::core::ffi::c_char = (*controller).hid.input_report_buf;
    let mut state: XINPUT_VIBRATION = (*controller).vibration;
    let mut ret: BOOL = 0;
    if (*controller).enabled != 0 {
        return (TRUE) as BOOL;
    }
    if (*controller).caps.Flags as ::core::ffi::c_int & (XINPUT_CAPS_FFB_SUPPORTED as i32) != 0 {
        HID_set_state(controller, &raw mut state);
    }
    memset(
        &raw mut (*controller).hid.read_ovl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<OVERLAPPED>() as size_t,
    );
    (*controller).hid.read_ovl.hEvent = (*controller).read_event;
    ret = ReadFile(
        (*controller).device,
        report_buf as PVOID,
        report_len as DWORD,
        ::core::ptr::null_mut::<DWORD>(),
        &raw mut (*controller).hid.read_ovl,
    );
    if ret == 0 && GetLastError() != ERROR_IO_PENDING as DWORD {
        return (FALSE) as BOOL;
    }
    (*controller).enabled = TRUE as BOOL;
    SetEvent(update_event);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn controller_disable(controller: *mut xinput_controller) {
    let mut state: XINPUT_VIBRATION = core::mem::zeroed();

    if (*controller).enabled == 0 {
        return;
    }
    if ((*controller).caps.Flags as u32 & XINPUT_CAPS_FFB_SUPPORTED) != 0 {
        HID_set_state(controller, &raw mut state);
    }
    (*controller).enabled = (FALSE as i32);

    if CancelIoEx((*controller).device, &raw mut (*controller).hid.read_ovl) != 0 {
        WaitForSingleObject((*controller).hid.read_ovl.hEvent, INFINITE);
    }
    SetEvent(update_event);
}

unsafe extern "C" fn controller_init(
    mut controller: *mut xinput_controller,
    mut preparsed: PHIDP_PREPARSED_DATA,
    mut caps: *mut HIDP_CAPS,
    mut device: HANDLE,
    mut device_path: *const WCHAR,
) -> BOOL {
    let mut state: XINPUT_STATE = core::mem::zeroed();

    'failed: {
        (*controller).hid.caps = *caps;
        (*controller).hid.feature_report_buf = calloc(
            1,
            ((*controller).hid.caps.FeatureReportByteLength as size_t) as _,
        ) as *mut ::core::ffi::c_char;
        if (*controller).hid.feature_report_buf.is_null() {
            break 'failed;
        }
        if controller_check_caps(controller, device, preparsed) == 0 {
            break 'failed;
        }

        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/xinput1_3/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"controller_init\0" as *const u8 as *const ::core::ffi::c_char,
                371 as ::core::ffi::c_int,
                b"Found gamepad %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(device_path),
            );
        }

        (*controller).hid.preparsed = preparsed;
        (*controller).hid.input_report_buf = calloc(
            1,
            ((*controller).hid.caps.InputReportByteLength as size_t) as _,
        ) as *mut ::core::ffi::c_char;
        if (*controller).hid.input_report_buf.is_null() {
            break 'failed;
        }
        (*controller).hid.output_report_buf = calloc(
            1,
            ((*controller).hid.caps.OutputReportByteLength as size_t) as _,
        ) as *mut ::core::ffi::c_char;
        if (*controller).hid.output_report_buf.is_null() {
            break 'failed;
        }

        memset(
            core::ptr::addr_of_mut!((*controller).vibration) as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<XINPUT_VIBRATION>() as size_t,
        );
        lstrcpynW(
            core::ptr::addr_of_mut!((*controller).device_path) as LPWSTR,
            device_path as LPCWSTR,
            (MAX_PATH as i32),
        );
        (*controller).enabled = (FALSE as i32);

        (*controller).device = device;
        if controller_enable(controller) == 0 {
            break 'failed;
        }

        set_current_state(
            controller.offset_from(core::ptr::addr_of_mut!(controllers) as *mut xinput_controller)
                as ::core::ffi::c_longlong as UINT,
            core::ptr::addr_of_mut!(state),
        );
        return (TRUE) as BOOL;
    }

    // failed:
    free((*controller).hid.input_report_buf as *mut ::core::ffi::c_void);
    free((*controller).hid.output_report_buf as *mut ::core::ffi::c_void);
    free((*controller).hid.feature_report_buf as *mut ::core::ffi::c_void);
    memset(
        core::ptr::addr_of_mut!((*controller).hid) as *mut ::core::ffi::c_void,
        0,
        core::mem::size_of::<xinput_controller_hid>() as size_t,
    );
    (*controller).device = core::ptr::null_mut();
    return (FALSE) as BOOL;
}

unsafe extern "C" fn get_registry_keys(defkey: *mut HKEY, appkey: *mut HKEY) {
    let mut buffer: [WCHAR; (MAX_PATH + 26) as usize] = [0; (MAX_PATH + 26) as usize];
    let mut name: *mut WCHAR = buffer.as_mut_ptr();
    let mut tmp: *mut WCHAR;
    let mut len: DWORD;
    let mut hkey: HKEY = core::mem::zeroed();

    *appkey = core::ptr::null_mut();
    if RegOpenKeyW(HKEY_CURRENT_USER, RUSTCONV_WSTR_F1850E60EE4D.as_ptr(), defkey) != 0 {
        *defkey = core::ptr::null_mut();
    }

    len = GetModuleFileNameW(core::ptr::null_mut(), buffer.as_mut_ptr(), MAX_PATH);
    if len == 0 || len >= MAX_PATH {
        return;
    }

    if RegOpenKeyW(HKEY_CURRENT_USER, RUSTCONV_WSTR_7B5EDCF11682.as_ptr(), core::ptr::addr_of_mut!(hkey)) == 0 {
        tmp = wcsrchr(name, '/' as u16 as wchar_t) as *mut WCHAR;
        if !tmp.is_null() {
            name = tmp.offset(1 as isize);
        }
        tmp = wcsrchr(name, '\\' as u16 as wchar_t) as *mut WCHAR;
        if !tmp.is_null() {
            name = tmp.offset(1 as isize);
        }
        wcscat(name, RUSTCONV_WSTR_0A6804564E14.as_ptr());
        if RegOpenKeyW(hkey, name, appkey) != 0 {
            *appkey = core::ptr::null_mut();
        }
        RegCloseKey(hkey);
    }
}

unsafe extern "C" fn device_is_overridden(device: HANDLE) -> BOOL {
    let mut name: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut size: DWORD = ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    let mut disable: BOOL = (FALSE as i32);
    let mut defkey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut appkey: HKEY = ::core::ptr::null_mut::<HKEY__>();

    if HidD_GetProductString(
        device,
        &raw mut name as *mut WCHAR as PVOID,
        MAX_PATH as ULONG,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    get_registry_keys(&raw mut defkey, &raw mut appkey);
    if defkey.is_null() && appkey.is_null() {
        return (FALSE) as BOOL;
    }
    if !appkey.is_null()
        && RegQueryValueExW(
            appkey,
            &raw mut name as *mut WCHAR as LPCWSTR,
            core::ptr::null_mut(),
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut buffer as *mut WCHAR as LPBYTE,
            &raw mut size,
        ) == 0
        || !defkey.is_null()
            && RegQueryValueExW(
                defkey,
                &raw mut name as *mut WCHAR as LPCWSTR,
                core::ptr::null_mut(),
                ::core::ptr::null_mut::<DWORD>(),
                &raw mut buffer as *mut WCHAR as LPBYTE,
                &raw mut size,
            ) == 0
    {
        disable = (wcscmp(
            &raw mut buffer as *mut WCHAR,
            RUSTCONV_WSTR_AD1FFBB38714.as_ptr(),
        ) == 0
            || wcscmp(
                &raw mut buffer as *mut WCHAR,
                RUSTCONV_WSTR_52B0C056F710.as_ptr(),
            ) == 0) as ::core::ffi::c_int as BOOL;
        if disable != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/xinput1_3/main.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"device_is_overridden\0" as *const u8 as *const ::core::ffi::c_char,
                    434 as ::core::ffi::c_int,
                    b"Disabling gamepad '%s' based on registry key.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut name as *mut WCHAR),
                );
            }
        }
    }

    if !appkey.is_null() {
        RegCloseKey(appkey);
    }
    if !defkey.is_null() {
        RegCloseKey(defkey);
    }
    return disable;
}

unsafe extern "C" fn open_device_at_index(device_path: *const WCHAR, index: core::ffi::c_int) -> BOOL {
    let mut preparsed: PHIDP_PREPARSED_DATA = core::ptr::null_mut();
    let mut caps: HIDP_CAPS = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut device: HANDLE;

    device = CreateFileW(device_path, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE,
                         core::ptr::null_mut(), OPEN_EXISTING, FILE_FLAG_OVERLAPPED | FILE_FLAG_NO_BUFFERING, core::ptr::null_mut());
    if device == INVALID_HANDLE_VALUE {
        return (TRUE) as BOOL;
    }

    preparsed = core::ptr::null_mut();
    if HidD_GetPreparsedData(device, core::ptr::addr_of_mut!(preparsed)) == 0 {
        WARN!("ignoring HID device, HidD_GetPreparsedData failed with error %lu\n", GetLastError());
    } else {
        status = HidP_GetCaps(preparsed, core::ptr::addr_of_mut!(caps));
        if status != HIDP_STATUS_SUCCESS {
            WARN!("ignoring HID device, HidP_GetCaps returned %#lx\n", status);
        } else if caps.UsagePage != HID_USAGE_PAGE_GENERIC {
            WARN!("ignoring HID device, unsupported usage page %04x\n", caps.UsagePage as core::ffi::c_uint);
        } else if caps.Usage != HID_USAGE_GENERIC_GAMEPAD && caps.Usage != HID_USAGE_GENERIC_JOYSTICK &&
                  caps.Usage != HID_USAGE_GENERIC_MULTI_AXIS_CONTROLLER {
            WARN!("ignoring HID device, unsupported usage %04x:%04x\n", caps.UsagePage as core::ffi::c_uint, caps.Usage as core::ffi::c_uint);
        } else if device_is_overridden(device) != 0 {
            WARN!("ignoring HID device, overridden for dinput or disabled\n");
        } else if controller_init(
            (core::ptr::addr_of_mut!(controllers) as *mut xinput_controller).offset(index as isize),
            preparsed,
            core::ptr::addr_of_mut!(caps),
            device,
            device_path,
        ) == 0 {
            WARN!("ignoring HID device, failed to initialize\n");
        } else {
            TRACE!("opened device %s at index %u\n", debugstr_w(device_path), index);
            return (TRUE) as BOOL;
        }
    }

    CloseHandle(device);
    HidD_FreePreparsedData(preparsed);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn find_opened_device(device_path: *const WCHAR, slot: *mut ::core::ffi::c_int) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    *slot = (XUSER_MAX_COUNT as i32);
    i = (XUSER_MAX_COUNT as i32);
    while i > 0 {
        if controllers[(i - 1) as usize].device.is_null() {
            *slot = i - 1;
        } else if wcsicmp(
            device_path as *const wchar_t,
            addr_of_mut!(controllers[(i - 1) as usize].device_path) as *mut WCHAR,
        ) == 0
        {
            *slot = i - 1;
            return (TRUE) as BOOL;
        }
        i -= 1;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn try_add_device(device_path: *const WCHAR) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    if find_opened_device(device_path, core::ptr::addr_of_mut!(i)) != 0 {
        return (TRUE) as BOOL;
    }
    if i == (XUSER_MAX_COUNT as i32) {
        return (FALSE) as BOOL;
    }
    return open_device_at_index(device_path, i);
}

unsafe extern "C" fn try_remove_device(device_path: *const WCHAR) {
    let mut i: core::ffi::c_int = 0;
    if find_opened_device(device_path, core::ptr::addr_of_mut!(i)) != 0 {
        controller_destroy((core::ptr::addr_of_mut!(controllers) as *mut xinput_controller).offset(i as isize), (FALSE) as BOOL);
    }
}

unsafe extern "C" fn update_controller_list() {
    let mut buffer: [u64; (core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() + MAX_PATH as usize * core::mem::size_of::<WCHAR>() + 7) / 8] = core::mem::zeroed();
    let mut detail: *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W = buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
    let mut iface: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed();
    iface.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
    let mut guid: GUID = GUID_DEVINTERFACE_WINEXINPUT;
    let mut set: HDEVINFO = core::ptr::null_mut();
    let mut idx: DWORD;

    EnterCriticalSection(core::ptr::addr_of_mut!(xinput_cs));

    set = SetupDiGetClassDevsW(
        core::ptr::addr_of_mut!(guid),
        core::ptr::null(),
        core::ptr::null_mut(),
        (DIGCF_DEVICEINTERFACE | DIGCF_PRESENT) as DWORD,
    );
    (*detail).cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;

    idx = 0;
    loop { // nolint: dowhile-continue (C is `while (SetupDiEnumDeviceInterfaces(...))`, tested at the top as here; the trailing break is its `if (!try_add_device()) break;`)
        let fresh = idx;
        idx = idx.wrapping_add(1);
        if !(SetupDiEnumDeviceInterfaces(
            set,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(guid),
            fresh,
            core::ptr::addr_of_mut!(iface),
        ) != 0)
        {
            break;
        }
        if SetupDiGetDeviceInterfaceDetailW(
            set,
            core::ptr::addr_of_mut!(iface),
            detail,
            core::mem::size_of_val(&buffer) as DWORD,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            continue;
        }
        if try_add_device(core::ptr::addr_of_mut!((*detail).DevicePath) as *mut WCHAR) == 0 {
            break;
        }
    }

    SetupDiDestroyDeviceInfoList(set);

    LeaveCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
}

unsafe extern "C" fn controller_destroy(controller: *mut xinput_controller, already_removed: BOOL) {
    set_current_state(
        controller.offset_from(core::ptr::addr_of_mut!(controllers) as *mut xinput_controller) as UINT,
        core::ptr::null::<XINPUT_STATE>(),
    );

    if !(*controller).device.is_null() {
        if already_removed != 0 {
            (*controller).enabled = (FALSE as i32);
        } else {
            controller_disable(controller);
        }

        CloseHandle((*controller).device);
        (*controller).device = core::ptr::null_mut();

        free((*controller).hid.input_report_buf as *mut core::ffi::c_void);
        free((*controller).hid.output_report_buf as *mut core::ffi::c_void);
        free((*controller).hid.feature_report_buf as *mut core::ffi::c_void);
        HidD_FreePreparsedData((*controller).hid.preparsed);
        memset(
            core::ptr::addr_of_mut!((*controller).hid) as *mut core::ffi::c_void,
            0,
            core::mem::size_of_val(&(*controller).hid),
        );
    }
}

unsafe extern "C" fn sign_extend(mut value: ULONG, mut caps: *const HIDP_VALUE_CAPS) -> LONG {
    let mut sign: UINT = ((1 as ::core::ffi::c_int)
        << ((*caps).BitSize as ::core::ffi::c_int - 1 as ::core::ffi::c_int))
        as UINT;
    if sign <= 1 as UINT || (*caps).LogicalMin >= 0 as LONG {
        return value as LONG;
    }
    return value.wrapping_sub((value & sign as ULONG) << 1 as ::core::ffi::c_int) as LONG;
}

unsafe extern "C" fn scale_value(value: ULONG, caps: *const HIDP_VALUE_CAPS, min: LONG, max: LONG) -> LONG {
    let mut tmp: LONG = sign_extend(value, caps);
    if (*caps).LogicalMin > (*caps).LogicalMax {
        return 0;
    }
    if (*caps).LogicalMin > tmp || (*caps).LogicalMax < tmp {
        return 0;
    }
    return min + MulDiv(
        tmp - (*caps).LogicalMin,
        max - min,
        (*caps).LogicalMax - (*caps).LogicalMin,
    );
}

unsafe extern "C" fn read_controller_state(controller: *mut xinput_controller) {
    let mut read_len: ULONG = 0;
    let mut report_len: ULONG = (*controller).hid.caps.InputReportByteLength as ULONG;
    let mut report_buf: *mut core::ffi::c_char = (*controller).hid.input_report_buf;
    let mut state: XINPUT_STATE = core::mem::zeroed();
    let mut status: NTSTATUS = 0;
    let mut buttons: [USAGE; 11] = [0; 11];
    let mut i: ULONG = 0;
    let mut button_length: ULONG = 0;
    let mut value: ULONG = 0;
    let mut ret: BOOL = 0;

    if GetOverlappedResult(
        (*controller).device,
        core::ptr::addr_of_mut!((*controller).hid.read_ovl),
        core::ptr::addr_of_mut!(read_len),
        (TRUE as i32),
    ) == 0
    {
        if GetLastError() == ERROR_OPERATION_ABORTED {
            return;
        }
        if GetLastError() == ERROR_ACCESS_DENIED
            || GetLastError() == ERROR_INVALID_HANDLE
            || GetLastError() == ERROR_DEVICE_NOT_CONNECTED
        {
            controller_destroy(controller, (TRUE) as BOOL);
        } else {
            ERR!(
                "Failed to read input report, GetOverlappedResult failed with error %lu\n",
                GetLastError()
            );
        }
        return;
    }

    button_length = ARRAY_SIZE!(buttons) as ULONG;
    status = HidP_GetUsages(
        HidP_Input,
        HID_USAGE_PAGE_BUTTON,
        0,
        buttons.as_mut_ptr(),
        core::ptr::addr_of_mut!(button_length),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsages HID_USAGE_PAGE_BUTTON returned %#lx\n",
            status
        );
    }

    get_current_state(
        controller.offset_from(core::ptr::addr_of_mut!(controllers) as *mut xinput_controller)
            as core::ffi::c_longlong as UINT,
        core::ptr::addr_of_mut!(state),
    );

    state.Gamepad.wButtons = 0;
    i = 0;
    while i < button_length {
        match buttons[i as usize] {
            1 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_A as u16);
            }
            2 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_B as u16);
            }
            3 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_X as u16);
            }
            4 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_Y as u16);
            }
            5 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_LEFT_SHOULDER as u16);
            }
            6 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_RIGHT_SHOULDER as u16);
            }
            7 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_BACK as u16);
            }
            8 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_START as u16);
            }
            9 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_LEFT_THUMB as u16);
            }
            10 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_RIGHT_THUMB as u16);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }

    button_length = ARRAY_SIZE!(buttons) as ULONG;
    status = HidP_GetUsages(
        HidP_Input,
        HID_USAGE_PAGE_VENDOR_DEFINED_BEGIN,
        0,
        buttons.as_mut_ptr(),
        core::ptr::addr_of_mut!(button_length),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsages HID_USAGE_PAGE_VENDOR_DEFINED_BEGIN returned %#lx\n",
            status
        );
    }
    if button_length != 0 {
        state.Gamepad.wButtons |= (XINPUT_GAMEPAD_GUIDE as u16);
    }

    status = HidP_GetUsageValue(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        0,
        HID_USAGE_GENERIC_HATSWITCH,
        core::ptr::addr_of_mut!(value),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsageValue HID_USAGE_PAGE_GENERIC / HID_USAGE_GENERIC_HATSWITCH returned %#lx\n",
            status
        );
    } else {
        match value {
            /* 8 1 2
             * 7 0 3
             * 6 5 4 */
            0 => {}
            1 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_UP as u16);
            }
            2 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_UP | XINPUT_GAMEPAD_DPAD_RIGHT) as u16;
            }
            3 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_RIGHT as u16);
            }
            4 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_RIGHT | XINPUT_GAMEPAD_DPAD_DOWN) as u16;
            }
            5 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_DOWN as u16);
            }
            6 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_DOWN | XINPUT_GAMEPAD_DPAD_LEFT) as u16;
            }
            7 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_LEFT as u16);
            }
            8 => {
                state.Gamepad.wButtons |= (XINPUT_GAMEPAD_DPAD_LEFT | XINPUT_GAMEPAD_DPAD_UP) as u16;
            }
            _ => {}
        }
    }

    status = HidP_GetUsageValue(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        0,
        HID_USAGE_GENERIC_X,
        core::ptr::addr_of_mut!(value),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsageValue HID_USAGE_PAGE_GENERIC / HID_USAGE_GENERIC_X returned %#lx\n",
            status
        );
    } else {
        state.Gamepad.sThumbLX = scale_value(
            value,
            core::ptr::addr_of_mut!((*controller).hid.lx_caps),
            -32768,
            32767,
        ) as SHORT;
    }

    status = HidP_GetUsageValue(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        0,
        HID_USAGE_GENERIC_Y,
        core::ptr::addr_of_mut!(value),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsageValue HID_USAGE_PAGE_GENERIC / HID_USAGE_GENERIC_Y returned %#lx\n",
            status
        );
    } else {
        state.Gamepad.sThumbLY = scale_value(
            value,
            core::ptr::addr_of_mut!((*controller).hid.ly_caps),
            -32768,
            32767,
        ) as SHORT;
    }

    status = HidP_GetUsageValue(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        0,
        HID_USAGE_GENERIC_RX,
        core::ptr::addr_of_mut!(value),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsageValue HID_USAGE_PAGE_GENERIC / HID_USAGE_GENERIC_RX returned %#lx\n",
            status
        );
    } else {
        state.Gamepad.sThumbRX = scale_value(
            value,
            core::ptr::addr_of_mut!((*controller).hid.rx_caps),
            -32768,
            32767,
        ) as SHORT;
    }

    status = HidP_GetUsageValue(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        0,
        HID_USAGE_GENERIC_RY,
        core::ptr::addr_of_mut!(value),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsageValue HID_USAGE_PAGE_GENERIC / HID_USAGE_GENERIC_RY returned %#lx\n",
            status
        );
    } else {
        state.Gamepad.sThumbRY = scale_value(
            value,
            core::ptr::addr_of_mut!((*controller).hid.ry_caps),
            -32768,
            32767,
        ) as SHORT;
    }

    status = HidP_GetUsageValue(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        0,
        HID_USAGE_GENERIC_RZ,
        core::ptr::addr_of_mut!(value),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsageValue HID_USAGE_PAGE_GENERIC / HID_USAGE_GENERIC_RZ returned %#lx\n",
            status
        );
    } else {
        state.Gamepad.bRightTrigger = scale_value(
            value,
            core::ptr::addr_of_mut!((*controller).hid.rt_caps),
            0,
            255,
        ) as BYTE;
    }

    status = HidP_GetUsageValue(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        0,
        HID_USAGE_GENERIC_Z,
        core::ptr::addr_of_mut!(value),
        (*controller).hid.preparsed,
        report_buf as PCHAR,
        report_len,
    );
    if status != HIDP_STATUS_SUCCESS {
        WARN!(
            "HidP_GetUsageValue HID_USAGE_PAGE_GENERIC / HID_USAGE_GENERIC_Z returned %#lx\n",
            status
        );
    } else {
        state.Gamepad.bLeftTrigger = scale_value(
            value,
            core::ptr::addr_of_mut!((*controller).hid.lt_caps),
            0,
            255,
        ) as BYTE;
    }

    if (*controller).enabled != 0 {
        set_current_state(
            controller.offset_from(core::ptr::addr_of_mut!(controllers) as *mut xinput_controller)
                as core::ffi::c_longlong as UINT,
            core::ptr::addr_of_mut!(state),
        );
        memset(
            core::ptr::addr_of_mut!((*controller).hid.read_ovl) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<OVERLAPPED>() as size_t,
        );
        (*controller).hid.read_ovl.hEvent = (*controller).read_event;
        ret = ReadFile(
            (*controller).device,
            report_buf as PVOID,
            report_len as DWORD,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!((*controller).hid.read_ovl),
        );
        if ret == 0 && GetLastError() != ERROR_IO_PENDING {
            controller_destroy(controller, (TRUE) as BOOL);
        }
    }
}

unsafe extern "system" fn xinput_devnotify_wndproc(
    mut hwnd: HWND,
    mut msg: UINT,
    mut wparam: WPARAM,
    mut lparam: LPARAM,
) -> LRESULT {
    if msg == WM_DEVICECHANGE as UINT {
        let mut iface: *mut DEV_BROADCAST_DEVICEINTERFACE_W =
            lparam as *mut DEV_BROADCAST_DEVICEINTERFACE_W;

        EnterCriticalSection(&raw mut xinput_cs);
        if wparam == DBT_DEVICEARRIVAL as WPARAM {
            try_add_device(&raw mut (*iface).dbcc_name as *mut WCHAR);
        }
        if wparam == DBT_DEVICEREMOVECOMPLETE as WPARAM {
            try_remove_device(&raw mut (*iface).dbcc_name as *mut WCHAR);
        }
        LeaveCriticalSection(&raw mut xinput_cs);
    }

    return DefWindowProcW(hwnd, msg, wparam, lparam);
}

unsafe extern "system" fn hid_update_thread_proc(param: *mut core::ffi::c_void) -> DWORD {
    let mut devices: [*mut xinput_controller; (XUSER_MAX_COUNT + 1) as _] =
        [core::ptr::null_mut::<xinput_controller>(); (XUSER_MAX_COUNT + 1) as _];
    let mut events: [HANDLE; (XUSER_MAX_COUNT + 1) as _] =
        [core::ptr::null_mut::<core::ffi::c_void>(); (XUSER_MAX_COUNT + 1) as _];
    let mut i: DWORD = 0;
    let mut count: DWORD = 1;
    let mut ret: DWORD = WAIT_TIMEOUT;
    let mut filter: DEV_BROADCAST_DEVICEINTERFACE_W = DEV_BROADCAST_DEVICEINTERFACE_W {
        dbcc_size: core::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as DWORD,
        dbcc_devicetype: DBT_DEVTYP_DEVICEINTERFACE as DWORD,
        dbcc_classguid: GUID_DEVINTERFACE_WINEXINPUT,
        ..core::mem::zeroed()
    };
    let mut cls: WNDCLASSEXW = WNDCLASSEXW {
        cbSize: core::mem::size_of::<WNDCLASSEXW>() as UINT,
        hInstance: xinput_instance as HINSTANCE,
        lpszClassName: RUSTCONV_WSTR_FEB45BBE340C.as_ptr(),
        lpfnWndProc: Some(
            xinput_devnotify_wndproc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
        ),
        ..core::mem::zeroed()
    };
    let mut notif: HDEVNOTIFY = core::ptr::null_mut();
    let mut hwnd: HWND = core::ptr::null_mut();
    let mut msg: MSG = core::mem::zeroed();

    SetThreadDescription(GetCurrentThread(), RUSTCONV_WSTR_C4B1D9B41A0C.as_ptr());

    RegisterClassExW(core::ptr::addr_of_mut!(cls));
    hwnd = CreateWindowExW(
        0,
        cls.lpszClassName,
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        0,
        HWND_MESSAGE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    notif = RegisterDeviceNotificationW(
        (hwnd).cast(),
        (core::ptr::addr_of_mut!(filter)).cast(),
        DEVICE_NOTIFY_WINDOW_HANDLE,
    );

    update_controller_list();
    SetEvent(start_event);

    loop {
        if ret == count {
            while PeekMessageW(
                core::ptr::addr_of_mut!(msg),
                hwnd,
                0,
                0,
                PM_REMOVE,
            ) != 0
            {
                DispatchMessageW(core::ptr::addr_of_mut!(msg));
            }
        }
        if ret == WAIT_TIMEOUT {
            update_controller_list();
        }

        EnterCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
        if ret < count.wrapping_sub(1)
            && !(*devices[ret as usize]).device.is_null()
        {
            read_controller_state(devices[ret as usize]);
        }
        count = 0;
        i = 0;
        while i < XUSER_MAX_COUNT {
            if !controllers[i as usize].device.is_null() && controllers[i as usize].enabled != 0 {
                devices[count as usize] =
                    (core::ptr::addr_of_mut!(controllers) as *mut xinput_controller).add(i as usize);
                events[count as usize] = controllers[i as usize].read_event;
                count = count.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        let fresh15 = count;
        count = count.wrapping_add(1);
        events[fresh15 as usize] = update_event;
        LeaveCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
        loop {
            ret = MsgWaitForMultipleObjectsEx(
                count,
                events.as_mut_ptr(),
                2000,
                QS_ALLINPUT,
                MWMO_ALERTABLE,
            );
            if !(ret <= count || ret == WAIT_TIMEOUT) {
                break;
            }
            break;
        }
        if !(ret <= count || ret == WAIT_TIMEOUT) {
            break;
        }
    }

    ERR!(
        "wait failed in the update thread, ret %lu, error %lu\n",
        ret,
        GetLastError()
    );

    UnregisterDeviceNotification(notif);
    DestroyWindow(hwnd);
    UnregisterClassW(cls.lpszClassName, xinput_instance);

    FreeLibraryAndExitThread(xinput_instance, ret);
}
unsafe extern "system" fn start_update_thread_once(
    once: *mut INIT_ONCE,
    param: *mut core::ffi::c_void,
    context: *mut *mut core::ffi::c_void,
) -> BOOL {
    let mut thread: HANDLE = core::ptr::null_mut();
    let mut module: HMODULE = core::ptr::null_mut();
    let mut i: i32;

    if GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, hid_update_thread_proc as usize as LPCWSTR, &mut module) == 0 {
        WARN!("Failed to increase module's reference count, error: %lu\n", GetLastError());
    }

    start_event = CreateEventA(core::ptr::null_mut(), (FALSE) as BOOL, (FALSE) as BOOL, core::ptr::null());
    if start_event.is_null() {
        ERR!("failed to create start event, error %lu\n", GetLastError());
    }

    update_event = CreateEventA(core::ptr::null_mut(), (FALSE) as BOOL, (FALSE) as BOOL, core::ptr::null());
    if update_event.is_null() {
        ERR!("failed to create update event, error %lu\n", GetLastError());
    }

    i = 0;
    while i < (XUSER_MAX_COUNT as i32) {
        controllers[i as usize].read_event = CreateEventA(core::ptr::null_mut(), (TRUE) as BOOL, (FALSE) as BOOL, core::ptr::null());
        if controllers[i as usize].read_event.is_null() {
            ERR!("failed to create read event for controller %d, error %lu\n", i, GetLastError());
        }
        i += 1;
    }

    thread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(hid_update_thread_proc),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if thread.is_null() {
        ERR!("failed to create update thread, error %lu\n", GetLastError());
    }
    CloseHandle(thread);

    WaitForSingleObject(start_event, INFINITE);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn start_update_thread() {
    static mut init_once: INIT_ONCE = unsafe { core::mem::zeroed() }; // INIT_ONCE_STATIC_INIT is {0}
    InitOnceExecuteOnce(
        core::ptr::addr_of_mut!(init_once),
        Some(
            start_update_thread_once,
        ),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(inst: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            xinput_instance = inst as HMODULE;
            DisableThreadLibraryCalls(inst as HMODULE);
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn XInputEnable(enable: BOOL) {
    let mut index: i32;

    start_update_thread();

    EnterCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
    index = 0;
    while index < (XUSER_MAX_COUNT as i32) {
        let c: *mut xinput_controller =
            (core::ptr::addr_of_mut!(controllers) as *mut xinput_controller).offset(index as isize);
        if !(*c).device.is_null() {
            if enable == 0 {
                controller_disable(c);
            } else if controller_enable(c) == 0 {
                controller_destroy(c, (TRUE) as BOOL);
            }
        }
        index += 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
}

#[no_mangle]
pub unsafe extern "system" fn XInputSetState(index: DWORD, vibration: *mut XINPUT_VIBRATION) -> DWORD {
    let mut ret: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/xinput1_3/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"XInputSetState\0" as *const u8 as *const ::core::ffi::c_char,
            836 as ::core::ffi::c_int,
            b"index %lu, vibration %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            index,
            vibration,
        );
    }
    start_update_thread();
    if index >= XUSER_MAX_COUNT {
        return ERROR_BAD_ARGUMENTS;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
    if (*core::ptr::addr_of_mut!(controllers))[index as usize].device.is_null() {
        update_controller_list();
    }
    if (*core::ptr::addr_of_mut!(controllers))[index as usize].device.is_null() {
        ret = ERROR_DEVICE_NOT_CONNECTED;
    } else {
        ret = HID_set_state(
            (core::ptr::addr_of_mut!(controllers) as *mut xinput_controller).offset(index as isize),
            vibration,
        );
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
    return ret;
}

unsafe extern "C" fn xinput_get_state(mut index: DWORD, mut state: *mut XINPUT_STATE) -> DWORD {
    let mut ret: DWORD;

    if state.is_null() {
        return ERROR_BAD_ARGUMENTS;
    }

    start_update_thread();

    if index >= XUSER_MAX_COUNT {
        return ERROR_BAD_ARGUMENTS;
    }
    if get_current_state(index, state) != 0 {
        return ERROR_SUCCESS;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
    update_controller_list();
    ret = if get_current_state(index, state) != 0 {
        ERROR_SUCCESS
    } else {
        ERROR_DEVICE_NOT_CONNECTED
    };
    LeaveCriticalSection(core::ptr::addr_of_mut!(xinput_cs));

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetState(index: DWORD, state: *mut XINPUT_STATE) -> DWORD {
    let mut ret: DWORD = 0;

    ret = xinput_get_state(index, state);
    if ret != ERROR_SUCCESS {
        return ret;
    }

    /* The main difference between this and the Ex version is the media guide button */
    (*state).Gamepad.wButtons &= !(XINPUT_GAMEPAD_GUIDE as WORD);

    return ERROR_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn XInputGetStateEx(
    index: DWORD,
    state: *mut XINPUT_STATE,
) -> DWORD {
    // TRACE("index %lu, state %p.\n", index, state);
    xinput_get_state(index, state)
}

unsafe extern "C" fn joystick_state(value: SHORT) -> i32 {
    if value as i32 > 20000 {
        return JS_STATE_HIGH;
    }
    if (value as i32) < -(20000) {
        return JS_STATE_LOW;
    }
    return JS_STATE_OFF;
}

unsafe extern "C" fn js_vk_offs(x: ::core::ffi::c_int, y: ::core::ffi::c_int) -> WORD {
    if y == JS_STATE_OFF {
        /*if (x == JS_STATE_OFF) shouldn't get here */
        if x == JS_STATE_LOW {
            return 3; /* LEFT */
        }
        /*if (x == JS_STATE_HIGH)*/
        return 2; /* RIGHT */
    }
    if y == JS_STATE_HIGH {
        if x == JS_STATE_OFF {
            return 0; /* UP */
        }
        if x == JS_STATE_LOW {
            return 4; /* UPLEFT */
        }
        /*if (x == JS_STATE_HIGH)*/
        return 5; /* UPRIGHT */
    }
    /*if (y == JS_STATE_LOW)*/
    if x == JS_STATE_OFF {
        return 1; /* DOWN */
    }
    if x == JS_STATE_LOW {
        return 7; /* DOWNLEFT */
    }
    /*if (x == JS_STATE_HIGH)*/
    return 6; /* DOWNRIGHT */
}

unsafe extern "C" fn check_joystick_keystroke(
    index: DWORD,
    keystroke: *mut XINPUT_KEYSTROKE,
    cur_x: *const SHORT,
    cur_y: *const SHORT,
    last_x: *mut SHORT,
    last_y: *mut SHORT,
    base_vk: WORD,
) -> DWORD {
    let mut cur_vk: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur_x_st: ::core::ffi::c_int;
    let mut cur_y_st: ::core::ffi::c_int;
    let mut last_vk: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut last_x_st: ::core::ffi::c_int;
    let mut last_y_st: ::core::ffi::c_int;

    cur_x_st = joystick_state(*cur_x) as ::core::ffi::c_int;
    cur_y_st = joystick_state(*cur_y) as ::core::ffi::c_int;
    if cur_x_st != 0 || cur_y_st != 0 {
        cur_vk = base_vk as ::core::ffi::c_int
            + js_vk_offs(cur_x_st, cur_y_st) as ::core::ffi::c_int;
    }

    last_x_st = joystick_state(*last_x) as ::core::ffi::c_int;
    last_y_st = joystick_state(*last_y) as ::core::ffi::c_int;
    if last_x_st != 0 || last_y_st != 0 {
        last_vk = base_vk as ::core::ffi::c_int
            + js_vk_offs(last_x_st, last_y_st) as ::core::ffi::c_int;
    }

    if cur_vk != last_vk {
        if last_vk != 0 {
            /* joystick was set, and now different. send a KEYUP event, and set
             * last pos to centered, so the appropriate KEYDOWN event will be
             * sent on the next call. */
            (*keystroke).VirtualKey = last_vk as WORD;
            (*keystroke).Unicode = 0 as WCHAR; /* unused */
            (*keystroke).Flags = XINPUT_KEYSTROKE_KEYUP as WORD;
            (*keystroke).UserIndex = index as BYTE;
            (*keystroke).HidCode = 0 as BYTE;

            *last_x = 0 as SHORT;
            *last_y = 0 as SHORT;

            return ERROR_SUCCESS as DWORD;
        }

        /* joystick was unset, send KEYDOWN. */
        (*keystroke).VirtualKey = cur_vk as WORD;
        (*keystroke).Unicode = 0 as WCHAR; /* unused */
        (*keystroke).Flags = XINPUT_KEYSTROKE_KEYDOWN as WORD;
        (*keystroke).UserIndex = index as BYTE;
        (*keystroke).HidCode = 0 as BYTE;

        *last_x = *cur_x;
        *last_y = *cur_y;

        return ERROR_SUCCESS as DWORD;
    }

    *last_x = *cur_x;
    *last_y = *cur_y;

    return ERROR_EMPTY as DWORD;
}

unsafe extern "C" fn trigger_is_on(value: BYTE) -> BOOL {
    return (value > 30) as BOOL;
}

unsafe extern "C" fn check_for_keystroke(index: DWORD, keystroke: *mut XINPUT_KEYSTROKE) -> DWORD {
    static mut last_state: [XINPUT_GAMEPAD; XUSER_MAX_COUNT as usize] = [XINPUT_GAMEPAD {
        wButtons: 0,
        bLeftTrigger: 0,
        bRightTrigger: 0,
        sThumbLX: 0,
        sThumbLY: 0,
        sThumbRX: 0,
        sThumbRY: 0,
    }; XUSER_MAX_COUNT as usize];
    static mut keystroke_lock: SRWLOCK = unsafe { core::mem::zeroed() }; // SRWLOCK_INIT is {0}

    let mut last: *mut XINPUT_GAMEPAD =
        (core::ptr::addr_of_mut!(last_state) as *mut XINPUT_GAMEPAD).add(index as usize);
    let mut cur: *const XINPUT_GAMEPAD = core::ptr::null::<XINPUT_GAMEPAD>();
    let mut ret: DWORD = ERROR_EMPTY as DWORD;
    let mut state: XINPUT_STATE = XINPUT_STATE {
        dwPacketNumber: 0,
        Gamepad: XINPUT_GAMEPAD {
            wButtons: 0,
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        },
    };
    let mut i: i32;

    // (mask, vk) pairs
    static buttons: [(i32, WORD); 14] = [
        (XINPUT_GAMEPAD_DPAD_UP as i32, VK_PAD_DPAD_UP as WORD),
        (XINPUT_GAMEPAD_DPAD_DOWN as i32, VK_PAD_DPAD_DOWN as WORD),
        (XINPUT_GAMEPAD_DPAD_LEFT as i32, VK_PAD_DPAD_LEFT as WORD),
        (XINPUT_GAMEPAD_DPAD_RIGHT as i32, VK_PAD_DPAD_RIGHT as WORD),
        (XINPUT_GAMEPAD_START as i32, VK_PAD_START as WORD),
        (XINPUT_GAMEPAD_BACK as i32, VK_PAD_BACK as WORD),
        (XINPUT_GAMEPAD_LEFT_THUMB as i32, VK_PAD_LTHUMB_PRESS as WORD),
        (XINPUT_GAMEPAD_RIGHT_THUMB as i32, VK_PAD_RTHUMB_PRESS as WORD),
        (XINPUT_GAMEPAD_LEFT_SHOULDER as i32, VK_PAD_LSHOULDER as WORD),
        (XINPUT_GAMEPAD_RIGHT_SHOULDER as i32, VK_PAD_RSHOULDER as WORD),
        (XINPUT_GAMEPAD_A as i32, VK_PAD_A as WORD),
        (XINPUT_GAMEPAD_B as i32, VK_PAD_B as WORD),
        (XINPUT_GAMEPAD_X as i32, VK_PAD_X as WORD),
        (XINPUT_GAMEPAD_Y as i32, VK_PAD_Y as WORD),
        // note: guide button does not send an event
    ];

    ret = xinput_get_state(index, core::ptr::addr_of_mut!(state));
    if ret != 0 {
        return ret;
    }
    AcquireSRWLockExclusive(core::ptr::addr_of_mut!(keystroke_lock));

    cur = core::ptr::addr_of!(state.Gamepad);

    'done: {
        /*** buttons ***/
        i = 0 as i32;
        while i < ARRAY_SIZE!(buttons) as i32 {
            if ((*cur).wButtons as i32 & buttons[i as usize].0)
                ^ ((*last).wButtons as i32 & buttons[i as usize].0)
                != 0
            {
                (*keystroke).VirtualKey = buttons[i as usize].1;
                (*keystroke).Unicode = 0 as WCHAR; /* unused */
                if (*cur).wButtons as i32 & buttons[i as usize].0 != 0 {
                    (*keystroke).Flags = XINPUT_KEYSTROKE_KEYDOWN as WORD;
                    (*last).wButtons =
                        ((*last).wButtons as i32 | buttons[i as usize].0) as WORD;
                } else {
                    (*keystroke).Flags = XINPUT_KEYSTROKE_KEYUP as WORD;
                    (*last).wButtons =
                        ((*last).wButtons as i32 & !buttons[i as usize].0) as WORD;
                }
                (*keystroke).UserIndex = index as BYTE;
                (*keystroke).HidCode = 0 as BYTE;
                ret = ERROR_SUCCESS as DWORD;
                break 'done;
            }
            i += 1;
        }

        /*** triggers ***/
        if (trigger_is_on((*cur).bLeftTrigger)
            ^ trigger_is_on((*last).bLeftTrigger))
            != 0
        {
            (*keystroke).VirtualKey = VK_PAD_LTRIGGER as WORD;
            (*keystroke).Unicode = 0 as WCHAR; /* unused */
            (*keystroke).Flags = (if trigger_is_on((*cur).bLeftTrigger) != 0 {
                XINPUT_KEYSTROKE_KEYDOWN
            } else {
                XINPUT_KEYSTROKE_KEYUP
            }) as WORD;
            (*keystroke).UserIndex = index as BYTE;
            (*keystroke).HidCode = 0 as BYTE;
            (*last).bLeftTrigger = (*cur).bLeftTrigger;
            ret = ERROR_SUCCESS as DWORD;
            break 'done;
        }

        if (trigger_is_on((*cur).bRightTrigger)
            ^ trigger_is_on((*last).bRightTrigger))
            != 0
        {
            (*keystroke).VirtualKey = VK_PAD_RTRIGGER as WORD;
            (*keystroke).Unicode = 0 as WCHAR; /* unused */
            (*keystroke).Flags = (if trigger_is_on((*cur).bRightTrigger) != 0 {
                XINPUT_KEYSTROKE_KEYDOWN
            } else {
                XINPUT_KEYSTROKE_KEYUP
            }) as WORD;
            (*keystroke).UserIndex = index as BYTE;
            (*keystroke).HidCode = 0 as BYTE;
            (*last).bRightTrigger = (*cur).bRightTrigger;
            ret = ERROR_SUCCESS as DWORD;
            break 'done;
        }

        /*** joysticks ***/
        ret = check_joystick_keystroke(
            index,
            keystroke,
            core::ptr::addr_of!((*cur).sThumbLX),
            core::ptr::addr_of!((*cur).sThumbLY),
            core::ptr::addr_of_mut!((*last).sThumbLX),
            core::ptr::addr_of_mut!((*last).sThumbLY),
            VK_PAD_LTHUMB_UP as WORD,
        );
        if ret == ERROR_SUCCESS as DWORD {
            break 'done;
        }

        ret = check_joystick_keystroke(
            index,
            keystroke,
            core::ptr::addr_of!((*cur).sThumbRX),
            core::ptr::addr_of!((*cur).sThumbRY),
            core::ptr::addr_of_mut!((*last).sThumbRX),
            core::ptr::addr_of_mut!((*last).sThumbRY),
            VK_PAD_RTHUMB_UP as WORD,
        );
        if ret == ERROR_SUCCESS as DWORD {
            break 'done;
        }
    }

    // done:
    ReleaseSRWLockExclusive(core::ptr::addr_of_mut!(keystroke_lock));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetKeystroke(
    mut index: DWORD,
    mut reserved: DWORD,
    mut keystroke: PXINPUT_KEYSTROKE,
) -> DWORD {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/xinput1_3/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"XInputGetKeystroke\0" as *const u8 as *const ::core::ffi::c_char,
            1095 as ::core::ffi::c_int,
            b"index %lu, reserved %lu, keystroke %p.\n\0" as *const u8
                as *const ::core::ffi::c_char,
            index,
            reserved,
            keystroke,
        );
    }

    if index >= XUSER_MAX_COUNT && index != XUSER_INDEX_ANY {
        return ERROR_BAD_ARGUMENTS;
    }

    if index == XUSER_INDEX_ANY {
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < (XUSER_MAX_COUNT as i32) {
            if check_for_keystroke(i as DWORD, keystroke as *mut XINPUT_KEYSTROKE)
                == ERROR_SUCCESS
            {
                return ERROR_SUCCESS;
            }
            i += 1;
        }
        return ERROR_EMPTY;
    }

    return check_for_keystroke(index, keystroke as *mut XINPUT_KEYSTROKE);
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetCapabilities(index: DWORD, flags: DWORD, capabilities: *mut XINPUT_CAPABILITIES) -> DWORD {
    let mut caps_ex: XINPUT_CAPABILITIES_EX = core::mem::zeroed();
    let mut ret: DWORD;

    ret = XInputGetCapabilitiesEx(1 as DWORD, index, flags, core::ptr::addr_of_mut!(caps_ex));

    if ret == 0 {
        *capabilities = caps_ex.Capabilities;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetDSoundAudioDeviceGuids(
    mut index: DWORD,
    mut render_guid: *mut GUID,
    mut capture_guid: *mut GUID,
) -> DWORD {
    let mut ret: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/xinput1_3/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"XInputGetDSoundAudioDeviceGuids\0" as *const u8 as *const ::core::ffi::c_char,
            1128 as ::core::ffi::c_int,
            b"index %lu, render_guid %s, capture_guid %s stub!\n\0" as *const u8
                as *const ::core::ffi::c_char,
            index,
            debugstr_guid(render_guid),
            debugstr_guid(capture_guid),
        );
    }
    if index >= XUSER_MAX_COUNT as DWORD || render_guid.is_null() || capture_guid.is_null() {
        return ERROR_BAD_ARGUMENTS as DWORD;
    }
    EnterCriticalSection(&raw mut xinput_cs);
    ret = (if !controllers[index as usize].device.is_null() {
        ERROR_NOT_SUPPORTED
    } else {
        ERROR_DEVICE_NOT_CONNECTED
    }) as DWORD;
    LeaveCriticalSection(&raw mut xinput_cs);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetBatteryInformation(
    mut index: DWORD,
    mut type_0: BYTE,
    mut battery: *mut XINPUT_BATTERY_INFORMATION,
) -> DWORD {
    static mut once: i32 = 0;
    let mut ret: DWORD = 0;
    let old_once = once;
    once = once + 1;
    if old_once == 0 {
        FIXME!("index %lu, type %u, battery %p.\n", index, type_0 as core::ffi::c_uint, battery);
    }
    if index >= XUSER_MAX_COUNT {
        return ERROR_BAD_ARGUMENTS;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
    ret = if !controllers[index as usize].device.is_null() {
        ERROR_NOT_SUPPORTED
    } else {
        ERROR_DEVICE_NOT_CONNECTED
    };
    LeaveCriticalSection(core::ptr::addr_of_mut!(xinput_cs));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetCapabilitiesEx(
    mut unk: DWORD,
    mut index: DWORD,
    mut flags: DWORD,
    mut caps: *mut XINPUT_CAPABILITIES_EX,
) -> DWORD {
    let mut attr: HIDD_ATTRIBUTES = core::mem::zeroed();
    let mut ret: DWORD = ERROR_SUCCESS as DWORD;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/xinput1_3/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"XInputGetCapabilitiesEx\0" as *const u8 as *const ::core::ffi::c_char,
            1160 as ::core::ffi::c_int,
            b"unk %lu, index %lu, flags %#lx, capabilities %p.\n\0" as *const u8
                as *const ::core::ffi::c_char,
            unk,
            index,
            flags,
            caps,
        );
    }
    start_update_thread();
    if index >= XUSER_MAX_COUNT as DWORD {
        return ERROR_BAD_ARGUMENTS as DWORD;
    }
    EnterCriticalSection(&raw mut xinput_cs);
    if controllers[index as usize].device.is_null() {
        update_controller_list();
    }
    if controllers[index as usize].device.is_null() {
        ret = ERROR_DEVICE_NOT_CONNECTED as DWORD;
    } else if (flags & (XINPUT_FLAG_GAMEPAD as DWORD)) != 0
        && controllers[index as usize].caps.SubType as ::core::ffi::c_int
            != XINPUT_DEVSUBTYPE_GAMEPAD as ::core::ffi::c_int
    {
        ret = ERROR_DEVICE_NOT_CONNECTED as DWORD;
    } else if HidD_GetAttributes(controllers[index as usize].device, &raw mut attr) == 0 {
        ret = ERROR_DEVICE_NOT_CONNECTED as DWORD;
    } else {
        (*caps).Capabilities = controllers[index as usize].caps;
        (*caps).VendorId = attr.VendorID as WORD;
        (*caps).ProductId = attr.ProductID as WORD;
        (*caps).VersionNumber = attr.VersionNumber as WORD;
    }
    LeaveCriticalSection(&raw mut xinput_cs);
    return ret;
}
