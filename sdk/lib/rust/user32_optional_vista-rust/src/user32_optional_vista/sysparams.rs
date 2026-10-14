use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WIN32_NO_STATUS, _INC_WINDOWS, COM_NO_WINDOWS_H: preprocessor guards, no Rust equivalent.
// WINE_DEFAULT_DEBUG_CHANNEL(win): debug-channel registration macro, no Rust declaration emitted.

pub type DISPLAYCONFIG_TOPOLOGY_ID = u32;
pub const DISPLAYCONFIG_TOPOLOGY_INTERNAL: u32 = 0x00000001;
pub const DISPLAYCONFIG_TOPOLOGY_CLONE: u32 = 0x00000002;
pub const DISPLAYCONFIG_TOPOLOGY_EXTEND: u32 = 0x00000004;
pub const DISPLAYCONFIG_TOPOLOGY_EXTERNAL: u32 = 0x00000008;
pub const DISPLAYCONFIG_TOPOLOGY_FORCE_UINT32: u32 = 0xFFFFFFFF;

pub type DISPLAYCONFIG_DEVICE_INFO_TYPE = u32;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME: u32 = 1;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME: u32 = 2;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_PREFERRED_MODE: u32 = 3;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME: u32 = 4;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_TARGET_PERSISTENCE: u32 = 5;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_BASE_TYPE: u32 = 6;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_SUPPORT_VIRTUAL_RESOLUTION: u32 = 7;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_SUPPORT_VIRTUAL_RESOLUTION: u32 = 8;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO: u32 = 9;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE: u32 = 10;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL: u32 = 11;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_MONITOR_SPECIALIZATION: u32 = 12;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_MONITOR_SPECIALIZATION: u32 = 13;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_RESERVED1: u32 = 14;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO_2: u32 = 15;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_HDR_STATE: u32 = 16;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_WCG_STATE: u32 = 17;
pub const DISPLAYCONFIG_DEVICE_INFO_FORCE_UINT32: u32 = 0xFFFFFFFF;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_DEVICE_INFO_HEADER {
    pub type_: DISPLAYCONFIG_DEVICE_INFO_TYPE,
    pub size: UINT32,
    pub adapterId: LUID,
    pub id: UINT32,
}
pub type PDISPLAYCONFIG_DEVICE_INFO_HEADER = *mut DISPLAYCONFIG_DEVICE_INFO_HEADER;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_DESKTOP_IMAGE_INFO {
    pub PathSourceSize: POINTL,
    pub DesktopImageRegion: RECTL,
    pub DesktopImageClip: RECTL,
}

pub type DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = i32;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER: i32 = -1;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15: i32 = 0;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SVIDEO: i32 = 1;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPOSITE_VIDEO: i32 = 2;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPONENT_VIDEO: i32 = 3;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI: i32 = 4;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI: i32 = 5;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_LVDS: i32 = 6;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_D_JPN: i32 = 8;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SDI: i32 = 9;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL: i32 = 10;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED: i32 = 11;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EXTERNAL: i32 = 12;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED: i32 = 13;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SDTVDONGLE: i32 = 14;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_MIRACAST: i32 = 15;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_WIRED: i32 = 16;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_VIRTUAL: i32 = 17;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_USB_TUNNEL: i32 = 18;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL: i32 = 0x80000000u32 as i32;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_FORCE_UINT32: i32 = 0xFFFFFFFFu32 as i32;

pub type DISPLAYCONFIG_ROTATION = u32;
pub const DISPLAYCONFIG_ROTATION_IDENTITY: u32 = 1;
pub const DISPLAYCONFIG_ROTATION_ROTATE90: u32 = 2;
pub const DISPLAYCONFIG_ROTATION_ROTATE180: u32 = 3;
pub const DISPLAYCONFIG_ROTATION_ROTATE270: u32 = 4;
pub const DISPLAYCONFIG_ROTATION_FORCE_UINT32: u32 = 0xFFFFFFFF;

pub type DISPLAYCONFIG_SCALING = u32;
pub const DISPLAYCONFIG_SCALING_IDENTITY: u32 = 1;
pub const DISPLAYCONFIG_SCALING_CENTERED: u32 = 2;
pub const DISPLAYCONFIG_SCALING_STRETCHED: u32 = 3;
pub const DISPLAYCONFIG_SCALING_ASPECTRATIOCENTEREDMAX: u32 = 4;
pub const DISPLAYCONFIG_SCALING_CUSTOM: u32 = 5;
pub const DISPLAYCONFIG_SCALING_PREFERRED: u32 = 128;
pub const DISPLAYCONFIG_SCALING_FORCE_UINT32: u32 = 0xFFFFFFFF;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_RATIONAL {
    pub Numerator: UINT32,
    pub Denominator: UINT32,
}

pub type DISPLAYCONFIG_SCANLINE_ORDERING = u32;
pub const DISPLAYCONFIG_SCANLINE_ORDERING_UNSPECIFIED: u32 = 0;
pub const DISPLAYCONFIG_SCANLINE_ORDERING_PROGRESSIVE: u32 = 1;
pub const DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED: u32 = 2;
pub const DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED_UPPERFIELDFIRST: u32 = 3;
pub const DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED_LOWERFIELDFIRST: u32 = 3;
pub const DISPLAYCONFIG_SCANLINE_ORDERING_FORCE_UINT32: u32 = 0xFFFFFFFF;

// Bitfield struct: bits packed into _bitfield (desktopModeInfoIdx: low 16, targetModeInfoIdx: high 16).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_TARGET_INFO_DUMMYSTRUCTNAME {
    pub _bitfield: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_PATH_TARGET_INFO_DUMMYUNIONNAME {
    pub modeInfoIdx: UINT32,
    pub DUMMYSTRUCTNAME: DISPLAYCONFIG_PATH_TARGET_INFO_DUMMYSTRUCTNAME,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_TARGET_INFO {
    pub adapterId: LUID,
    pub id: UINT32,
    pub DUMMYUNIONNAME: DISPLAYCONFIG_PATH_TARGET_INFO_DUMMYUNIONNAME,
    pub outputTechnology: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
    pub rotation: DISPLAYCONFIG_ROTATION,
    pub scaling: DISPLAYCONFIG_SCALING,
    pub refreshRate: DISPLAYCONFIG_RATIONAL,
    pub scanLineOrdering: DISPLAYCONFIG_SCANLINE_ORDERING,
    pub targetAvailable: BOOL,
    pub statusFlags: UINT32,
}

// Bitfield struct: bits packed into _bitfield (cloneGroupId: low 16, sourceModeInfoIdx: high 16).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_SOURCE_INFO_DUMMYSTRUCTNAME {
    pub _bitfield: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_PATH_SOURCE_INFO_DUMMYUNIONNAME {
    pub modeInfoIdx: UINT32,
    pub DUMMYSTRUCTNAME: DISPLAYCONFIG_PATH_SOURCE_INFO_DUMMYSTRUCTNAME,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_SOURCE_INFO {
    pub adapterId: LUID,
    pub id: UINT32,
    pub DUMMYUNIONNAME: DISPLAYCONFIG_PATH_SOURCE_INFO_DUMMYUNIONNAME,
    pub statusFlags: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_INFO {
    pub sourceInfo: DISPLAYCONFIG_PATH_SOURCE_INFO,
    pub targetInfo: DISPLAYCONFIG_PATH_TARGET_INFO,
    pub flags: UINT32,
}

pub type DISPLAYCONFIG_PIXELFORMAT = u32;
pub const DISPLAYCONFIG_PIXELFORMAT_8BPP: u32 = 1;
pub const DISPLAYCONFIG_PIXELFORMAT_16BPP: u32 = 2;
pub const DISPLAYCONFIG_PIXELFORMAT_24BPP: u32 = 3;
pub const DISPLAYCONFIG_PIXELFORMAT_32BPP: u32 = 4;
pub const DISPLAYCONFIG_PIXELFORMAT_NONGDI: u32 = 5;
pub const DISPLAYCONFIG_PIXELFORMAT_FORCE_UINT32: u32 = 0xffffffff;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_SOURCE_MODE {
    pub width: UINT32,
    pub height: UINT32,
    pub pixelFormat: DISPLAYCONFIG_PIXELFORMAT,
    pub position: POINTL,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_2DREGION {
    pub cx: UINT32,
    pub cy: UINT32,
}

// Bitfield struct: videoStandard (bits 0-15), vSyncFreqDivider (16-21), reserved (22-31) packed in _bitfield.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_VIDEO_SIGNAL_INFO_DUMMYUNIONNAME_AdditionalSignalInfo {
    pub _bitfield: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_VIDEO_SIGNAL_INFO_DUMMYUNIONNAME {
    pub AdditionalSignalInfo: DISPLAYCONFIG_VIDEO_SIGNAL_INFO_DUMMYUNIONNAME_AdditionalSignalInfo,
    pub videoStandard: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_VIDEO_SIGNAL_INFO {
    pub pixelRate: UINT64,
    pub hSyncFreq: DISPLAYCONFIG_RATIONAL,
    pub vSyncFreq: DISPLAYCONFIG_RATIONAL,
    pub activeSize: DISPLAYCONFIG_2DREGION,
    pub totalSize: DISPLAYCONFIG_2DREGION,
    pub DUMMYUNIONNAME: DISPLAYCONFIG_VIDEO_SIGNAL_INFO_DUMMYUNIONNAME,
    pub scanLineOrdering: DISPLAYCONFIG_SCANLINE_ORDERING,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_TARGET_MODE {
    pub targetVideoSignalInfo: DISPLAYCONFIG_VIDEO_SIGNAL_INFO,
}

pub type DISPLAYCONFIG_MODE_INFO_TYPE = u32;
pub const DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE: u32 = 1;
pub const DISPLAYCONFIG_MODE_INFO_TYPE_TARGET: u32 = 2;
pub const DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE: u32 = 3;
pub const DISPLAYCONFIG_MODE_INFO_TYPE_FORCE_UINT32: u32 = 0xFFFFFFFF;

#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_MODE_INFO_DUMMYUNIONNAME {
    pub targetMode: DISPLAYCONFIG_TARGET_MODE,
    pub sourceMode: DISPLAYCONFIG_SOURCE_MODE,
    pub desktopImageInfo: DISPLAYCONFIG_DESKTOP_IMAGE_INFO,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_MODE_INFO {
    pub infoType: DISPLAYCONFIG_MODE_INFO_TYPE,
    pub id: UINT32,
    pub adapterId: LUID,
    pub DUMMYUNIONNAME: DISPLAYCONFIG_MODE_INFO_DUMMYUNIONNAME,
}

pub type ORIENTATION_PREFERENCE = u32;
pub const ORIENTATION_PREFERENCE_NONE: u32 = 0x0;
pub const ORIENTATION_PREFERENCE_LANDSCAPE: u32 = 0x1;
pub const ORIENTATION_PREFERENCE_PORTRAIT: u32 = 0x2;
pub const ORIENTATION_PREFERENCE_LANDSCAPE_FLIPPED: u32 = 0x4;
pub const ORIENTATION_PREFERENCE_PORTRAIT_FLIPPED: u32 = 0x8;

// ---- DisplayConfig request packets (wingdi.h, exact SDK layouts) ----
pub const DISPLAYCONFIG_SOURCE_IN_USE: u32 = 0x00000001;
pub const DISPLAYCONFIG_TARGET_IN_USE: u32 = 0x00000001;
pub const DISPLAYCONFIG_PATH_ACTIVE: u32 = 0x00000001;
pub const QDC_ALL_PATHS: u32 = 0x00000001;
pub const QDC_ONLY_ACTIVE_PATHS: u32 = 0x00000002;
pub const QDC_DATABASE_CURRENT: u32 = 0x00000004;
// QDC_VIRTUAL_MODE_AWARE | QDC_INCLUDE_HMD | QDC_VIRTUAL_REFRESH_RATE_AWARE: accepted, no effect.
const QDC_MODIFIER_MASK: u32 = 0x00000010 | 0x00000020 | 0x00000040;
/// d3dkmdt.h D3DKMDT_VSS_OTHER (the video standard of a non-analogue signal).
const D3DKMDT_VSS_OTHER: u32 = 255;
const ENUM_CURRENT_SETTINGS_DC: u32 = 0xFFFFFFFF;
const ERROR_INSUFFICIENT_BUFFER_DC: LONG = 122;
const ERROR_GEN_FAILURE_DC: LONG = 31;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_SOURCE_DEVICE_NAME {
    pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    pub viewGdiDeviceName: [WCHAR; 32],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS {
    /// friendlyNameFromEdid:1, friendlyNameForced:1, edidIdsValid:1, reserved:29
    pub value: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_TARGET_DEVICE_NAME {
    pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    pub flags: DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS,
    pub outputTechnology: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
    pub edidManufactureId: u16,
    pub edidProductCodeId: u16,
    pub connectorInstance: UINT32,
    pub monitorFriendlyDeviceName: [WCHAR; 64],
    pub monitorDevicePath: [WCHAR; 128],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_ADAPTER_NAME {
    pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    pub adapterDevicePath: [WCHAR; 128],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_TARGET_PREFERRED_MODE {
    pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    pub width: UINT32,
    pub height: UINT32,
    pub targetMode: DISPLAYCONFIG_TARGET_MODE,
}

// The one adapter / one source / one target this machine has.  The LUID is fixed so that
// every call agrees on it.
const ADAPTER_LUID_LOW: u32 = 1;
const SOURCE_ID: UINT32 = 0;
const TARGET_ID: UINT32 = 0;

#[inline]
fn adapter_luid() -> LUID {
    LUID { LowPart: ADAPTER_LUID_LOW, HighPart: 0 }
}

extern "system" {
    fn EnumDisplayDevicesW(lpDevice: *const WCHAR, iDevNum: DWORD, lpDisplayDevice: *mut DISPLAY_DEVICEW, dwFlags: DWORD) -> BOOL;
    fn EnumDisplaySettingsW(lpszDeviceName: *const WCHAR, iModeNum: DWORD, lpDevMode: *mut DEVMODEW) -> BOOL;
}

/// The primary display device and its current mode.
unsafe fn primary_display(dd: &mut DISPLAY_DEVICEW, dm: &mut DEVMODEW) -> bool {
    let mut i: DWORD = 0;
    let mut found = false;
    loop {
        core::ptr::write_bytes(dd as *mut DISPLAY_DEVICEW, 0, 1);
        dd.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
        if EnumDisplayDevicesW(core::ptr::null(), i, dd, 0) == 0 {
            break;
        }
        if dd.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE != 0 {
            found = true;
            break;
        }
        if i == 0 {
            // Fall back to device 0 if nothing is flagged primary.
            found = true;
        }
        i += 1;
        if i >= 16 {
            break;
        }
    }
    if !found {
        return false;
    }
    if dd.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE == 0 {
        core::ptr::write_bytes(dd as *mut DISPLAY_DEVICEW, 0, 1);
        dd.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
        if EnumDisplayDevicesW(core::ptr::null(), 0, dd, 0) == 0 {
            return false;
        }
    }
    core::ptr::write_bytes(dm as *mut DEVMODEW, 0, 1);
    dm.dmSize = core::mem::size_of::<DEVMODEW>() as WORD;
    EnumDisplaySettingsW(dd.DeviceName.as_ptr(), ENUM_CURRENT_SETTINGS_DC, dm) != 0
}

/// Copy a NUL-terminated wide string, truncating, always NUL-terminating `dst`.
unsafe fn copy_wstr(dst: *mut WCHAR, cap: usize, src: *const WCHAR) {
    let mut n = 0usize;
    while n + 1 < cap && *src.add(n) != 0 {
        n += 1;
    }
    core::ptr::copy_nonoverlapping(src, dst, n);
    *dst.add(n) = 0;
}

unsafe fn target_mode_from(dm: &DEVMODEW) -> DISPLAYCONFIG_TARGET_MODE {
    let freq = if dm.dmDisplayFrequency > 1 { dm.dmDisplayFrequency } else { 60 };
    let (w, h) = (dm.dmPelsWidth, dm.dmPelsHeight);
    // No blanking intervals are known: total == active.
    let mut m: DISPLAYCONFIG_TARGET_MODE = core::mem::zeroed();
    let s = &mut m.targetVideoSignalInfo;
    s.pixelRate = freq as u64 * w as u64 * h as u64;
    s.hSyncFreq = DISPLAYCONFIG_RATIONAL { Numerator: freq.wrapping_mul(h), Denominator: 1 };
    s.vSyncFreq = DISPLAYCONFIG_RATIONAL { Numerator: freq, Denominator: 1 };
    s.activeSize = DISPLAYCONFIG_2DREGION { cx: w, cy: h };
    s.totalSize = DISPLAYCONFIG_2DREGION { cx: w, cy: h };
    s.DUMMYUNIONNAME.videoStandard = D3DKMDT_VSS_OTHER;
    s.scanLineOrdering = DISPLAYCONFIG_SCANLINE_ORDERING_UNSPECIFIED;
    m
}

fn check_qdc_flags(flags: UINT32) -> Option<u32> {
    let base = flags & !QDC_MODIFIER_MASK;
    match base {
        QDC_ALL_PATHS | QDC_ONLY_ACTIVE_PATHS | QDC_DATABASE_CURRENT => Some(base),
        _ => None,
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetDisplayConfigBufferSizes(
    flags: UINT32,
    numPathArrayElements: *mut UINT32,
    numModeInfoArrayElements: *mut UINT32,
) -> LONG {
    if numPathArrayElements.is_null() || numModeInfoArrayElements.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    if check_qdc_flags(flags).is_none() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    // One path (source 0 -> target 0), each with a source mode and a target mode.
    *numPathArrayElements = 1;
    *numModeInfoArrayElements = 2;
    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn QueryDisplayConfig(
    flags: UINT32,
    numPathArrayElements: *mut UINT32,
    pathArray: *mut DISPLAYCONFIG_PATH_INFO,
    numModeInfoArrayElements: *mut UINT32,
    modeInfoArray: *mut DISPLAYCONFIG_MODE_INFO,
    currentTopologyId: *mut DISPLAYCONFIG_TOPOLOGY_ID,
) -> LONG {
    if numPathArrayElements.is_null()
        || numModeInfoArrayElements.is_null()
        || pathArray.is_null()
        || modeInfoArray.is_null()
    {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    let base = match check_qdc_flags(flags) {
        Some(b) => b,
        None => return ERROR_INVALID_PARAMETER as LONG,
    };
    // The topology id is returned for, and only for, QDC_DATABASE_CURRENT.
    if (base == QDC_DATABASE_CURRENT) == currentTopologyId.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    if *numPathArrayElements < 1 || *numModeInfoArrayElements < 2 {
        *numPathArrayElements = 1;
        *numModeInfoArrayElements = 2;
        return ERROR_INSUFFICIENT_BUFFER_DC;
    }

    let mut dd: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut dm: DEVMODEW = core::mem::zeroed();
    if !primary_display(&mut dd, &mut dm) {
        return ERROR_GEN_FAILURE_DC;
    }
    let freq = if dm.dmDisplayFrequency > 1 { dm.dmDisplayFrequency } else { 60 };
    let pos = dm.__bindgen_anon_1.__bindgen_anon_2.dmPosition;

    let mut path: DISPLAYCONFIG_PATH_INFO = core::mem::zeroed();
    path.sourceInfo.adapterId = adapter_luid();
    path.sourceInfo.id = SOURCE_ID;
    path.sourceInfo.DUMMYUNIONNAME.modeInfoIdx = 0;
    path.sourceInfo.statusFlags = DISPLAYCONFIG_SOURCE_IN_USE;
    path.targetInfo.adapterId = adapter_luid();
    path.targetInfo.id = TARGET_ID;
    path.targetInfo.DUMMYUNIONNAME.modeInfoIdx = 1;
    path.targetInfo.outputTechnology = DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER;
    path.targetInfo.rotation = DISPLAYCONFIG_ROTATION_IDENTITY;
    path.targetInfo.scaling = DISPLAYCONFIG_SCALING_IDENTITY;
    path.targetInfo.refreshRate = DISPLAYCONFIG_RATIONAL { Numerator: freq, Denominator: 1 };
    path.targetInfo.scanLineOrdering = DISPLAYCONFIG_SCANLINE_ORDERING_UNSPECIFIED;
    path.targetInfo.targetAvailable = 1;
    path.targetInfo.statusFlags = DISPLAYCONFIG_TARGET_IN_USE;
    path.flags = DISPLAYCONFIG_PATH_ACTIVE;
    *pathArray = path;

    let mut src: DISPLAYCONFIG_MODE_INFO = core::mem::zeroed();
    src.infoType = DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE;
    src.id = SOURCE_ID;
    src.adapterId = adapter_luid();
    src.DUMMYUNIONNAME.sourceMode = DISPLAYCONFIG_SOURCE_MODE {
        width: dm.dmPelsWidth,
        height: dm.dmPelsHeight,
        pixelFormat: match dm.dmBitsPerPel {
            8 => DISPLAYCONFIG_PIXELFORMAT_8BPP,
            16 => DISPLAYCONFIG_PIXELFORMAT_16BPP,
            24 => DISPLAYCONFIG_PIXELFORMAT_24BPP,
            _ => DISPLAYCONFIG_PIXELFORMAT_32BPP,
        },
        position: pos,
    };
    *modeInfoArray = src;

    let mut tgt: DISPLAYCONFIG_MODE_INFO = core::mem::zeroed();
    tgt.infoType = DISPLAYCONFIG_MODE_INFO_TYPE_TARGET;
    tgt.id = TARGET_ID;
    tgt.adapterId = adapter_luid();
    tgt.DUMMYUNIONNAME.targetMode = target_mode_from(&dm);
    *modeInfoArray.add(1) = tgt;

    *numPathArrayElements = 1;
    *numModeInfoArrayElements = 2;
    if !currentTopologyId.is_null() {
        // One display: the desktop is just the one monitor.
        *currentTopologyId = DISPLAYCONFIG_TOPOLOGY_INTERNAL;
    }
    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn DisplayConfigGetDeviceInfo(
    packet: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
) -> LONG {
    if packet.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    let hdr = *packet;
    if (hdr.size as usize) < core::mem::size_of::<DISPLAYCONFIG_DEVICE_INFO_HEADER>() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    let need = match hdr.type_ {
        DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME => core::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>(),
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME => core::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>(),
        DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME => core::mem::size_of::<DISPLAYCONFIG_ADAPTER_NAME>(),
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_PREFERRED_MODE => core::mem::size_of::<DISPLAYCONFIG_TARGET_PREFERRED_MODE>(),
        _ => return ERROR_NOT_SUPPORTED as LONG,
    };
    if (hdr.size as usize) < need {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    // Unknown adapter / source / target.
    if hdr.adapterId.LowPart != ADAPTER_LUID_LOW || hdr.adapterId.HighPart != 0 {
        return ERROR_GEN_FAILURE_DC;
    }
    let want_id = if hdr.type_ == DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME { SOURCE_ID } else { TARGET_ID };
    if hdr.type_ != DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME && hdr.id != want_id {
        return ERROR_GEN_FAILURE_DC;
    }

    let mut dd: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut dm: DEVMODEW = core::mem::zeroed();
    if !primary_display(&mut dd, &mut dm) {
        return ERROR_GEN_FAILURE_DC;
    }

    match hdr.type_ {
        DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME => {
            let p = packet as *mut DISPLAYCONFIG_SOURCE_DEVICE_NAME;
            copy_wstr((*p).viewGdiDeviceName.as_mut_ptr(), 32, dd.DeviceName.as_ptr());
        }
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME => {
            let p = packet as *mut DISPLAYCONFIG_TARGET_DEVICE_NAME;
            (*p).flags.value = 0; // no EDID is available
            (*p).outputTechnology = DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER;
            (*p).edidManufactureId = 0;
            (*p).edidProductCodeId = 0;
            (*p).connectorInstance = 0;
            (*p).monitorFriendlyDeviceName[0] = 0;
            (*p).monitorDevicePath[0] = 0;
            // The monitor attached to the primary device (child enumeration).
            let mut mon: DISPLAY_DEVICEW = core::mem::zeroed();
            mon.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
            if EnumDisplayDevicesW(dd.DeviceName.as_ptr(), 0, &mut mon, 0) != 0 {
                copy_wstr((*p).monitorFriendlyDeviceName.as_mut_ptr(), 64, mon.DeviceString.as_ptr());
                copy_wstr((*p).monitorDevicePath.as_mut_ptr(), 128, mon.DeviceID.as_ptr());
            }
        }
        DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME => {
            let p = packet as *mut DISPLAYCONFIG_ADAPTER_NAME;
            copy_wstr((*p).adapterDevicePath.as_mut_ptr(), 128, dd.DeviceID.as_ptr());
        }
        _ => {
            let p = packet as *mut DISPLAYCONFIG_TARGET_PREFERRED_MODE;
            (*p).width = dm.dmPelsWidth;
            (*p).height = dm.dmPelsHeight;
            (*p).targetMode = target_mode_from(&dm);
        }
    }
    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn DisplayConfigSetDeviceInfo(
    packet: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
) -> LONG {
    if packet.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    // Nothing on this machine is settable.
    ERROR_NOT_SUPPORTED as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetDisplayAutoRotationPreferences(orientation: *mut ORIENTATION_PREFERENCE) -> BOOL {
    *orientation = ORIENTATION_PREFERENCE_NONE;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetDisplayConfig(
    path_info_count: UINT32,
    path_info: *mut DISPLAYCONFIG_PATH_INFO,
    mode_info_count: UINT32,
    mode_info: *mut DISPLAYCONFIG_MODE_INFO,
    flags: UINT32,
) -> LONG {
    // FIXME("path_info_count %u, path_info %p, mode_info_count %u, mode_info %p, flags %#x stub.\n", path_info_count, path_info, mode_info_count, mode_info, flags);
    (ERROR_SUCCESS) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SetDisplayAutoRotationPreferences(
    orientation: ORIENTATION_PREFERENCE,
) -> BOOL {
    // FIXME("(%d): stub\n", orientation);
    (TRUE) as i32
}

// Sizes measured from the mingw-w64 SDK headers (llvm-mingw clang, sizeof), the same on
// i686 and x86_64: none of these holds a pointer.
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_DEVICE_INFO_HEADER>() == 20);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_DESKTOP_IMAGE_INFO>() == 40);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_RATIONAL>() == 8);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_PATH_TARGET_INFO>() == 48);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_PATH_SOURCE_INFO>() == 20);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_PATH_INFO>() == 72);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_SOURCE_MODE>() == 20);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_2DREGION>() == 8);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_VIDEO_SIGNAL_INFO>() == 48);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_TARGET_MODE>() == 48);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_MODE_INFO>() == 64);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() == 84);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS>() == 4);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() == 420);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_ADAPTER_NAME>() == 276);
const _: () = assert!(core::mem::size_of::<DISPLAYCONFIG_TARGET_PREFERRED_MODE>() == 80);
