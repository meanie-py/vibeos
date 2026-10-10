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

#[no_mangle]
pub unsafe extern "system" fn GetDisplayConfigBufferSizes(
    flags: UINT32,
    numPathArrayElements: *mut UINT32,
    numModeInfoArrayElements: *mut UINT32,
) -> LONG {
    /* Validate parameters */
    if numPathArrayElements.is_null() || numModeInfoArrayElements.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    /* We don't support WDDM yet */
    return ERROR_NOT_SUPPORTED as LONG;
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
    (ERROR_ACCESS_DENIED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn DisplayConfigGetDeviceInfo(
    packet: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
) -> LONG {
    // FIXME( "DisplayConfigGetDeviceInfo: stub!\n" );
    1
}

#[no_mangle]
pub unsafe extern "system" fn DisplayConfigSetDeviceInfo(
    packet: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
) -> LONG {
    // FIXME( "DisplayConfigSetDeviceInfo: stub!\n" );
    1
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
