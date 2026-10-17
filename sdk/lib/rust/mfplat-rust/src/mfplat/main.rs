use crate::mfplat::queue::MFInvokeCallback;
use crate::mfplat::buffer::buffer;
use crate::mfplat::sample::sample;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1B79C1FA9B08: [u16; 8] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x002F, 0x002F, 0x0000];
pub static RUSTCONV_WSTR_2872FAAF7237: [u16; 5] = [0x002Eu16, 0x0077, 0x0061, 0x0076, 0x0000];
pub static RUSTCONV_WSTR_3C86F98BD527: [u16; 12] = [
    0x004Du16, 0x0046, 0x0050, 0x0025, 0x006C, 0x006C, 0x0058, 0x002E, 0x0054, 0x004D,
    0x0050, 0x0000,
];
pub static RUSTCONV_WSTR_4BB168AF0C9D: [u16; 59] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004D, 0x0065, 0x0064,
    0x0069, 0x0061, 0x0020, 0x0046, 0x006F, 0x0075, 0x006E, 0x0064, 0x0061, 0x0074,
    0x0069, 0x006F, 0x006E, 0x005C, 0x0053, 0x0063, 0x0068, 0x0065, 0x006D, 0x0065,
    0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_51ABFF51CF66: [u16; 5] = [0x002Eu16, 0x006D, 0x0070, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_5D5100CABDC1: [u16; 11] = [
    0x0049u16, 0x006E, 0x0070, 0x0075, 0x0074, 0x0054, 0x0079, 0x0070, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_659752A281E4: [u16; 5] = [0x002Eu16, 0x0061, 0x0073, 0x0066, 0x0000];
pub static RUSTCONV_WSTR_714123A8AF14: [u16; 9] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_8949E72AB0EA: [u16; 49] = [
    0x0025u16, 0x0030, 0x0038, 0x0078, 0x002D, 0x0025, 0x0030, 0x0034, 0x0078, 0x002D,
    0x0025, 0x0030, 0x0034, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025,
    0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030,
    0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078,
    0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0000,
];
pub static RUSTCONV_WSTR_A38E93E48CB3: [u16; 11] = [
    0x0041u16, 0x0074, 0x0074, 0x0072, 0x0069, 0x0062, 0x0075, 0x0074, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_B17B36FA88F7: [u16; 12] = [
    0x004Fu16, 0x0075, 0x0074, 0x0070, 0x0075, 0x0074, 0x0054, 0x0079, 0x0070, 0x0065,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_BE99061A9E69: [u16; 5] = [0x002Eu16, 0x006D, 0x0034, 0x0076, 0x0000];
pub static RUSTCONV_WSTR_DE1FC36E58D3: [u16; 5] = [0x002Eu16, 0x006D, 0x0070, 0x0033, 0x0000];
pub static RUSTCONV_WSTR_E89EC7379787: [u16; 9] = [0x004Du16, 0x0046, 0x0054, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_F31527B5B4F5: [u16; 63] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004D, 0x0065, 0x0064,
    0x0069, 0x0061, 0x0020, 0x0046, 0x006F, 0x0075, 0x006E, 0x0064, 0x0061, 0x0074,
    0x0069, 0x006F, 0x006E, 0x005C, 0x0042, 0x0079, 0x0074, 0x0065, 0x0053, 0x0074,
    0x0072, 0x0065, 0x0061, 0x006D, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065,
    0x0072, 0x0073, 0x0000,
];

use core::ffi::c_void;

/// Hand-ported (`port hand`): the C body is a 390-entry X-macro table, which the
/// model could not emit within its output cap. Generated from main.c's `X(...)` list, in its
/// (bsearch-sorted) order; MF_READER_WRITER_D3D_MANAGER is the C's alias of
/// MF_SOURCE_READER_D3D_MANAGER and keeps its own name, as `#g` stringizes it unexpanded.
#[no_mangle]
pub unsafe extern "C" fn debugstr_attr(guid: *const GUID) -> *const ::core::ffi::c_char {
    static mut guid_defs: [guid_def; 390] = unsafe {
        [
            guid_def { guid: &raw const MF_READWRITE_MMCSS_CLASS, name: b"MF_READWRITE_MMCSS_CLASS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_MARKIN_HERE, name: b"MF_TOPONODE_MARKIN_HERE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_SUPPORTED_SYNC_FRAME_TYPES, name: b"MF_MT_H264_SUPPORTED_SYNC_FRAME_TYPES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_MARKOUT_HERE, name: b"MF_TOPONODE_MARKOUT_HERE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_ForceBob, name: b"EVRConfig_ForceBob\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_DECODER, name: b"MF_TOPONODE_DECODER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_AllowDropToBob, name: b"EVRConfig_AllowDropToBob\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_PROJECTSTART, name: b"MF_TOPOLOGY_PROJECTSTART\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_ForceThrottle, name: b"EVRConfig_ForceThrottle\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_VIDEO_MAX_MB_PER_SEC, name: b"MF_VIDEO_MAX_MB_PER_SEC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_PROJECTSTOP, name: b"MF_TOPOLOGY_PROJECTSTOP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SINK_WRITER_ENCODER_CONFIG, name: b"MF_SINK_WRITER_ENCODER_CONFIG\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_AllowDropToThrottle, name: b"EVRConfig_AllowDropToThrottle\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_NO_MARKIN_MARKOUT, name: b"MF_TOPOLOGY_NO_MARKIN_MARKOUT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_ForceHalfInterlace, name: b"EVRConfig_ForceHalfInterlace\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_AllowDropToHalfInterlace, name: b"EVRConfig_AllowDropToHalfInterlace\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_ForceScaling, name: b"EVRConfig_ForceScaling\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_CAPABILITIES, name: b"MF_MT_H264_CAPABILITIES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_AllowScaling, name: b"EVRConfig_AllowScaling\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_ENABLE_TRANSCODE_ONLY_TRANSFORMS, name: b"MF_SOURCE_READER_ENABLE_TRANSCODE_ONLY_TRANSFORMS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_PREFERRED_ENCODER_PROFILE, name: b"MFT_PREFERRED_ENCODER_PROFILE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_ForceBatching, name: b"EVRConfig_ForceBatching\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const EVRConfig_AllowBatching, name: b"EVRConfig_AllowBatching\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_DYNAMIC_CHANGE_NOT_ALLOWED, name: b"MF_TOPOLOGY_DYNAMIC_CHANGE_NOT_ALLOWED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_PROFILE, name: b"MF_MT_VIDEO_PROFILE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DV_AAUX_CTRL_PACK_1, name: b"MF_MT_DV_AAUX_CTRL_PACK_1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_ALPHA_MODE, name: b"MF_MT_ALPHA_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG2_TIMECODE, name: b"MF_MT_MPEG2_TIMECODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PMP_SERVER_CONTEXT, name: b"MF_PMP_SERVER_CONTEXT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSCODE_TOPOLOGYMODE, name: b"MF_TRANSCODE_TOPOLOGYMODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_SUPPORT_DYNAMIC_FORMAT_CHANGE, name: b"MFT_SUPPORT_DYNAMIC_FORMAT_CHANGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_TRACK_ID, name: b"MF_MEDIA_ENGINE_TRACK_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_MP3, name: b"MFTranscodeContainerType_MP3\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_CUSTOM_VIDEO_PRIMARIES, name: b"MF_MT_CUSTOM_VIDEO_PRIMARIES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_TIMESTAMP_CAN_BE_DTS, name: b"MF_MT_TIMESTAMP_CAN_BE_DTS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CODEC_MERIT_Attribute, name: b"MFT_CODEC_MERIT_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_PLAYBACK_MAX_DIMS, name: b"MF_TOPOLOGY_PLAYBACK_MAX_DIMS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_XVP_DISABLE_FRC, name: b"MF_XVP_DISABLE_FRC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_LOW_LATENCY, name: b"MF_LOW_LATENCY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSCODE_ADJUST_PROFILE, name: b"MF_TRANSCODE_ADJUST_PROFILE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, name: b"MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG2_FLAGS, name: b"MF_MT_MPEG2_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_AUDIO_CATEGORY, name: b"MF_MEDIA_ENGINE_AUDIO_CATEGORY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_PIXEL_ASPECT_RATIO, name: b"MF_MT_PIXEL_ASPECT_RATIO\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_VIDEO_PROCESSOR_ALGORITHM, name: b"MF_VIDEO_PROCESSOR_ALGORITHM\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_ENABLE_XVP_FOR_PLAYBACK, name: b"MF_TOPOLOGY_ENABLE_XVP_FOR_PLAYBACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CONNECTED_STREAM_ATTRIBUTE, name: b"MFT_CONNECTED_STREAM_ATTRIBUTE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_REALTIME_CONTENT, name: b"MF_MT_REALTIME_CONTENT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_CONTENT_PROTECTION_FLAGS, name: b"MF_MEDIA_ENGINE_CONTENT_PROTECTION_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_WRAPPED_TYPE, name: b"MF_MT_WRAPPED_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DRM_FLAGS, name: b"MF_MT_DRM_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AVG_BITRATE, name: b"MF_MT_AVG_BITRATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DECODER_USE_MAX_RESOLUTION, name: b"MF_MT_DECODER_USE_MAX_RESOLUTION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MAX_LUMINANCE_LEVEL, name: b"MF_MT_MAX_LUMINANCE_LEVEL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CONNECTED_TO_HW_STREAM, name: b"MFT_CONNECTED_TO_HW_STREAM\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D_AWARE, name: b"MF_SA_D3D_AWARE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_XVP_SAMPLE_LOCK_TIMEOUT, name: b"MF_XVP_SAMPLE_LOCK_TIMEOUT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MAX_KEYFRAME_SPACING, name: b"MF_MT_MAX_KEYFRAME_SPACING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_TRANSFORM_CLSID_Attribute, name: b"MFT_TRANSFORM_CLSID_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, name: b"MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AM_FORMAT_TYPE, name: b"MF_MT_AM_FORMAT_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SESSION_APPROX_EVENT_OCCURRENCE_TIME, name: b"MF_SESSION_APPROX_EVENT_OCCURRENCE_TIME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_SYNCHRONOUS_CLOSE, name: b"MF_MEDIA_ENGINE_SYNCHRONOUS_CLOSE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_MAX_MB_PER_SEC, name: b"MF_MT_H264_MAX_MB_PER_SEC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_MAX_BUFFERS, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_MAX_BUFFERS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_BLOCK_ALIGNMENT, name: b"MF_MT_AUDIO_BLOCK_ALIGNMENT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_PMPHOST_CONTEXT, name: b"MF_PD_PMPHOST_CONTEXT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_APP_CONTEXT, name: b"MF_PD_APP_CONTEXT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_DURATION, name: b"MF_PD_DURATION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_TOTAL_FILE_SIZE, name: b"MF_PD_TOTAL_FILE_SIZE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_AUDIO_ENCODING_BITRATE, name: b"MF_PD_AUDIO_ENCODING_BITRATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_VIDEO_ENCODING_BITRATE, name: b"MF_PD_VIDEO_ENCODING_BITRATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_TargetGlobalLuminance, name: b"MFSampleExtension_TargetGlobalLuminance\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_MIME_TYPE, name: b"MF_PD_MIME_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_SUPPORTED_SLICE_MODES, name: b"MF_MT_H264_SUPPORTED_SLICE_MODES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_LAST_MODIFIED_TIME, name: b"MF_PD_LAST_MODIFIED_TIME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const VIDEO_ZOOM_RECT, name: b"VIDEO_ZOOM_RECT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_PLAYBACK_ELEMENT_ID, name: b"MF_PD_PLAYBACK_ELEMENT_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE9, name: b"MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE9\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_ALL_SAMPLES_INDEPENDENT, name: b"MF_MT_ALL_SAMPLES_INDEPENDENT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D12_CLEAR_VALUE, name: b"MF_SA_D3D12_CLEAR_VALUE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_RESOURCE_FLAG_ALLOW_CROSS_ADAPTER, name: b"MF_MT_D3D12_RESOURCE_FLAG_ALLOW_CROSS_ADAPTER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_PREFERRED_LANGUAGE, name: b"MF_PD_PREFERRED_LANGUAGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_PLAYBACK_BOUNDARY_TIME, name: b"MF_PD_PLAYBACK_BOUNDARY_TIME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_TELEMETRY_APPLICATION_ID, name: b"MF_MEDIA_ENGINE_TELEMETRY_APPLICATION_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_MFT_LOCKED, name: b"MF_ACTIVATE_MFT_LOCKED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_WAVE, name: b"MFTranscodeContainerType_WAVE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT, name: b"MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, name: b"MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_FRAME_SIZE, name: b"MF_MT_FRAME_SIZE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_SIMULCAST_SUPPORT, name: b"MF_MT_H264_SIMULCAST_SUPPORT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SINK_WRITER_ASYNC_CALLBACK, name: b"MF_SINK_WRITER_ASYNC_CALLBACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_START_TIME_ON_PRESENTATION_SWITCH, name: b"MF_TOPOLOGY_START_TIME_ON_PRESENTATION_SWITCH\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_DECODER_EXPOSE_OUTPUT_TYPES_IN_NATIVE_ORDER, name: b"MFT_DECODER_EXPOSE_OUTPUT_TYPES_IN_NATIVE_ORDER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSCODE_CONTAINERTYPE, name: b"MF_TRANSCODE_CONTAINERTYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_WORKQUEUE_MMCSS_PRIORITY, name: b"MF_TOPONODE_WORKQUEUE_MMCSS_PRIORITY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_FRAME_RATE_RANGE_MAX, name: b"MF_MT_FRAME_RATE_RANGE_MAX\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_PALETTE, name: b"MF_MT_PALETTE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_PROVIDER_DEVICE_ID, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_PROVIDER_DEVICE_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_STATIC_PLAYBACK_OPTIMIZATIONS, name: b"MF_TOPOLOGY_STATIC_PLAYBACK_OPTIMIZATIONS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D11_USAGE, name: b"MF_SA_D3D11_USAGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_NEEDKEY_CALLBACK, name: b"MF_MEDIA_ENGINE_NEEDKEY_CALLBACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_GEOMETRIC_APERTURE, name: b"MF_MT_GEOMETRIC_APERTURE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_ORIGINAL_WAVE_FORMAT_TAG, name: b"MF_MT_ORIGINAL_WAVE_FORMAT_TAG\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DV_AAUX_SRC_PACK_1, name: b"MF_MT_DV_AAUX_SRC_PACK_1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_STREAM_CONTAINS_ALPHA_CHANNEL, name: b"MF_MEDIA_ENGINE_STREAM_CONTAINS_ALPHA_CHANNEL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_MEDIA_PLAYER_MODE, name: b"MF_MEDIA_ENGINE_MEDIA_PLAYER_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_EXTENSION, name: b"MF_MEDIA_ENGINE_EXTENSION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS, name: b"MF_MT_D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DEFAULT_STRIDE, name: b"MF_MT_DEFAULT_STRIDE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_ARBITRARY_FORMAT, name: b"MF_MT_ARBITRARY_FORMAT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSFORM_CATEGORY_Attribute, name: b"MF_TRANSFORM_CATEGORY_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG2_HDCP, name: b"MF_MT_MPEG2_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_FLOAT_SAMPLES_PER_SECOND, name: b"MF_MT_AUDIO_FLOAT_SAMPLES_PER_SECOND\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SPATIAL_AUDIO_MAX_DYNAMIC_OBJECTS, name: b"MF_MT_SPATIAL_AUDIO_MAX_DYNAMIC_OBJECTS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DECODER_MAX_DPB_COUNT, name: b"MF_MT_DECODER_MAX_DPB_COUNT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_ForwardedDecodeUnits, name: b"MFSampleExtension_ForwardedDecodeUnits\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D11_SHARED_WITHOUT_MUTEX, name: b"MF_SA_D3D11_SHARED_WITHOUT_MUTEX\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DV_AAUX_CTRL_PACK_0, name: b"MF_MT_DV_AAUX_CTRL_PACK_0\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_YUV_MATRIX, name: b"MF_MT_YUV_MATRIX\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SOURCE_TOPOLOGY_CANCELED, name: b"MF_EVENT_SOURCE_TOPOLOGY_CANCELED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG4_CURRENT_SAMPLE_ENTRY, name: b"MF_MT_MPEG4_CURRENT_SAMPLE_ENTRY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MAX_FRAME_AVERAGE_LUMINANCE_LEVEL, name: b"MF_MT_MAX_FRAME_AVERAGE_LUMINANCE_LEVEL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ENDPOINT_ID, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ENDPOINT_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, name: b"MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_ROTATION, name: b"MF_MT_VIDEO_ROTATION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_MPEG4, name: b"MFTranscodeContainerType_MPEG4\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_SYMBOLIC_LINK, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_SYMBOLIC_LINK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE11, name: b"MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE11\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_USER_DATA, name: b"MF_MT_USER_DATA\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_CUSTOM_VIDEO_MIXER_CLSID, name: b"MF_ACTIVATE_CUSTOM_VIDEO_MIXER_CLSID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MIN_MASTERING_LUMINANCE, name: b"MF_MT_MIN_MASTERING_LUMINANCE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_CUSTOM_VIDEO_MIXER_ACTIVATE, name: b"MF_ACTIVATE_CUSTOM_VIDEO_MIXER_ACTIVATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_REQUIRED_SAMPLE_COUNT, name: b"MF_SA_REQUIRED_SAMPLE_COUNT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_CUSTOM_VIDEO_MIXER_FLAGS, name: b"MF_ACTIVATE_CUSTOM_VIDEO_MIXER_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_CLSID, name: b"MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_CLSID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_STREAM_METADATA_SYSTEMID, name: b"MF_EVENT_STREAM_METADATA_SYSTEMID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_ACTIVATE, name: b"MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_ACTIVATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_CHANNEL_MASK, name: b"MF_MT_AUDIO_CHANNEL_MASK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_DISCONNECT_MEDIASOURCE_ON_SHUTDOWN, name: b"MF_SOURCE_READER_DISCONNECT_MEDIASOURCE_ON_SHUTDOWN\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_READWRITE_DISABLE_CONVERTERS, name: b"MF_READWRITE_DISABLE_CONVERTERS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE_EDGE, name: b"MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE_EDGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_FLAGS, name: b"MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D12_HEAP_FLAGS, name: b"MF_SA_D3D12_HEAP_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MINIMUM_DISPLAY_APERTURE, name: b"MF_MT_MINIMUM_DISPLAY_APERTURE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_Token, name: b"MFSampleExtension_Token\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_3GP, name: b"MFTranscodeContainerType_3GP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_CATEGORY, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_CATEGORY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_D3D12_SYNCHRONIZATION_OBJECT, name: b"MF_D3D12_SYNCHRONIZATION_OBJECT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_VALID_BITS_PER_SAMPLE, name: b"MF_MT_AUDIO_VALID_BITS_PER_SAMPLE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSFORM_ASYNC_UNLOCK, name: b"MF_TRANSFORM_ASYNC_UNLOCK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DISABLE_FRAME_CORRUPTION_INFO, name: b"MF_DISABLE_FRAME_CORRUPTION_INFO\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_ENUMERATE_SOURCE_TYPES, name: b"MF_TOPOLOGY_ENUMERATE_SOURCE_TYPES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_ASF, name: b"MFTranscodeContainerType_ASF\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_NO_FRAME_ORDERING, name: b"MF_MT_VIDEO_NO_FRAME_ORDERING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_PLAYBACK_VISUAL, name: b"MF_MEDIA_ENGINE_PLAYBACK_VISUAL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_CHROMA_SITING, name: b"MF_MT_VIDEO_CHROMA_SITING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_AUDIO_RENDERER_ATTRIBUTE_STREAM_CATEGORY, name: b"MF_AUDIO_RENDERER_ATTRIBUTE_STREAM_CATEGORY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_BUFFERS_PER_SAMPLE, name: b"MF_SA_BUFFERS_PER_SAMPLE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_3DVideo_SampleFormat, name: b"MFSampleExtension_3DVideo_SampleFormat\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_RESOLUTION_SCALING, name: b"MF_MT_H264_RESOLUTION_SCALING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_LEVEL, name: b"MF_MT_VIDEO_LEVEL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D12_HEAP_TYPE, name: b"MF_SA_D3D12_HEAP_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SAMPLEGRABBERSINK_SAMPLE_TIME_OFFSET, name: b"MF_SAMPLEGRABBERSINK_SAMPLE_TIME_OFFSET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SAMPLE_SIZE, name: b"MF_MT_SAMPLE_SIZE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AAC_PAYLOAD_TYPE, name: b"MF_MT_AAC_PAYLOAD_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_PLAYBACK_FRAMERATE, name: b"MF_TOPOLOGY_PLAYBACK_FRAMERATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_D3D11_BIND_FLAGS, name: b"MF_SOURCE_READER_D3D11_BIND_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSCODE_ENCODINGPROFILE, name: b"MF_TRANSCODE_ENCODINGPROFILE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_FOLDDOWN_MATRIX, name: b"MF_MT_AUDIO_FOLDDOWN_MATRIX\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_WMADRC_PEAKREF, name: b"MF_MT_AUDIO_WMADRC_PEAKREF\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_ADTS, name: b"MFTranscodeContainerType_ADTS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_WMADRC_PEAKTARGET, name: b"MF_MT_AUDIO_WMADRC_PEAKTARGET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSFORM_FLAGS_Attribute, name: b"MF_TRANSFORM_FLAGS_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_SUPPORTED_RATE_CONTROL_MODES, name: b"MF_MT_H264_SUPPORTED_RATE_CONTROL_MODES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_SAMI_STYLELIST, name: b"MF_PD_SAMI_STYLELIST\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_WMADRC_AVGREF, name: b"MF_MT_AUDIO_WMADRC_AVGREF\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_BITS_PER_SAMPLE, name: b"MF_MT_AUDIO_BITS_PER_SAMPLE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SD_LANGUAGE, name: b"MF_SD_LANGUAGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_WMADRC_AVGTARGET, name: b"MF_MT_AUDIO_WMADRC_AVGTARGET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SD_PROTECTED, name: b"MF_SD_PROTECTED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SESSION_TOPOLOADER, name: b"MF_SESSION_TOPOLOADER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SESSION_GLOBAL_TIME, name: b"MF_SESSION_GLOBAL_TIME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SESSION_QUALITY_MANAGER, name: b"MF_SESSION_QUALITY_MANAGER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SESSION_CONTENT_PROTECTION_MANAGER, name: b"MF_SESSION_CONTENT_PROTECTION_MANAGER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG4_SAMPLE_DESCRIPTION, name: b"MF_MT_MPEG4_SAMPLE_DESCRIPTION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D_RESOURCE_VERSION, name: b"MF_MT_D3D_RESOURCE_VERSION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, name: b"MF_MT_D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG_START_TIME_CODE, name: b"MF_MT_MPEG_START_TIME_CODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_REMUX_MARK_I_PICTURE_AS_CLEAN_POINT, name: b"MFT_REMUX_MARK_I_PICTURE_AS_CLEAN_POINT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_REMUX_MARK_I_PICTURE_AS_CLEAN_POINT, name: b"MFT_REMUX_MARK_I_PICTURE_AS_CLEAN_POINT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_READWRITE_MMCSS_PRIORITY_AUDIO, name: b"MF_READWRITE_MMCSS_PRIORITY_AUDIO\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_MAX_CODEC_CONFIG_DELAY, name: b"MF_MT_H264_MAX_CODEC_CONFIG_DELAY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DV_AAUX_SRC_PACK_0, name: b"MF_MT_DV_AAUX_SRC_PACK_0\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_BYTESTREAM_ORIGIN_NAME, name: b"MF_BYTESTREAM_ORIGIN_NAME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_BYTESTREAM_CONTENT_TYPE, name: b"MF_BYTESTREAM_CONTENT_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DEPTH_MEASUREMENT, name: b"MF_MT_DEPTH_MEASUREMENT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_COMPATIBILITY_MODE_WIN10, name: b"MF_MEDIA_ENGINE_COMPATIBILITY_MODE_WIN10\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_3D_NUM_VIEWS, name: b"MF_MT_VIDEO_3D_NUM_VIEWS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_BYTESTREAM_DURATION, name: b"MF_BYTESTREAM_DURATION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SD_SAMI_LANGUAGE, name: b"MF_SD_SAMI_LANGUAGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_OUTPUT_NODE, name: b"MF_EVENT_OUTPUT_NODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_BYTESTREAM_LAST_MODIFIED_TIME, name: b"MF_BYTESTREAM_LAST_MODIFIED_TIME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_ENUM_ADAPTER_LUID, name: b"MFT_ENUM_ADAPTER_LUID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_FRAME_RATE_RANGE_MIN, name: b"MF_MT_FRAME_RATE_RANGE_MIN\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_BYTESTREAM_IFO_FILE_URI, name: b"MF_BYTESTREAM_IFO_FILE_URI\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_TOPOLOGY_STATUS, name: b"MF_EVENT_TOPOLOGY_STATUS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_BYTESTREAM_DLNA_PROFILE_ID, name: b"MF_BYTESTREAM_DLNA_PROFILE_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ROLE, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ROLE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MAJOR_TYPE, name: b"MF_MT_MAJOR_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_REQUIRED_SAMPLE_COUNT_PROGRESSIVE, name: b"MF_SA_REQUIRED_SAMPLE_COUNT_PROGRESSIVE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_IN_BAND_PARAMETER_SET, name: b"MF_MT_IN_BAND_PARAMETER_SET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SOURCE_CHARACTERISTICS, name: b"MF_EVENT_SOURCE_CHARACTERISTICS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SOURCE_CHARACTERISTICS_OLD, name: b"MF_EVENT_SOURCE_CHARACTERISTICS_OLD\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SESSION_SERVER_CONTEXT, name: b"MF_SESSION_SERVER_CONTEXT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_3D_FIRST_IS_LEFT, name: b"MF_MT_VIDEO_3D_FIRST_IS_LEFT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_DECODER_FINAL_VIDEO_RESOLUTION_HINT, name: b"MFT_DECODER_FINAL_VIDEO_RESOLUTION_HINT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_ADAPTIVE_STREAMING, name: b"MF_PD_ADAPTIVE_STREAMING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_SOURCE_RESOLVER_CONFIG_STORE, name: b"MF_MEDIA_ENGINE_SOURCE_RESOLVER_CONFIG_STORE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_COMPATIBILITY_MODE_WWA_EDGE, name: b"MF_MEDIA_ENGINE_COMPATIBILITY_MODE_WWA_EDGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_SUPPORTED_USAGES, name: b"MF_MT_H264_SUPPORTED_USAGES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_PREFERRED_OUTPUTTYPE_Attribute, name: b"MFT_PREFERRED_OUTPUTTYPE_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_Timestamp, name: b"MFSampleExtension_Timestamp\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_PRIMARYOUTPUT, name: b"MF_TOPONODE_PRIMARYOUTPUT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SUBTYPE, name: b"MF_MT_SUBTYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSFORM_ASYNC, name: b"MF_TRANSFORM_ASYNC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_STREAMID, name: b"MF_TOPONODE_STREAMID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_PLAYBACK_HWND, name: b"MF_MEDIA_ENGINE_PLAYBACK_HWND\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_NOSHUTDOWN_ON_REMOVE, name: b"MF_TOPONODE_NOSHUTDOWN_ON_REMOVE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_LIGHTING, name: b"MF_MT_VIDEO_LIGHTING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SD_MUTUALLY_EXCLUSIVE, name: b"MF_SD_MUTUALLY_EXCLUSIVE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SD_STREAM_NAME, name: b"MF_SD_STREAM_NAME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DV_VAUX_SRC_PACK, name: b"MF_MT_DV_VAUX_SRC_PACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_RATELESS, name: b"MF_TOPONODE_RATELESS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_STREAM_METADATA_CONTENT_KEYIDS, name: b"MF_EVENT_STREAM_METADATA_CONTENT_KEYIDS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_DISABLE_PREROLL, name: b"MF_TOPONODE_DISABLE_PREROLL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D11_ALLOW_DYNAMIC_YUV_TEXTURE, name: b"MF_SA_D3D11_ALLOW_DYNAMIC_YUV_TEXTURE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_3D_FORMAT, name: b"MF_MT_VIDEO_3D_FORMAT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_STREAM_METADATA_KEYDATA, name: b"MF_EVENT_STREAM_METADATA_KEYDATA\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_D3D_MANAGER, name: b"MF_READER_WRITER_D3D_MANAGER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_FLAC, name: b"MFTranscodeContainerType_FLAC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_3DVideo, name: b"MFSampleExtension_3DVideo\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_MINIMUM_OUTPUT_SAMPLE_COUNT_PROGRESSIVE, name: b"MF_SA_MINIMUM_OUTPUT_SAMPLE_COUNT_PROGRESSIVE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_USAGE, name: b"MF_MT_H264_USAGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_EME_CALLBACK, name: b"MF_MEDIA_ENGINE_EME_CALLBACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SOURCE_FAKE_START, name: b"MF_EVENT_SOURCE_FAKE_START\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SOURCE_PROJECTSTART, name: b"MF_EVENT_SOURCE_PROJECTSTART\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SOURCE_ACTUAL_START, name: b"MF_EVENT_SOURCE_ACTUAL_START\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_TEXTURE_LAYOUT, name: b"MF_MT_D3D12_TEXTURE_LAYOUT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_CONTENT_PROTECTION_MANAGER, name: b"MF_MEDIA_ENGINE_CONTENT_PROTECTION_MANAGER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_SAMPLES_PER_BLOCK, name: b"MF_MT_AUDIO_SAMPLES_PER_BLOCK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_ENUM_HARDWARE_URL_Attribute, name: b"MFT_ENUM_HARDWARE_URL_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_ASYNC_CALLBACK, name: b"MF_SOURCE_READER_ASYNC_CALLBACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE, name: b"MF_MT_D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_OUTPUT_BUFFER_NUM, name: b"MF_MT_OUTPUT_BUFFER_NUM\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D11_BINDFLAGS, name: b"MF_SA_D3D11_BINDFLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_ENCODER_SUPPORTS_CONFIG_EVENT, name: b"MFT_ENCODER_SUPPORTS_CONFIG_EVENT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_FLAC_MAX_BLOCK_SIZE, name: b"MF_MT_AUDIO_FLAC_MAX_BLOCK_SIZE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_FRIENDLY_NAME_Attribute, name: b"MFT_FRIENDLY_NAME_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_AVI, name: b"MFTranscodeContainerType_AVI\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_FIXED_SIZE_SAMPLES, name: b"MF_MT_FIXED_SIZE_SAMPLES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_SUPPORT_3DVIDEO, name: b"MFT_SUPPORT_3DVIDEO\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_SUPPORT_3DVIDEO, name: b"MFT_SUPPORT_3DVIDEO\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_INPUT_TYPES_Attributes, name: b"MFT_INPUT_TYPES_Attributes\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS, name: b"MF_MT_D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_LAYOUT_PER_STREAM, name: b"MF_MT_H264_LAYOUT_PER_STREAM\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SCRUBSAMPLE_TIME, name: b"MF_EVENT_SCRUBSAMPLE_TIME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SPATIAL_AUDIO_MAX_METADATA_ITEMS, name: b"MF_MT_SPATIAL_AUDIO_MAX_METADATA_ITEMS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG2_ONE_FRAME_PER_PACKET, name: b"MF_MT_MPEG2_ONE_FRAME_PER_PACKET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_INTERLACE_MODE, name: b"MF_MT_INTERLACE_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_CALLBACK, name: b"MF_MEDIA_ENGINE_CALLBACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_RENDERER_EXTENSION_PROFILE, name: b"MF_MT_VIDEO_RENDERER_EXTENSION_PROFILE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_HW_SOURCE, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_HW_SOURCE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_PREFER_WAVEFORMATEX, name: b"MF_MT_AUDIO_PREFER_WAVEFORMATEX\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_XVP_CALLER_ALLOCATES_OUTPUT, name: b"MF_XVP_CALLER_ALLOCATES_OUTPUT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_SVC_CAPABILITIES, name: b"MF_MT_H264_SVC_CAPABILITIES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_WORKQUEUE_ITEM_PRIORITY, name: b"MF_TOPONODE_WORKQUEUE_ITEM_PRIORITY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SPATIAL_AUDIO_OBJECT_METADATA_LENGTH, name: b"MF_MT_SPATIAL_AUDIO_OBJECT_METADATA_LENGTH\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SPATIAL_AUDIO_OBJECT_METADATA_FORMAT_ID, name: b"MF_MT_SPATIAL_AUDIO_OBJECT_METADATA_FORMAT_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SAMPLEGRABBERSINK_IGNORE_CLOCK, name: b"MF_SAMPLEGRABBERSINK_IGNORE_CLOCK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D11_SHARED, name: b"MF_SA_D3D11_SHARED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_PAN_SCAN_ENABLED, name: b"MF_MT_PAN_SCAN_ENABLED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL, name: b"MF_MT_D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_AC3, name: b"MFTranscodeContainerType_AC3\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ID, name: b"MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DV_VAUX_CTRL_PACK, name: b"MF_MT_DV_VAUX_CTRL_PACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_ForwardedDecodeUnitType, name: b"MFSampleExtension_ForwardedDecodeUnitType\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D11_AWARE, name: b"MF_SA_D3D11_AWARE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_AVG_BYTES_PER_SECOND, name: b"MF_MT_AUDIO_AVG_BYTES_PER_SECOND\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_MEDIASOURCE_CHARACTERISTICS, name: b"MF_SOURCE_READER_MEDIASOURCE_CHARACTERISTICS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SPATIAL_AUDIO_MIN_METADATA_ITEM_OFFSET_SPACING, name: b"MF_MT_SPATIAL_AUDIO_MIN_METADATA_ITEM_OFFSET_SPACING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_TRANSFORM_OBJECTID, name: b"MF_TOPONODE_TRANSFORM_OBJECTID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_MEDIA_TYPE, name: b"MF_DEVSOURCE_ATTRIBUTE_MEDIA_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_MFT_INPUT_STREAM_ID, name: b"MF_EVENT_MFT_INPUT_STREAM_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SOURCE_CONTENT_HINT, name: b"MF_MT_SOURCE_CONTENT_HINT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_ENUM_HARDWARE_VENDOR_ID_Attribute, name: b"MFT_ENUM_HARDWARE_VENDOR_ID_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_ENUM_TRANSCODE_ONLY_ATTRIBUTE, name: b"MFT_ENUM_TRANSCODE_ONLY_ATTRIBUTE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_READWRITE_MMCSS_PRIORITY, name: b"MF_READWRITE_MMCSS_PRIORITY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSCODE_DONOT_INSERT_ENCODER, name: b"MF_TRANSCODE_DONOT_INSERT_ENCODER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_3D, name: b"MF_MT_VIDEO_3D\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_START_PRESENTATION_TIME, name: b"MF_EVENT_START_PRESENTATION_TIME\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SESSIONCAPS, name: b"MF_EVENT_SESSIONCAPS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_PRESENTATION_TIME_OFFSET, name: b"MF_EVENT_PRESENTATION_TIME_OFFSET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_AUDIO_ENDPOINT_ROLE, name: b"MF_MEDIA_ENGINE_AUDIO_ENDPOINT_ROLE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_SESSIONCAPS_DELTA, name: b"MF_EVENT_SESSIONCAPS_DELTA\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_START_PRESENTATION_TIME_AT_OUTPUT, name: b"MF_EVENT_START_PRESENTATION_TIME_AT_OUTPUT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_AMR, name: b"MFTranscodeContainerType_AMR\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFSampleExtension_DecodeTimestamp, name: b"MFSampleExtension_DecodeTimestamp\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_COMPATIBILITY_MODE, name: b"MF_MEDIA_ENGINE_COMPATIBILITY_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_MINIMUM_OUTPUT_SAMPLE_COUNT, name: b"MF_SA_MINIMUM_OUTPUT_SAMPLE_COUNT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_H264_NO_FMOASO, name: b"MF_MT_VIDEO_H264_NO_FMOASO\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AVG_BIT_ERROR_RATE, name: b"MF_MT_AVG_BIT_ERROR_RATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_PRIMARIES, name: b"MF_MT_VIDEO_PRIMARIES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SINK_WRITER_DISABLE_THROTTLING, name: b"MF_SINK_WRITER_DISABLE_THROTTLING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_H264_RATE_CONTROL_MODES, name: b"MF_MT_H264_RATE_CONTROL_MODES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_READWRITE_D3D_OPTIONAL, name: b"MF_READWRITE_D3D_OPTIONAL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SA_D3D11_HW_PROTECTED, name: b"MF_SA_D3D11_HW_PROTECTED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_DXGI_MANAGER, name: b"MF_MEDIA_ENGINE_DXGI_MANAGER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_READWRITE_MMCSS_CLASS_AUDIO, name: b"MF_READWRITE_MMCSS_CLASS_AUDIO\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_COREWINDOW, name: b"MF_MEDIA_ENGINE_COREWINDOW\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_DISABLE_CAMERA_PLUGINS, name: b"MF_SOURCE_READER_DISABLE_CAMERA_PLUGINS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG4_TRACK_TYPE, name: b"MF_MT_MPEG4_TRACK_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_ACTIVATE_VIDEO_WINDOW, name: b"MF_ACTIVATE_VIDEO_WINDOW\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_PAN_SCAN_APERTURE, name: b"MF_MT_PAN_SCAN_APERTURE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_RESOLUTION_STATUS, name: b"MF_TOPOLOGY_RESOLUTION_STATUS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_ORIGINAL_4CC, name: b"MF_MT_ORIGINAL_4CC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_PD_AUDIO_ISVARIABLEBITRATE, name: b"MF_PD_AUDIO_ISVARIABLEBITRATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_AUDIO_RENDERER_ATTRIBUTE_FLAGS, name: b"MF_AUDIO_RENDERER_ATTRIBUTE_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE, name: b"MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_D3D12_CPU_READBACK, name: b"MF_MT_D3D12_CPU_READBACK\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_AUDIO_RENDERER_ATTRIBUTE_SESSION_ID, name: b"MF_AUDIO_RENDERER_ATTRIBUTE_SESSION_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG2_CONTENT_PACKET, name: b"MF_MT_MPEG2_CONTENT_PACKET\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_PROCESS_LOCAL_Attribute, name: b"MFT_PROCESS_LOCAL_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_PROCESS_LOCAL_Attribute, name: b"MFT_PROCESS_LOCAL_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_PAD_CONTROL_FLAGS, name: b"MF_MT_PAD_CONTROL_FLAGS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_NOMINAL_RANGE, name: b"MF_MT_VIDEO_NOMINAL_RANGE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AAC_AUDIO_PROFILE_LEVEL_INDICATION, name: b"MF_MT_AAC_AUDIO_PROFILE_LEVEL_INDICATION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG_SEQUENCE_HEADER, name: b"MF_MT_MPEG_SEQUENCE_HEADER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_OPM_HWND, name: b"MF_MEDIA_ENGINE_OPM_HWND\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_SAMPLES_PER_SECOND, name: b"MF_MT_AUDIO_SAMPLES_PER_SECOND\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SPATIAL_AUDIO_DATA_PRESENT, name: b"MF_MT_SPATIAL_AUDIO_DATA_PRESENT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_FRAME_RATE, name: b"MF_MT_FRAME_RATE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_FLUSH, name: b"MF_TOPONODE_FLUSH\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_MPEG2_STANDARD, name: b"MF_MT_MPEG2_STANDARD\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_DRAIN, name: b"MF_TOPONODE_DRAIN\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_TRANSFER_FUNCTION, name: b"MF_MT_TRANSFER_FUNCTION\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_MEDIASTART, name: b"MF_TOPONODE_MEDIASTART\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_MEDIASTOP, name: b"MF_TOPONODE_MEDIASTOP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_MEDIASOURCE_CONFIG, name: b"MF_SOURCE_READER_MEDIASOURCE_CONFIG\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_SOURCE, name: b"MF_TOPONODE_SOURCE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_PRESENTATION_DESCRIPTOR, name: b"MF_TOPONODE_PRESENTATION_DESCRIPTOR\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_D3DAWARE, name: b"MF_TOPONODE_D3DAWARE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_COMPRESSED, name: b"MF_MT_COMPRESSED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_STREAM_DESCRIPTOR, name: b"MF_TOPONODE_STREAM_DESCRIPTOR\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_ERRORCODE, name: b"MF_TOPONODE_ERRORCODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_SEQUENCE_ELEMENTID, name: b"MF_TOPONODE_SEQUENCE_ELEMENTID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSCODE_SKIP_METADATA_TRANSFER, name: b"MF_TRANSCODE_SKIP_METADATA_TRANSFER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_MFT_CONTEXT, name: b"MF_EVENT_MFT_CONTEXT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_FORWARD_CUSTOM_SEI, name: b"MF_MT_FORWARD_CUSTOM_SEI\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_FMPEG4, name: b"MFTranscodeContainerType_FMPEG4\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_CONNECT_METHOD, name: b"MF_TOPONODE_CONNECT_METHOD\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_OUTPUT_TYPES_Attributes, name: b"MFT_OUTPUT_TYPES_Attributes\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_IMAGE_LOSS_TOLERANT, name: b"MF_MT_IMAGE_LOSS_TOLERANT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SESSION_REMOTE_SOURCE_MODE, name: b"MF_SESSION_REMOTE_SOURCE_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_DEPTH_VALUE_UNIT, name: b"MF_MT_DEPTH_VALUE_UNIT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_AUDIO_NUM_CHANNELS, name: b"MF_MT_AUDIO_NUM_CHANNELS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_ARBITRARY_HEADER, name: b"MF_MT_ARBITRARY_HEADER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_DXVA_MODE, name: b"MF_TOPOLOGY_DXVA_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_LOCKED, name: b"MF_TOPONODE_LOCKED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TRANSCODE_QUALITYVSSPEED, name: b"MF_TRANSCODE_QUALITYVSSPEED\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_WORKQUEUE_ID, name: b"MF_TOPONODE_WORKQUEUE_ID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_CONTINUE_ON_CODEC_ERROR, name: b"MF_MEDIA_ENGINE_CONTINUE_ON_CODEC_ERROR\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_WORKQUEUE_MMCSS_CLASS, name: b"MF_TOPONODE_WORKQUEUE_MMCSS_CLASS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFTranscodeContainerType_MPEG2, name: b"MFTranscodeContainerType_MPEG2\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_DECRYPTOR, name: b"MF_TOPONODE_DECRYPTOR\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_EVENT_DO_THINNING, name: b"MF_EVENT_DO_THINNING\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_DISCARDABLE, name: b"MF_TOPONODE_DISCARDABLE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPOLOGY_HARDWARE_MODE, name: b"MF_TOPOLOGY_HARDWARE_MODE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_SOURCE_READER_DISABLE_DXVA, name: b"MF_SOURCE_READER_DISABLE_DXVA\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_FORWARD_CUSTOM_NALU, name: b"MF_MT_FORWARD_CUSTOM_NALU\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE10, name: b"MF_MEDIA_ENGINE_BROWSER_COMPATIBILITY_MODE_IE10\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_ERROR_MAJORTYPE, name: b"MF_TOPONODE_ERROR_MAJORTYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_SECURE, name: b"MF_MT_SECURE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_FIELDOFUSE_UNLOCK_Attribute, name: b"MFT_FIELDOFUSE_UNLOCK_Attribute\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_ERROR_SUBTYPE, name: b"MF_TOPONODE_ERROR_SUBTYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ROLE, name: b"MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ROLE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_MT_VIDEO_3D_LEFT_IS_BASE, name: b"MF_MT_VIDEO_3D_LEFT_IS_BASE\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_TOPONODE_WORKQUEUE_MMCSS_TASKID, name: b"MF_TOPONODE_WORKQUEUE_MMCSS_TASKID\0" as *const u8 as *const ::core::ffi::c_char },
        ]
    };
    let mut ret: *mut guid_def = ::core::ptr::null_mut::<guid_def>();
    if !guid.is_null() {
        ret = bsearch(
            guid as *const ::core::ffi::c_void,
            &raw const guid_defs as *const guid_def as *const ::core::ffi::c_void,
            390,
            ::core::mem::size_of::<guid_def>() as size_t,
            Some(
                debug_compare_guid
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        ) as *mut guid_def;
    }
    return if !ret.is_null() {
        wine_dbg_sprintf(b"%s\0" as *const u8 as *const ::core::ffi::c_char, (*ret).name)
    } else {
        wine_dbgstr_guid(guid)
    };
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_IRtwqAsyncResult: GUID = _GUID {
    Data1: 0xac6b7889 as ::core::ffi::c_uint,
    Data2: 0x740 as ::core::ffi::c_ushort,
    Data3: 0x4d51 as ::core::ffi::c_ushort,
    Data4: [
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IRtwqAsyncCallback: GUID = _GUID {
    Data1: 0xa27003cf as ::core::ffi::c_uint,
    Data2: 0x2354 as ::core::ffi::c_ushort,
    Data3: 0x4f2a as ::core::ffi::c_ushort,
    Data4: [
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_ID3D11Device: GUID = _GUID {
    Data1: 0xdb6f6ddb as ::core::ffi::c_uint,
    Data2: 0xac77 as ::core::ffi::c_ushort,
    Data3: 0x4e88 as ::core::ffi::c_ushort,
    Data4: [
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_ID3D12Fence: GUID = _GUID {
    Data1: 0xa753dcf as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc4d8 as ::core::ffi::c_ushort,
    Data3: 0x4b91 as ::core::ffi::c_ushort,
    Data4: [
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_ID3D12Device: GUID = _GUID {
    Data1: 0x189819f1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1db6 as ::core::ffi::c_ushort,
    Data3: 0x4b57 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x54 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IMFD3D12SynchronizationObjectCommands: GUID = _GUID {
    Data1: 0x9d0f835 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x92ff as ::core::ffi::c_ushort,
    Data3: 0x4e53 as ::core::ffi::c_ushort,
    Data4: [
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IMFD3D12SynchronizationObject: GUID = _GUID {
    Data1: 0x802302b0 as ::core::ffi::c_uint,
    Data2: 0x82de as ::core::ffi::c_ushort,
    Data3: 0x45e1 as ::core::ffi::c_ushort,
    Data4: [
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_D3D12_SYNCHRONIZATION_OBJECT: GUID = _GUID {
    Data1: 0x2a7c8d6a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x85a6 as ::core::ffi::c_ushort,
    Data3: 0x494d as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D_RESOURCE_VERSION: GUID = _GUID {
    Data1: 0x174f1e85 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xfe26 as ::core::ffi::c_ushort,
    Data3: 0x453d as ::core::ffi::c_ushort,
    Data4: [
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_CPU_READBACK: GUID = _GUID {
    Data1: 0x28ee9fe3 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xd481 as ::core::ffi::c_ushort,
    Data3: 0x46a6 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_TEXTURE_LAYOUT: GUID = _GUID {
    Data1: 0x97c85caa as ::core::ffi::c_uint,
    Data2: 0xbeb as ::core::ffi::c_ushort,
    Data3: 0x4ee1 as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET: GUID = _GUID {
    Data1: 0xeeac2585 as ::core::ffi::c_uint,
    Data2: 0x3430 as ::core::ffi::c_ushort,
    Data3: 0x498c as ::core::ffi::c_ushort,
    Data4: [
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL: GUID = _GUID {
    Data1: 0xb1138dc3 as ::core::ffi::c_uint,
    Data2: 0x1d5 as ::core::ffi::c_ushort,
    Data3: 0x4c14 as ::core::ffi::c_ushort,
    Data4: [
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS: GUID = _GUID {
    Data1: 0x82c85647 as ::core::ffi::c_uint,
    Data2: 0x5057 as ::core::ffi::c_ushort,
    Data3: 0x4960 as ::core::ffi::c_ushort,
    Data4: [
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE: GUID = _GUID {
    Data1: 0xba06bfac as ::core::ffi::c_uint,
    Data2: 0xffe3 as ::core::ffi::c_ushort,
    Data3: 0x474a as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_RESOURCE_FLAG_ALLOW_CROSS_ADAPTER: GUID = _GUID {
    Data1: 0xa6a1e439 as ::core::ffi::c_uint,
    Data2: 0x2f96 as ::core::ffi::c_ushort,
    Data3: 0x4ab5 as ::core::ffi::c_ushort,
    Data4: [
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_MT_D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS: GUID = _GUID {
    Data1: 0xa4940b2 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xcfd6 as ::core::ffi::c_ushort,
    Data3: 0x4738 as ::core::ffi::c_ushort,
    Data4: [
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_SA_D3D12_HEAP_FLAGS: GUID = _GUID {
    Data1: 0x496b3266 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xd28f as ::core::ffi::c_ushort,
    Data3: 0x4f8c as ::core::ffi::c_ushort,
    Data4: [
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_SA_D3D12_HEAP_TYPE: GUID = _GUID {
    Data1: 0x56f26a76 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xbbc1 as ::core::ffi::c_ushort,
    Data3: 0x4ce0 as ::core::ffi::c_ushort,
    Data4: [
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x68 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MF_SA_D3D12_CLEAR_VALUE: GUID = _GUID {
    Data1: 0x86ba9a39 as ::core::ffi::c_uint,
    Data2: 0x526 as ::core::ffi::c_ushort,
    Data3: 0x495d as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x54 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xec as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut local_handlers_section: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

static mut local_scheme_handlers: list = unsafe {
    list {
        next: &raw const local_scheme_handlers as *mut list,
        prev: &raw const local_scheme_handlers as *mut list,
    }
};

static mut local_bytestream_handlers: list = unsafe {
    list {
        next: &raw const local_bytestream_handlers as *mut list,
        prev: &raw const local_bytestream_handlers as *mut list,
    }
};

static mut local_mfts_section: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

static mut local_mfts: list = unsafe {
    list {
        next: &raw const local_mfts as *mut list,
        prev: &raw const local_mfts as *mut list,
    }
};

static mut transform_keyW: [WCHAR; 27] = unsafe {
    ::core::mem::transmute::<[u8; 54], [WCHAR; 27]>(
        *b"M\0e\0d\0i\0a\0F\0o\0u\0n\0d\0a\0t\0i\0o\0n\0\\\0T\0r\0a\0n\0s\0f\0o\0r\0m\0s\0\0\0",
    )
};

static mut categories_keyW: [WCHAR; 38] = unsafe {
    ::core::mem::transmute::<
        [u8; 76],
        [WCHAR; 38],
    >(
        *b"M\0e\0d\0i\0a\0F\0o\0u\0n\0d\0a\0t\0i\0o\0n\0\\\0T\0r\0a\0n\0s\0f\0o\0r\0m\0s\0\\\0C\0a\0t\0e\0g\0o\0r\0i\0e\0s\0\0\0",
    )
};

static mut guid_conv_table: [BYTE; 256] = [
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    1 as ::core::ffi::c_int as BYTE,
    2 as ::core::ffi::c_int as BYTE,
    3 as ::core::ffi::c_int as BYTE,
    4 as ::core::ffi::c_int as BYTE,
    5 as ::core::ffi::c_int as BYTE,
    6 as ::core::ffi::c_int as BYTE,
    7 as ::core::ffi::c_int as BYTE,
    8 as ::core::ffi::c_int as BYTE,
    9 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

static mut plugin_control: IMFPluginControl = unsafe {
    IMFPluginControl {
        lpVtbl: &raw const MFPluginControlVtbl,
    }
};

static mut async_create_file_results: list = unsafe {
    list {
        next: &raw const async_create_file_results as *mut list,
        prev: &raw const async_create_file_results as *mut list,
    }
};

static mut async_create_file_cs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

static mut shared_dm: shared_dxgi_manager = shared_dxgi_manager {
    manager: ::core::ptr::null::<IMFDXGIDeviceManager>() as *mut IMFDXGIDeviceManager,
    token: 0,
    locks: 0,
};

static mut shared_dm_cs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

// `COBJMACROS`, `INITGUID` and `EXTERN_GUID` are preprocessor flags with no value, so they are recorded as comments.
// `WINE_DEFAULT_DEBUG_CHANNEL(mfplat)` is a macro invocation, so it is recorded as a comment too.
// C macros: COBJMACROS (flag), INITGUID (flag), EXTERN_GUID -> DEFINE_GUID (flag)
// WINE_DEFAULT_DEBUG_CHANNEL(mfplat);

pub const ATTRIBUTES_STORE_MAGIC: u32 = 0x494d4641;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct local_handler_u_bytestream {
    pub extension: *mut WCHAR,
    pub mime: *mut WCHAR,
}

#[repr(C)]
pub union local_handler_u {
    pub scheme: *mut WCHAR,
    pub bytestream: local_handler_u_bytestream,
}

#[repr(C)]
pub struct local_handler {
    pub entry: list,
    pub u: local_handler_u,
    pub activate: *mut IMFActivate,
}

#[repr(C)]
pub struct mft_registration {
    pub entry: list,
    pub factory: *mut IClassFactory,
    pub clsid: CLSID,
    pub category: GUID,
    pub name: *mut WCHAR,
    pub flags: DWORD,
    pub input_types: *mut MFT_REGISTER_TYPE_INFO,
    pub input_types_count: UINT32,
    pub output_types: *mut MFT_REGISTER_TYPE_INFO,
    pub output_types_count: UINT32,
    pub local: BOOL,
}

#[repr(C)]
pub struct transform_activate {
    pub attributes: attributes,
    pub IMFActivate_iface: IMFActivate,
    pub factory: *mut IClassFactory,
    pub transform: *mut IMFTransform,
}

#[repr(C)]
pub struct system_clock {
    pub IMFClock_iface: IMFClock,
    pub refcount: LONG,
}

#[repr(C)]
pub struct system_time_source {
    pub IMFPresentationTimeSource_iface: IMFPresentationTimeSource,
    pub IMFClockStateSink_iface: IMFClockStateSink,
    pub refcount: LONG,
    pub state: MFCLOCK_STATE,
    pub clock: *mut IMFClock,
    pub start_offset: LONGLONG,
    pub system_time: LONGLONG,
    pub clock_time: LONGLONG,
    pub rate: f32,
    pub i_rate: i32,
    pub cs: CRITICAL_SECTION,
}

#[repr(C)]
pub struct guid_def {
    pub guid: *const GUID,
    pub name: *const i8,
}

#[repr(C)]
pub struct event_id {
    pub id: DWORD,
    pub name: *const i8,
}

#[repr(C)]
pub struct attributes_store_header {
    pub magic: DWORD,
    pub count: UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct attributes_store_item_u_subheader {
    pub size: DWORD,
    pub offset: DWORD,
}

#[repr(C)]
pub union attributes_store_item_u {
    pub f: f64,
    pub i32_0: UINT32,
    pub i64_0: UINT64,
    pub subheader: attributes_store_item_u_subheader,
}

#[repr(C)]
pub struct attributes_store_item {
    pub key: GUID,
    pub type_: QWORD,
    pub u: attributes_store_item_u,
}

#[repr(C)]
pub struct attr_serialize_context {
    pub buffer: *mut UINT8,
    pub ptr: *mut UINT8,
    pub size: UINT32,
}

#[repr(C)]
pub struct bytestream {
    pub attributes: attributes,
    pub IMFByteStream_iface: IMFByteStream,
    pub IMFGetService_iface: IMFGetService,
    pub read_callback: IRtwqAsyncCallback,
    pub write_callback: IRtwqAsyncCallback,
    pub stream: *mut IStream,
    pub hfile: HANDLE,
    pub position: u64,
    pub capabilities: DWORD,
    pub pending: list,
    pub cs: CRITICAL_SECTION,
}

pub const ASYNC_STREAM_OP_READ: i32 = 0;
pub const ASYNC_STREAM_OP_WRITE: i32 = 1;

#[repr(C)]
pub union async_stream_op_u {
    pub src: *const BYTE,
    pub dest: *mut BYTE,
}

#[repr(C)]
pub struct async_stream_op {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,
    pub u: async_stream_op_u,
    pub position: u64,
    pub requested_length: ULONG,
    pub actual_length: ULONG,
    pub caller: *mut IMFAsyncResult,
    pub entry: list,
    pub r#type: i32,
}

#[repr(C)]
pub struct bytestream_wrapper {
    pub IMFByteStreamCacheControl_iface: IMFByteStreamCacheControl,
    pub IMFByteStreamBuffering_iface: IMFByteStreamBuffering,
    pub IMFMediaEventGenerator_iface: IMFMediaEventGenerator,
    pub IMFByteStreamTimeSeek_iface: IMFByteStreamTimeSeek,
    pub IMFSampleOutputStream_iface: IMFSampleOutputStream,
    pub IPropertyStore_iface: IPropertyStore,
    pub IMFByteStream_iface: IMFByteStream,
    pub IMFAttributes_iface: IMFAttributes,
    pub refcount: LONG,

    pub cache_control: *mut IMFByteStreamCacheControl,
    pub stream_buffering: *mut IMFByteStreamBuffering,
    pub event_generator: *mut IMFMediaEventGenerator,
    pub time_seek: *mut IMFByteStreamTimeSeek,
    pub sample_output: *mut IMFSampleOutputStream,
    pub propstore: *mut IPropertyStore,
    pub stream: *mut IMFByteStream,
    pub attributes: *mut IMFAttributes,
    pub is_closed: BOOL,
}

pub const OBJECT_FROM_BYTESTREAM: i32 = 0;
pub const OBJECT_FROM_URL: i32 = 1;

#[repr(C)]
pub struct resolver_queued_result {
    pub entry: list,
    pub object: *mut IUnknown,
    pub obj_type: MF_OBJECT_TYPE,
    pub hr: HRESULT,
    pub inner_result: *mut IRtwqAsyncResult,
    pub origin: i32,
}

#[repr(C)]
pub union resolver_cancel_object_u {
    pub handler: *mut IUnknown,
    pub stream_handler: *mut IMFByteStreamHandler,
    pub scheme_handler: *mut IMFSchemeHandler,
}

#[repr(C)]
pub struct resolver_cancel_object {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,
    pub u: resolver_cancel_object_u,
    pub cancel_cookie: *mut IUnknown,
    pub origin: i32,
}

#[repr(C)]
pub struct source_resolver {
    pub IMFSourceResolver_iface: IMFSourceResolver,
    pub refcount: LONG,
    pub stream_callback: IRtwqAsyncCallback,
    pub url_callback: IRtwqAsyncCallback,
    pub cs: CRITICAL_SECTION,
    pub pending: list,
}

#[repr(C)]
pub struct media_event {
    pub attributes: attributes,
    pub IMFMediaEvent_iface: IMFMediaEvent,

    pub r#type: MediaEventType,
    pub extended_type: GUID,
    pub status: HRESULT,
    pub value: PROPVARIANT,
}

#[repr(C)]
pub struct event_queue {
    pub IMFMediaEventQueue_iface: IMFMediaEventQueue,
    pub refcount: LONG,

    pub cs: CRITICAL_SECTION,
    pub update_event: CONDITION_VARIABLE,
    pub events: list,
    pub is_shut_down: BOOL,
    pub notified: BOOL,
    pub subscriber: *mut IRtwqAsyncResult,
}

#[repr(C)]
pub struct queued_event {
    pub entry: list,
    pub event: *mut IMFMediaEvent,
}

#[repr(C)]
pub struct collection {
    pub IMFCollection_iface: IMFCollection,
    pub refcount: LONG,
    pub elements: *mut *mut IUnknown,
    pub capacity: usize,
    pub count: usize,
}

pub const CLOCK_CMD_START: i32 = 0;
pub const CLOCK_CMD_STOP: i32 = 1;
pub const CLOCK_CMD_PAUSE: i32 = 2;
pub const CLOCK_CMD_RESTART: i32 = 3;
pub const CLOCK_CMD_MAX: i32 = 4;

#[repr(C)]
pub struct async_create_file {
    pub IRtwqAsyncCallback_iface: IRtwqAsyncCallback,
    pub refcount: LONG,
    pub access_mode: MF_FILE_ACCESSMODE,
    pub open_mode: MF_FILE_OPENMODE,
    pub flags: MF_FILE_FLAGS,
    pub path: *mut WCHAR,
}

#[repr(C)]
pub struct async_create_file_result {
    pub entry: list,
    pub result: *mut IRtwqAsyncResult,
    pub stream: *mut IMFByteStream,
}

#[repr(C)]
pub struct property_store_value {
    pub key: PROPERTYKEY,
    pub value: PROPVARIANT,
}

#[repr(C)]
pub struct property_store {
    pub IPropertyStore_iface: IPropertyStore,
    pub refcount: LONG,
    pub cs: CRITICAL_SECTION,
    pub count: usize,
    pub capacity: usize,
    pub values: *mut property_store_value,
}

#[repr(C)]
pub struct d3d12_sync_object_release {
    pub entry: list,
    pub fence: *mut ID3D12Fence,
    pub event: HANDLE,
}

#[repr(C)]
pub struct d3d12_sync_object {
    pub IMFD3D12SynchronizationObject_iface: IMFD3D12SynchronizationObject,
    pub IMFD3D12SynchronizationObjectCommands_iface: IMFD3D12SynchronizationObjectCommands,
    pub async_release_iface: IRtwqAsyncCallback,
    pub refcount: LONG,
    pub cs: CRITICAL_SECTION,
    pub device: *mut ID3D12Device,
    pub ready_fence: *mut ID3D12Fence,
    pub generation: UINT64,
    pub release_wait: u32,
    pub final_release_event: HANDLE,
    pub release_freelist: list,
    pub release_freelist_cursor: *mut list,
}

#[repr(C)]
pub struct shared_dxgi_manager {
    pub manager: *mut IMFDXGIDeviceManager,
    pub token: u32,
    pub locks: u32,
}

pub const DXGI_DEVICE_HANDLE_FLAG_OPEN: i32 = 0x1;
pub const DXGI_DEVICE_HANDLE_FLAG_INVALID: i32 = 0x2;
pub const DXGI_DEVICE_HANDLE_FLAG_LOCKED: i32 = 0x4;

#[repr(C)]
pub struct dxgi_device_manager {
    pub IMFDXGIDeviceManager_iface: IMFDXGIDeviceManager,
    pub refcount: LONG,
    pub token: UINT,
    pub device: *mut IUnknown,

    pub handles: *mut u32,
    pub count: usize,
    pub capacity: usize,

    pub locks: u32,
    pub locking_tid: u32,

    pub cs: CRITICAL_SECTION,
    pub lock: CONDITION_VARIABLE,
}

unsafe extern "C" fn debugstr_reg_typeinfo(info: *const MFT_REGISTER_TYPE_INFO) -> *const i8 {
    return if !info.is_null() {
        wine_dbg_sprintf(
            b"%p{%s,%s}\0".as_ptr() as *const i8,
            info,
            debugstr_mf_guid(core::ptr::addr_of!((*info).guidMajorType)),
            debugstr_mf_guid(core::ptr::addr_of!((*info).guidSubtype)),
        )
    } else {
        wine_dbg_sprintf(b"%p\0".as_ptr() as *const i8, info)
    };
}

unsafe extern "C" fn system_time_source_update_clock_time(source: *mut system_time_source, system_time: LONGLONG) {
    let mut diff: LONGLONG = system_time - (*source).system_time;
    if (*source).i_rate != 0 {
        diff *= (*source).i_rate as LONGLONG;
    } else if (*source).rate != 1.0f32 {
        diff = (diff as f32 * (*source).rate) as LONGLONG;
    }
    (*source).clock_time += diff;
    (*source).system_time = system_time;
}

unsafe extern "C" fn impl_from_IMFPresentationTimeSource(
    iface: *mut IMFPresentationTimeSource,
) -> *mut system_time_source {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(system_time_source, IMFPresentationTimeSource_iface),
    ) as *mut system_time_source;
}

unsafe extern "C" fn impl_from_IMFClockStateSink(
    iface: *mut IMFClockStateSink,
) -> *mut system_time_source {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(system_time_source, IMFClockStateSink_iface),
    ) as *mut system_time_source;
}

unsafe extern "C" fn impl_from_IMFClock(iface: *mut IMFClock) -> *mut system_clock {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(system_clock, IMFClock_iface)) as *mut system_clock;
}

unsafe extern "C" fn impl_from_IMFActivate(iface: *mut IMFActivate) -> *mut transform_activate {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(transform_activate, IMFActivate_iface),
    ) as *mut transform_activate;
}

unsafe extern "system" fn transform_activate_QueryInterface(iface: *mut IMFActivate, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFActivate)) != 0
            || IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0
            || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = iface as *mut core::ffi::c_void;
        IMFActivate_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn transform_activate_AddRef(iface: *mut IMFActivate) -> ULONG {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*activate).attributes.ref_) as *mut i32,
    ) as ULONG;

    // TRACE("%p, refcount %lu.\n", iface, refcount);  (trace not present in the given C source)

    return refcount;
}
unsafe extern "system" fn transform_activate_Release(iface: *mut IMFActivate) -> ULONG {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*activate).attributes.ref_) as *mut i32,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        clear_attributes_object(core::ptr::addr_of_mut!((*activate).attributes));
        if !(*activate).factory.is_null() {
            IClassFactory_Release((*activate).factory);
        }
        if !(*activate).transform.is_null() {
            IMFTransform_Release((*activate).transform);
        }
        free(activate as *mut core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "system" fn transform_activate_GetItem(
    mut iface: *mut IMFActivate,
    key: *const GUID,
    mut value: *mut PROPVARIANT,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"transform_activate_GetItem\0" as *const u8 as *const core::ffi::c_char,
            213 as core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetItem(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_GetItemType(iface: *mut IMFActivate, key: *const GUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_GetItemType(core::ptr::addr_of_mut!((*activate).attributes), key, type_);
}
unsafe extern "system" fn transform_activate_CompareItem(
    mut iface: *mut IMFActivate,
    key: *const GUID,
    value: *const PROPVARIANT,
    mut result: *mut BOOL,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"transform_activate_CompareItem\0" as *const u8 as *const ::core::ffi::c_char,
            232 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return attributes_CompareItem(
        core::ptr::addr_of_mut!((*activate).attributes),
        key,
        value,
        result,
    );
}

unsafe extern "system" fn transform_activate_Compare(iface: *mut IMFActivate, theirs: *mut IMFAttributes, match_type: MF_ATTRIBUTES_MATCH_TYPE, ret: *mut BOOL) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_Compare(core::ptr::addr_of_mut!((*activate).attributes), theirs, match_type, ret);
}

unsafe extern "system" fn transform_activate_GetUINT32(
    iface: *mut IMFActivate,
    key: *const GUID,
    value: *mut UINT32,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const i8,
            b"transform_activate_GetUINT32\0".as_ptr() as *const i8,
            251 as i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT32(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_GetUINT64(iface: *mut IMFActivate, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"transform_activate_GetUINT64\0" as *const u8 as *const ::core::ffi::c_char,
            260 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT64(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_GetDouble(iface: *mut IMFActivate, key: REFGUID, value: *mut f64) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"transform_activate_GetDouble\0" as *const u8 as *const i8,
            269 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetDouble(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_GetGUID(iface: *mut IMFActivate, key: *const GUID, value: *mut GUID) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);

    return attributes_GetGUID(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_GetStringLength(iface: *mut IMFActivate, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), length);
    return attributes_GetStringLength(core::ptr::addr_of_mut!((*activate).attributes), key, length);
}

unsafe extern "system" fn transform_activate_GetString(
    mut iface: *mut IMFActivate,
    key: *const GUID,
    mut value: *mut WCHAR,
    mut size: UINT32,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"transform_activate_GetString\0" as *const u8 as *const ::core::ffi::c_char,
            297 as ::core::ffi::c_int,
            b"%p, %s, %p, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
            size,
            length,
        );
    }
    return attributes_GetString(core::ptr::addr_of_mut!((*activate).attributes), key, value, size, length);
}

unsafe extern "system" fn transform_activate_GetAllocatedString(
    iface: *mut IMFActivate,
    key: REFGUID,
    value: *mut *mut WCHAR,
    length: *mut UINT32,
) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"transform_activate_GetAllocatedString\0".as_ptr() as *const core::ffi::c_char,
            307 as core::ffi::c_int,
            b"%p, %s, %p, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
            length,
        );
    }
    return attributes_GetAllocatedString(&raw mut (*activate).attributes, key, value, length);
}

unsafe extern "system" fn transform_activate_GetBlobSize(
    iface: *mut IMFActivate,
    key: REFGUID,
    size: *mut UINT32,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"transform_activate_GetBlobSize\0" as *const u8 as *const ::core::ffi::c_char,
            316 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            size,
        );
    }
    return attributes_GetBlobSize(&raw mut (*activate).attributes, key, size);
}

unsafe extern "system" fn transform_activate_GetBlob(
    iface: *mut IMFActivate,
    key: *const GUID,
    buf: *mut UINT8,
    bufsize: UINT32,
    blobsize: *mut UINT32,
) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_GetBlob(core::ptr::addr_of_mut!((*activate).attributes), key, buf, bufsize, blobsize);
}

unsafe extern "system" fn transform_activate_GetAllocatedBlob(iface: *mut IMFActivate, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_GetAllocatedBlob(core::ptr::addr_of_mut!((*activate).attributes), key, buf, size);
}

unsafe extern "system" fn transform_activate_GetUnknown(
    iface: *mut IMFActivate,
    key: *const GUID,
    riid: *const IID,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"transform_activate_GetUnknown\0".as_ptr() as *const core::ffi::c_char,
            344 as core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_guid(riid as *const _GUID),
            out,
        );
    }
    return attributes_GetUnknown(core::ptr::addr_of_mut!((*activate).attributes), key, riid, out);
}

unsafe extern "system" fn transform_activate_SetItem(iface: *mut IMFActivate, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"transform_activate_SetItem\0" as *const u8 as *const i8,
            353 as i32,
            b"%p, %s, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
        );
    }
    return attributes_SetItem(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_DeleteItem(iface: *mut IMFActivate, key: *const GUID) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);

    TRACE!("%p, %s.\n", iface, debugstr_attr(key));

    return attributes_DeleteItem(&raw mut (*activate).attributes, key);
}

unsafe extern "system" fn transform_activate_DeleteAllItems(iface: *mut IMFActivate) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_DeleteAllItems(core::ptr::addr_of_mut!((*activate).attributes));
}

unsafe extern "system" fn transform_activate_SetUINT32(iface: *mut IMFActivate, key: *const GUID, value: UINT32) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const core::ffi::c_char,
            b"transform_activate_SetUINT32\0" as *const u8 as *const core::ffi::c_char,
            380 as core::ffi::c_int,
            b"%p, %s, %u.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_SetUINT32(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_SetUINT64(
    mut iface: *mut IMFActivate,
    key: *const GUID,
    mut value: UINT64,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"transform_activate_SetUINT64\0" as *const u8 as *const ::core::ffi::c_char,
            389 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return attributes_SetUINT64(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_SetDouble(iface: *mut IMFActivate, key: *const GUID, value: f64) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);
    TRACE!("%p, %s, %f.\n", iface, debugstr_attr(key), value);
    return attributes_SetDouble(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_SetGUID(iface: *mut IMFActivate, key: REFGUID, value: REFGUID) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_SetGUID(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}
unsafe extern "system" fn transform_activate_SetString(iface: *mut IMFActivate, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"transform_activate_SetString\0".as_ptr() as *const i8,
            416 as core::ffi::c_int,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_w(value),
        );
    }
    return attributes_SetString(core::ptr::addr_of_mut!((*activate).attributes), key, value);
}

unsafe extern "system" fn transform_activate_SetBlob(iface: *mut IMFActivate, key: REFGUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_SetBlob(core::ptr::addr_of_mut!((*activate).attributes), key, buf, size);
}

unsafe extern "system" fn transform_activate_SetUnknown(iface: *mut IMFActivate, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"transform_activate_SetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            434 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            unknown,
        );
    }
    return attributes_SetUnknown(core::ptr::addr_of_mut!((*activate).attributes), key, unknown);
}

unsafe extern "system" fn transform_activate_LockStore(iface: *mut IMFActivate) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_LockStore(core::ptr::addr_of_mut!((*activate).attributes));
}

unsafe extern "system" fn transform_activate_UnlockStore(iface: *mut IMFActivate) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_UnlockStore(core::ptr::addr_of_mut!((*activate).attributes));
}
unsafe extern "system" fn transform_activate_GetCount(
    mut iface: *mut IMFActivate,
    mut count: *mut UINT32,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl as i32) as u32,
            (__dbch as *mut core::ffi::c_void).cast(),
            b"dll/win32/mfplat/main.c\0"
                .as_ptr() as *const i8,
            b"transform_activate_GetCount\0".as_ptr() as *const i8,
            461i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface as *mut core::ffi::c_void,
            count as *mut core::ffi::c_void,
        );
    }
    return attributes_GetCount(core::ptr::addr_of_mut!((*activate).attributes), count);
}

unsafe extern "system" fn transform_activate_GetItemByIndex(
    iface: *mut IMFActivate,
    index: UINT32,
    key: *mut GUID,
    value: *mut PROPVARIANT,
) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_GetItemByIndex(
        core::ptr::addr_of_mut!((*activate).attributes),
        index,
        key,
        value,
    );
}

unsafe extern "system" fn transform_activate_CopyAllItems(iface: *mut IMFActivate, dest: *mut IMFAttributes) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);

    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*activate).attributes), dest);
}

unsafe extern "system" fn transform_activate_ActivateObject(iface: *mut IMFActivate, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let activate: *mut transform_activate = impl_from_IMFActivate(iface);
    let mut clsid: CLSID = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(riid), obj);

    EnterCriticalSection(core::ptr::addr_of_mut!((*activate).attributes.cs));

    if (*activate).transform.is_null() {
        if !(*activate).factory.is_null() {
            hr = IClassFactory_CreateInstance(
                (*activate).factory,
                core::ptr::null_mut::<IUnknown>(),
                core::ptr::addr_of!(IID_IMFTransform),
                core::ptr::addr_of_mut!((*activate).transform) as *mut *mut core::ffi::c_void,
            );
            if hr < 0 {
                hr = MF_E_INVALIDREQUEST;
            }
        } else {
            hr = attributes_GetGUID(
                core::ptr::addr_of_mut!((*activate).attributes),
                core::ptr::addr_of!(MFT_TRANSFORM_CLSID_Attribute),
                core::ptr::addr_of_mut!(clsid),
            );
            if hr >= 0 {
                hr = CoCreateInstance(
                    core::ptr::addr_of_mut!(clsid),
                    core::ptr::null_mut::<IUnknown>(),
                    CLSCTX_INPROC_SERVER as DWORD,
                    core::ptr::addr_of!(IID_IMFTransform),
                    core::ptr::addr_of_mut!((*activate).transform) as *mut LPVOID,
                );
                if hr < 0 {
                    hr = MF_E_INVALIDREQUEST;
                }
            }
        }
    }

    if !(*activate).transform.is_null() {
        hr = IMFTransform_QueryInterface((*activate).transform, riid, obj);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*activate).attributes.cs));

    return hr;
}
unsafe extern "system" fn transform_activate_ShutdownObject(iface: *mut IMFActivate) -> HRESULT {
    let mut activate: *mut transform_activate = impl_from_IMFActivate(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"transform_activate_ShutdownObject\0" as *const u8 as *const ::core::ffi::c_char,
            530 as ::core::ffi::c_int,
            b"%p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*activate).attributes.cs));

    if !(*activate).transform.is_null() {
        IMFTransform_Release((*activate).transform);
        (*activate).transform = core::ptr::null_mut();
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*activate).attributes.cs));

    return S_OK;
}
unsafe extern "system" fn transform_activate_DetachObject(iface: *mut IMFActivate) -> HRESULT {
    // TRACE("%p.\n", iface);
    E_NOTIMPL
}

unsafe extern "C" fn create_transform_activate(factory: *mut IClassFactory, activate: *mut *mut IMFActivate) -> HRESULT {
    let mut object: *mut transform_activate = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc((1 as size_t) as _, (core::mem::size_of::<transform_activate>() as size_t) as _) as *mut transform_activate;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(core::ptr::addr_of_mut!((*object).attributes), 0 as UINT32);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    (*object).IMFActivate_iface.lpVtbl = core::ptr::addr_of!(transform_activate_vtbl);
    (*object).factory = factory;
    if !(*object).factory.is_null() {
        IClassFactory_AddRef((*object).factory);
    }

    *activate = core::ptr::addr_of_mut!((*object).IMFActivate_iface);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateTransformActivate(
    activate: *mut *mut IMFActivate,
) -> HRESULT {
    // TRACE("%p.\n", activate);
    create_transform_activate(core::ptr::null_mut(), activate)
}

unsafe extern "C" fn guid_to_string(str: *mut WCHAR, guid: *const GUID) -> *mut WCHAR {
    swprintf(
        str,
        39 as usize,
        RUSTCONV_WSTR_8949E72AB0EA.as_ptr(),
        (*guid).Data1,
        (*guid).Data2 as core::ffi::c_int,
        (*guid).Data3 as core::ffi::c_int,
        (*guid).Data4[0 as usize] as core::ffi::c_int,
        (*guid).Data4[1 as usize] as core::ffi::c_int,
        (*guid).Data4[2 as usize] as core::ffi::c_int,
        (*guid).Data4[3 as usize] as core::ffi::c_int,
        (*guid).Data4[4 as usize] as core::ffi::c_int,
        (*guid).Data4[5 as usize] as core::ffi::c_int,
        (*guid).Data4[6 as usize] as core::ffi::c_int,
        (*guid).Data4[7 as usize] as core::ffi::c_int,
    );
    return str;
}

unsafe extern "C" fn is_valid_hex(c: WCHAR) -> BOOL {
    if !(((c as i32) >= ('0' as i32)) && ((c as i32) <= ('9' as i32))
        || ((c as i32) >= ('a' as i32)) && ((c as i32) <= ('f' as i32))
        || ((c as i32) >= ('A' as i32)) && ((c as i32) <= ('F' as i32)))
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn GUIDFromString(mut s: LPCWSTR, mut id: *mut GUID) -> BOOL {
    let mut i: i32 = 0;

    /* in form XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX */

    (*id).Data1 = 0;
    i = 0;
    while i < 8 {
        if is_valid_hex(*s.offset(i as isize)) == 0 {
            return (FALSE) as BOOL;
        }
        (*id).Data1 = ((*id).Data1 << 4) | guid_conv_table[*s.offset(i as isize) as usize] as u32;
        i += 1;
    }
    if *s.offset(8 as isize) != '-' as u16 {
        return (FALSE) as BOOL;
    }

    (*id).Data2 = 0;
    i = 9;
    while i < 13 {
        if is_valid_hex(*s.offset(i as isize)) == 0 {
            return (FALSE) as BOOL;
        }
        (*id).Data2 = ((*id).Data2 << 4) | guid_conv_table[*s.offset(i as isize) as usize] as u16;
        i += 1;
    }
    if *s.offset(13 as isize) != '-' as u16 {
        return (FALSE) as BOOL;
    }

    (*id).Data3 = 0;
    i = 14;
    while i < 18 {
        if is_valid_hex(*s.offset(i as isize)) == 0 {
            return (FALSE) as BOOL;
        }
        (*id).Data3 = ((*id).Data3 << 4) | guid_conv_table[*s.offset(i as isize) as usize] as u16;
        i += 1;
    }
    if *s.offset(18 as isize) != '-' as u16 {
        return (FALSE) as BOOL;
    }

    i = 19;
    while i < 36 {
        i += 2;
        // placeholder replaced below
        i -= 2;
        if i == 23 {
            if *s.offset(i as isize) != '-' as u16 {
                return (FALSE) as BOOL;
            }
            i += 1;
        }
        if is_valid_hex(*s.offset(i as isize)) == 0
            || is_valid_hex(*s.offset((i + 1) as isize)) == 0
        {
            return (FALSE) as BOOL;
        }
        (*id).Data4[((i - 19) / 2) as usize] =
            (((guid_conv_table[*s.offset(i as isize) as usize] as i32) << 4)
                | guid_conv_table[*s.offset((i + 1) as isize) as usize] as i32) as u8;
        i += 2;
    }

    if *s.offset(36 as isize) == 0 {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn register_transform(clsid: *const CLSID, name: *const WCHAR, mut flags: UINT32, cinput: UINT32, input_types: *const MFT_REGISTER_TYPE_INFO, coutput: UINT32, output_types: *const MFT_REGISTER_TYPE_INFO, attributes: *mut IMFAttributes) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut hclsid: HKEY = core::ptr::null_mut();
    let mut buffer: [WCHAR; 64] = [0 as WCHAR; 64];
    let mut str: [WCHAR; 250] = [0 as WCHAR; 250];
    let mut blob: *mut UINT8;
    let mut size: UINT32 = core::mem::zeroed();
    let mut ret: DWORD;

    guid_to_string(core::ptr::addr_of_mut!(buffer) as *mut WCHAR, clsid);
    swprintf(
        core::ptr::addr_of_mut!(str) as *mut WCHAR,
        (ARRAY_SIZE!(str)) as _,
        RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
        core::ptr::addr_of!(transform_keyW) as *const WCHAR,
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
    );

    ret = RegCreateKeyW(
        HKEY_CLASSES_ROOT,
        core::ptr::addr_of_mut!(str) as *const WCHAR,
        core::ptr::addr_of_mut!(hclsid),
    ) as DWORD;
    if ret != 0 {
        hr = HRESULT_FROM_WIN32!(ret);
    }

    if hr >= 0 {
        size = ((lstrlenW(name) + 1) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as UINT32;
        ret = RegSetValueExW(
            hclsid,
            core::ptr::null::<WCHAR>(),
            0,
            REG_SZ,
            name as *mut BYTE,
            size as DWORD,
        ) as DWORD;
        if ret != 0 {
            hr = HRESULT_FROM_WIN32!(ret);
        }
    }

    if hr >= 0 && cinput != 0 && !input_types.is_null() {
        size = (cinput as usize)
            .wrapping_mul(core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as usize) as UINT32;
        ret = RegSetValueExW(
            hclsid,
            RUSTCONV_WSTR_5D5100CABDC1.as_ptr(),
            0,
            REG_BINARY,
            input_types as *mut BYTE,
            size as DWORD,
        ) as DWORD;
        if ret != 0 {
            hr = HRESULT_FROM_WIN32!(ret);
        }
    }

    if hr >= 0 && coutput != 0 && !output_types.is_null() {
        size = (coutput as usize)
            .wrapping_mul(core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as usize) as UINT32;
        ret = RegSetValueExW(
            hclsid,
            RUSTCONV_WSTR_B17B36FA88F7.as_ptr(),
            0,
            REG_BINARY,
            output_types as *mut BYTE,
            size as DWORD,
        ) as DWORD;
        if ret != 0 {
            hr = HRESULT_FROM_WIN32!(ret);
        }
    }

    if hr >= 0 && !attributes.is_null() {
        hr = MFGetAttributesAsBlobSize(attributes, core::ptr::addr_of_mut!(size));
        if hr >= 0 {
            blob = malloc(size as usize) as *mut UINT8;
            if !blob.is_null() {
                hr = MFGetAttributesAsBlob(attributes, blob, size as UINT);
                if hr >= 0 {
                    ret = RegSetValueExW(
                        hclsid,
                        RUSTCONV_WSTR_A38E93E48CB3.as_ptr(),
                        0,
                        REG_BINARY,
                        blob,
                        size as DWORD,
                    ) as DWORD;
                    if ret != 0 {
                        hr = HRESULT_FROM_WIN32!(ret);
                    }
                }
                free(blob as *mut core::ffi::c_void);
            } else {
                hr = E_OUTOFMEMORY;
            }
        }
    }

    if hr >= 0 && flags != 0 {
        ret = RegSetValueExW(
            hclsid,
            RUSTCONV_WSTR_E89EC7379787.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(flags) as *mut BYTE,
            core::mem::size_of::<UINT32>() as DWORD,
        ) as DWORD;
        if ret != 0 {
            hr = HRESULT_FROM_WIN32!(ret);
        }
    }

    RegCloseKey(hclsid);
    return hr;
}

unsafe extern "C" fn register_category(clsid: *mut CLSID, category: *mut GUID) -> HRESULT {
    let mut htmp1: HKEY = core::ptr::null_mut();
    let mut guid1: [WCHAR; 64] = [0; 64];
    let mut guid2: [WCHAR; 64] = [0; 64];
    let mut str: [WCHAR; 350] = [0; 350];

    guid_to_string(core::ptr::addr_of_mut!(guid1) as *mut WCHAR, category);
    guid_to_string(core::ptr::addr_of_mut!(guid2) as *mut WCHAR, clsid);

    swprintf(
        core::ptr::addr_of_mut!(str) as *mut WCHAR,
        ARRAY_SIZE!(str) as _,
        RUSTCONV_WSTR_714123A8AF14.as_ptr(),
        core::ptr::addr_of!(categories_keyW) as *const WCHAR,
        core::ptr::addr_of_mut!(guid1) as *mut WCHAR,
        core::ptr::addr_of_mut!(guid2) as *mut WCHAR,
    );

    if RegCreateKeyW(
        HKEY_CLASSES_ROOT,
        core::ptr::addr_of_mut!(str) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(htmp1),
    ) != 0
    {
        return E_FAIL;
    }

    RegCloseKey(htmp1);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFTRegister(
    mut clsid: CLSID,
    mut category: GUID,
    mut name: LPWSTR,
    mut flags: UINT32,
    mut cinput: UINT32,
    mut input_types: *mut MFT_REGISTER_TYPE_INFO,
    mut coutput: UINT32,
    mut output_types: *mut MFT_REGISTER_TYPE_INFO,
    mut attributes: *mut IMFAttributes,
) -> HRESULT {
    let mut hr: HRESULT;

    hr = register_transform(
        core::ptr::addr_of_mut!(clsid),
        name as *const WCHAR,
        flags,
        cinput,
        input_types,
        coutput,
        output_types,
        attributes,
    );
    if hr < 0 {
        ERR!("Failed to write register transform\n");
    }

    if hr >= 0 {
        hr = register_category(
            core::ptr::addr_of_mut!(clsid),
            core::ptr::addr_of_mut!(category),
        );
    }

    return hr;
}

unsafe extern "C" fn release_mft_registration(mut mft: *mut mft_registration) {
    if !(*mft).factory.is_null() {
        IClassFactory_Release((*mft).factory);
    }
    free((*mft).name as *mut ::core::ffi::c_void);
    free((*mft).input_types as *mut ::core::ffi::c_void);
    free((*mft).output_types as *mut ::core::ffi::c_void);
    free(mft as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn mft_register_local(
    mut factory: *mut IClassFactory,
    clsid: *const CLSID,
    category: *const GUID,
    mut name: LPCWSTR,
    mut flags: UINT32,
    mut input_count: UINT32,
    mut input_types: *const MFT_REGISTER_TYPE_INFO,
    mut output_count: UINT32,
    mut output_types: *const MFT_REGISTER_TYPE_INFO,
) -> HRESULT {
    let mut mft: *mut mft_registration = core::ptr::null_mut::<mft_registration>();
    let mut cur: *mut mft_registration = core::ptr::null_mut::<mft_registration>();
    let mut unreg_mft: *mut mft_registration = core::ptr::null_mut::<mft_registration>();
    let mut hr: HRESULT = S_OK;

    if factory.is_null() && clsid.is_null() {
        WARN!("Can't register without factory or CLSID.\n");
        return E_FAIL;
    }

    mft = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<mft_registration>() as size_t) as _,
    ) as *mut mft_registration;
    if mft.is_null() {
        return E_OUTOFMEMORY;
    }

    (*mft).factory = factory;
    if !(*mft).factory.is_null() {
        IClassFactory_AddRef((*mft).factory);
    }
    if !clsid.is_null() {
        (*mft).clsid = *clsid;
    }
    (*mft).category = *category;
    if flags
        & (MFT_ENUM_FLAG_SYNCMFT | MFT_ENUM_FLAG_ASYNCMFT | MFT_ENUM_FLAG_HARDWARE) as UINT32
        == 0
    {
        flags |= MFT_ENUM_FLAG_SYNCMFT as UINT32;
    }
    (*mft).flags = flags as DWORD;
    (*mft).local = TRUE as BOOL;

    'failed: {
        if !name.is_null() && {
            (*mft).name = wcsdup(name as *const wchar_t) as *mut WCHAR;
            (*mft).name.is_null()
        } {
            hr = E_OUTOFMEMORY;
            break 'failed;
        }

        if input_count != 0 && !input_types.is_null() {
            (*mft).input_types_count = input_count;
            (*mft).input_types = calloc(
                ((*mft).input_types_count as size_t) as _,
                (core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as size_t) as _,
            ) as *mut MFT_REGISTER_TYPE_INFO;
            if (*mft).input_types.is_null() {
                hr = E_OUTOFMEMORY;
                break 'failed;
            }
            memcpy(
                (*mft).input_types as *mut core::ffi::c_void,
                input_types as *const core::ffi::c_void,
                ((*mft).input_types_count as size_t)
                    .wrapping_mul(core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as size_t),
            );
        }

        if output_count != 0 && !output_types.is_null() {
            (*mft).output_types_count = output_count;
            (*mft).output_types = calloc(
                ((*mft).output_types_count as size_t) as _,
                (core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as size_t) as _,
            ) as *mut MFT_REGISTER_TYPE_INFO;
            if (*mft).output_types.is_null() {
                hr = E_OUTOFMEMORY;
                break 'failed;
            }
            memcpy(
                (*mft).output_types as *mut core::ffi::c_void,
                output_types as *const core::ffi::c_void,
                ((*mft).output_types_count as size_t)
                    .wrapping_mul(core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as size_t),
            );
        }

        EnterCriticalSection(core::ptr::addr_of_mut!(local_mfts_section));

        cur = (local_mfts.next as *mut u8).offset(
            -(core::mem::offset_of!(mft_registration, entry) as isize),
        ) as *mut mft_registration;
        while core::ptr::addr_of_mut!((*cur).entry) != core::ptr::addr_of_mut!(local_mfts) {
            if (*cur).factory == factory {
                unreg_mft = cur;
                list_remove(core::ptr::addr_of_mut!((*cur).entry));
                break;
            } else {
                cur = ((*cur).entry.next as *mut u8).offset(
                    -(core::mem::offset_of!(mft_registration, entry) as isize),
                ) as *mut mft_registration;
            }
        }
        list_add_tail(
            core::ptr::addr_of_mut!(local_mfts),
            core::ptr::addr_of_mut!((*mft).entry),
        );

        LeaveCriticalSection(core::ptr::addr_of_mut!(local_mfts_section));

        if !unreg_mft.is_null() {
            release_mft_registration(unreg_mft);
        }
    }

    if hr < 0 {
        release_mft_registration(mft);
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFTRegisterLocal(factory: *mut IClassFactory, category: *const GUID, name: LPCWSTR, flags: UINT32, input_count: UINT32, input_types: *const MFT_REGISTER_TYPE_INFO, output_count: UINT32, output_types: *const MFT_REGISTER_TYPE_INFO) -> HRESULT {
    return mft_register_local(factory, core::ptr::null::<CLSID>(), category, name, flags, input_count, input_types, output_count, output_types);
}

#[no_mangle]
pub unsafe extern "system" fn MFTRegisterLocalByCLSID(
    clsid: REFCLSID,
    category: REFGUID,
    name: LPCWSTR,
    flags: UINT32,
    input_count: UINT32,
    input_types: *const MFT_REGISTER_TYPE_INFO,
    output_count: UINT32,
    output_types: *const MFT_REGISTER_TYPE_INFO,
) -> HRESULT {
    // TRACE("%s, %s, %s, %#x, %u, %p, %u, %p.\n", debugstr_guid(clsid), debugstr_guid(category), debugstr_w(name), flags, input_count, input_types, output_count, output_types);
    mft_register_local(core::ptr::null_mut(), clsid, category, name, flags, input_count, input_types, output_count, output_types)
}

unsafe extern "C" fn mft_unregister_local(factory: *mut IClassFactory, clsid: *const CLSID) -> HRESULT {
    let mut cur: *mut mft_registration = core::ptr::null_mut::<mft_registration>();
    let mut cur2: *mut mft_registration = core::ptr::null_mut::<mft_registration>();
    let unregister_all: BOOL = (factory.is_null() && clsid.is_null()) as i32;
    let mut unreg: list = list {
        next: core::ptr::null_mut::<list>(),
        prev: core::ptr::null_mut::<list>(),
    };

    list_init(core::ptr::addr_of_mut!(unreg));

    EnterCriticalSection(core::ptr::addr_of_mut!(local_mfts_section));

    cur = (local_mfts.next as *mut u8)
        .sub(core::mem::offset_of!(mft_registration, entry)) as *mut mft_registration;
    cur2 = ((*cur).entry.next as *mut u8)
        .sub(core::mem::offset_of!(mft_registration, entry)) as *mut mft_registration;
    while core::ptr::addr_of_mut!((*cur).entry) != core::ptr::addr_of_mut!(local_mfts) {
        if unregister_all == 0 {
            if (!factory.is_null() && (*cur).factory == factory)
                || IsEqualCLSID!(core::ptr::addr_of!((*cur).clsid), clsid) != 0
            {
                list_remove(core::ptr::addr_of_mut!((*cur).entry));
                list_add_tail(
                    core::ptr::addr_of_mut!(unreg),
                    core::ptr::addr_of_mut!((*cur).entry),
                );
                break;
            }
        } else {
            list_remove(core::ptr::addr_of_mut!((*cur).entry));
            list_add_tail(
                core::ptr::addr_of_mut!(unreg),
                core::ptr::addr_of_mut!((*cur).entry),
            );
        }
        cur = cur2;
        cur2 = ((*cur).entry.next as *mut u8)
            .sub(core::mem::offset_of!(mft_registration, entry)) as *mut mft_registration;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(local_mfts_section));

    if unregister_all == 0 && list_empty(core::ptr::addr_of_mut!(unreg)) != 0 {
        return HRESULT_FROM_WIN32!(ERROR_NOT_FOUND);
    }

    cur = (unreg.next as *mut u8)
        .sub(core::mem::offset_of!(mft_registration, entry)) as *mut mft_registration;
    cur2 = ((*cur).entry.next as *mut u8)
        .sub(core::mem::offset_of!(mft_registration, entry)) as *mut mft_registration;
    while core::ptr::addr_of_mut!((*cur).entry) != core::ptr::addr_of_mut!(unreg) {
        list_remove(core::ptr::addr_of_mut!((*cur).entry));
        release_mft_registration(cur);
        cur = cur2;
        cur2 = ((*cur).entry.next as *mut u8)
            .sub(core::mem::offset_of!(mft_registration, entry)) as *mut mft_registration;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFTUnregisterLocalByCLSID(mut clsid: CLSID) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFTUnregisterLocalByCLSID\0" as *const u8 as *const ::core::ffi::c_char,
            961 as ::core::ffi::c_int,
            b"%s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_guid(core::ptr::addr_of_mut!(clsid)),
        );
    }
    return mft_unregister_local(
        core::ptr::null_mut::<IClassFactory>(),
        core::ptr::addr_of_mut!(clsid),
    );
}

#[no_mangle]
pub unsafe extern "system" fn MFTUnregisterLocal(factory: *mut IClassFactory) -> HRESULT {
    return mft_unregister_local(factory, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn MFGetSystemTime() -> MFTIME {
    static mut frequency: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut counter: LARGE_INTEGER = unsafe { core::mem::zeroed() };

    if !(frequency.QuadPart == 0) {
    } else {
        QueryPerformanceFrequency(core::ptr::addr_of_mut!(frequency));
    }
    QueryPerformanceCounter(core::ptr::addr_of_mut!(counter));

    return counter.QuadPart * 10000000 / frequency.QuadPart;
}

unsafe extern "C" fn mft_is_type_info_match(
    mft: *mut mft_registration,
    category: *const GUID,
    flags: UINT32,
    plugin_control_local: *mut IMFPluginControl,
    input_type: *const MFT_REGISTER_TYPE_INFO,
    output_type: *const MFT_REGISTER_TYPE_INFO,
) -> BOOL {
    let mut matching: BOOL = TRUE as BOOL;
    let mut model: DWORD;
    let mut i: i32;

    if IsEqualGUID!(category, core::ptr::addr_of_mut!((*mft).category)) == false {
        return FALSE as BOOL;
    }

    /* Default model is synchronous. */
    model = (*mft).flags
        & ((MFT_ENUM_FLAG_SYNCMFT as DWORD)
            | (MFT_ENUM_FLAG_ASYNCMFT as DWORD)
            | (MFT_ENUM_FLAG_HARDWARE as DWORD));
    if model == 0 {
        model = MFT_ENUM_FLAG_SYNCMFT as DWORD;
    }
    if (model
        & flags as DWORD
        & ((MFT_ENUM_FLAG_SYNCMFT as DWORD)
            | (MFT_ENUM_FLAG_ASYNCMFT as DWORD)
            | (MFT_ENUM_FLAG_HARDWARE as DWORD)))
        == 0
    {
        return FALSE as BOOL;
    }

    /* These flags should be explicitly enabled. */
    if ((*mft).flags
        & !flags as DWORD
        & ((MFT_ENUM_FLAG_FIELDOFUSE as DWORD) | (MFT_ENUM_FLAG_TRANSCODE_ONLY as DWORD)))
        != 0
    {
        return FALSE as BOOL;
    }

    if (flags & (MFT_ENUM_FLAG_SORTANDFILTER as UINT32)) != 0
        && (*mft).factory.is_null()
        && !plugin_control_local.is_null()
        && IMFPluginControl_IsDisabled(
            plugin_control_local,
            MF_Plugin_Type_MFT as DWORD,
            core::ptr::addr_of_mut!((*mft).clsid),
        ) == S_OK
    {
        return FALSE as BOOL;
    }

    if !input_type.is_null() {
        i = 0 as i32;
        matching = FALSE as BOOL;
        while !input_type.is_null() && (i as UINT32) < (*mft).input_types_count {
            if memcmp(
                (*mft).input_types.offset(i as isize) as *mut MFT_REGISTER_TYPE_INFO
                    as *const ::core::ffi::c_void,
                input_type as *const ::core::ffi::c_void,
                core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as usize,
            ) == 0
            {
                matching = TRUE as BOOL;
                break;
            } else {
                i += 1;
            }
        }
    }

    if !output_type.is_null() && matching != 0 {
        i = 0 as i32;
        matching = FALSE as BOOL;
        while (i as UINT32) < (*mft).output_types_count {
            if memcmp(
                (*mft).output_types.offset(i as isize) as *mut MFT_REGISTER_TYPE_INFO
                    as *const ::core::ffi::c_void,
                output_type as *const ::core::ffi::c_void,
                core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as usize,
            ) == 0
            {
                matching = TRUE as BOOL;
                break;
            } else {
                i += 1;
            }
        }
    }

    return matching;
}

unsafe extern "C" fn mft_get_reg_type_info_internal(clsidW: *const WCHAR, typeW: *const WCHAR, type_: *mut *mut MFT_REGISTER_TYPE_INFO, count: *mut UINT32) {
    let mut htransform: HKEY = core::ptr::null_mut();
    let mut hfilter: HKEY = core::ptr::null_mut();
    let mut reg_type: DWORD = 0;
    let mut size: DWORD = 0;

    *type_ = core::ptr::null_mut();
    *count = 0;

    if RegOpenKeyW(HKEY_CLASSES_ROOT, core::ptr::addr_of!(transform_keyW) as LPCWSTR, &mut htransform) != 0 {
        return;
    }

    if RegOpenKeyW(htransform, clsidW as LPCWSTR, &mut hfilter) != 0 {
        RegCloseKey(htransform);
        return;
    }

    'out: {
        if RegQueryValueExW(hfilter, typeW as LPCWSTR, core::ptr::null_mut::<DWORD>(), &mut reg_type, core::ptr::null_mut::<BYTE>(), &mut size) != 0 {
            break 'out;
        }

        if reg_type != REG_BINARY {
            break 'out;
        }

        if size == 0 || (size as usize) % core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() != 0 {
            break 'out;
        }

        *type_ = malloc(size as size_t) as *mut MFT_REGISTER_TYPE_INFO;
        if (*type_).is_null() {
            break 'out;
        }

        *count = ((size as usize) / core::mem::size_of::<MFT_REGISTER_TYPE_INFO>()) as UINT32;

        if RegQueryValueExW(hfilter, typeW as LPCWSTR, core::ptr::null_mut::<DWORD>(), &mut reg_type, *type_ as *mut BYTE, &mut size) != 0 {
            free(*type_ as *mut core::ffi::c_void);
            *type_ = core::ptr::null_mut();
            *count = 0;
        }
    }

    RegCloseKey(hfilter);
    RegCloseKey(htransform);
}

unsafe extern "C" fn mft_get_reg_flags(
    clsidW: *const WCHAR,
    nameW: *const WCHAR,
    flags: *mut DWORD,
) {
    let mut ret: DWORD;
    let mut reg_type: DWORD;
    let mut size: DWORD = core::mem::zeroed();
    let mut hroot: HKEY = core::ptr::null_mut();
    let mut hmft: HKEY = core::ptr::null_mut();

    *flags = 0 as DWORD;

    if RegOpenKeyW(
        HKEY_CLASSES_ROOT,
        &raw const transform_keyW as LPCWSTR,
        &raw mut hroot,
    ) != 0
    {
        return;
    }

    ret = RegOpenKeyW(hroot, clsidW as LPCWSTR, &raw mut hmft) as DWORD;
    RegCloseKey(hroot);
    if ret != 0 {
        return;
    }

    reg_type = 0 as DWORD;
    if RegQueryValueExW(
        hmft,
        nameW as LPCWSTR,
        core::ptr::null_mut::<DWORD>(),
        &raw mut reg_type,
        core::ptr::null_mut::<BYTE>(),
        &raw mut size,
    ) == 0
        && reg_type == REG_DWORD as DWORD
    {
        RegQueryValueExW(
            hmft,
            nameW as LPCWSTR,
            core::ptr::null_mut::<DWORD>(),
            &raw mut reg_type,
            flags as *mut BYTE,
            &raw mut size,
        );
    }

    RegCloseKey(hmft);
}

unsafe extern "C" fn mft_collect_machine_reg(
    mfts: *mut list,
    category: *const GUID,
    flags: UINT32,
    plugin_control_local: *mut IMFPluginControl,
    input_type: *const MFT_REGISTER_TYPE_INFO,
    output_type: *const MFT_REGISTER_TYPE_INFO,
) -> HRESULT {
    let mut mft: mft_registration = core::mem::zeroed();
    let mut cur: *mut mft_registration = core::ptr::null_mut();
    let mut hcategory: HKEY = core::ptr::null_mut();
    let mut hlist: HKEY = core::ptr::null_mut();
    let mut clsidW: [WCHAR; 64] = [0; 64];
    let mut ret: DWORD = 0;
    let mut size: DWORD = 0;
    let mut index: core::ffi::c_int = 0;

    if RegOpenKeyW(
        HKEY_CLASSES_ROOT,
        core::ptr::addr_of!(categories_keyW) as LPCWSTR,
        core::ptr::addr_of_mut!(hcategory),
    ) != 0
    {
        return E_FAIL;
    }

    guid_to_string(core::ptr::addr_of_mut!(clsidW) as *mut WCHAR, category);
    ret = RegOpenKeyW(
        hcategory,
        core::ptr::addr_of_mut!(clsidW) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(hlist),
    ) as DWORD;
    RegCloseKey(hcategory);
    if ret != 0 {
        return E_FAIL;
    }

    size = ARRAY_SIZE!(clsidW) as DWORD;
    while RegEnumKeyExW(
        hlist,
        index as DWORD,
        core::ptr::addr_of_mut!(clsidW) as LPWSTR,
        core::ptr::addr_of_mut!(size),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<WCHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<FILETIME>(),
    ) == 0
    {
        'next: {
            memset(
                core::ptr::addr_of_mut!(mft) as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<mft_registration>() as size_t,
            );
            mft.category = *category;
            if GUIDFromString(
                core::ptr::addr_of_mut!(clsidW) as *mut WCHAR as LPCWSTR,
                core::ptr::addr_of_mut!(mft.clsid),
            ) == 0
            {
                break 'next;
            }

            mft_get_reg_flags(
                core::ptr::addr_of_mut!(clsidW) as *mut WCHAR,
                RUSTCONV_WSTR_E89EC7379787.as_ptr(),
                core::ptr::addr_of_mut!(mft.flags),
            );

            if !output_type.is_null() {
                mft_get_reg_type_info_internal(
                    core::ptr::addr_of_mut!(clsidW) as *mut WCHAR,
                    RUSTCONV_WSTR_B17B36FA88F7.as_ptr(),
                    core::ptr::addr_of_mut!(mft.output_types),
                    core::ptr::addr_of_mut!(mft.output_types_count),
                );
            }

            if !input_type.is_null() {
                mft_get_reg_type_info_internal(
                    core::ptr::addr_of_mut!(clsidW) as *mut WCHAR,
                    RUSTCONV_WSTR_5D5100CABDC1.as_ptr(),
                    core::ptr::addr_of_mut!(mft.input_types),
                    core::ptr::addr_of_mut!(mft.input_types_count),
                );
            }

            if mft_is_type_info_match(
                core::ptr::addr_of_mut!(mft),
                category,
                flags,
                plugin_control_local,
                input_type,
                output_type,
            ) == 0
            {
                free(mft.input_types as *mut core::ffi::c_void);
                free(mft.output_types as *mut core::ffi::c_void);
                break 'next;
            }

            cur = malloc(core::mem::size_of::<mft_registration>() as size_t)
                as *mut mft_registration;
            /* Reuse allocated type arrays. */
            *cur = core::ptr::read(core::ptr::addr_of!(mft));
            list_add_tail(mfts, core::ptr::addr_of_mut!((*cur).entry));
        }
        size = ARRAY_SIZE!(clsidW) as DWORD;
        index += 1;
    }

    return S_OK;
}

unsafe extern "C" fn mft_is_preferred(
    plugin_control_local: *mut IMFPluginControl,
    clsid: *const CLSID,
) -> BOOL {
    let mut preferred: CLSID = CLSID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut selector: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut index: core::ffi::c_int = 0;
    loop {
        let fresh24 = index;
        index = index + 1;
        if !(IMFPluginControl_GetPreferredClsidByIndex(
            plugin_control_local,
            MF_Plugin_Type_MFT as core::ffi::c_int as DWORD,
            fresh24 as DWORD,
            &raw mut selector,
            &raw mut preferred,
        ) >= 0)
        {
            break;
        }
        CoTaskMemFree(selector as LPVOID);

        if IsEqualGUID!(&raw mut preferred, clsid) != false {
            return (TRUE) as BOOL;
        }
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn mft_enum(
    mut category: GUID,
    mut flags: UINT32,
    mut input_type: *const MFT_REGISTER_TYPE_INFO,
    mut output_type: *const MFT_REGISTER_TYPE_INFO,
    mut attributes: *mut IMFAttributes,
    mut activate: *mut *mut *mut IMFActivate,
    mut count: *mut UINT32,
) -> HRESULT {
    let mut plugin_control_local: *mut IMFPluginControl = ::core::ptr::null_mut::<IMFPluginControl>();
    let mut mfts: list = list {
        next: ::core::ptr::null_mut::<list>(),
        prev: ::core::ptr::null_mut::<list>(),
    };
    let mut mfts_sorted: list = list {
        next: ::core::ptr::null_mut::<list>(),
        prev: ::core::ptr::null_mut::<list>(),
    };
    let mut result: *mut list = &raw mut mfts;
    let mut mft: *mut mft_registration = ::core::ptr::null_mut::<mft_registration>();
    let mut mft2: *mut mft_registration = ::core::ptr::null_mut::<mft_registration>();
    let mut obj_count: u32 = 0;
    let mut hr: HRESULT = 0;

    *count = 0 as UINT32;
    *activate = ::core::ptr::null_mut::<*mut IMFActivate>();

    if flags == 0 {
        flags = (MFT_ENUM_FLAG_SYNCMFT | MFT_ENUM_FLAG_LOCALMFT | MFT_ENUM_FLAG_SORTANDFILTER) as UINT32;
    }

    /* Synchronous processing is default. */
    if (flags & (MFT_ENUM_FLAG_SYNCMFT | MFT_ENUM_FLAG_ASYNCMFT | MFT_ENUM_FLAG_HARDWARE) as UINT32) == 0 {
        flags |= MFT_ENUM_FLAG_SYNCMFT as UINT32;
    }

    hr = MFGetPluginControl(&raw mut plugin_control_local);
    if hr < 0 as HRESULT {
        WARN!("Failed to get plugin control instance, hr %#lx.\n", hr);
        return hr;
    }

    list_init(&raw mut mfts);

    /* Collect from registry */
    mft_collect_machine_reg(
        &raw mut mfts,
        &raw mut category,
        flags,
        plugin_control_local,
        input_type,
        output_type,
    );

    /* Collect locally registered ones. */
    if flags & MFT_ENUM_FLAG_LOCALMFT as UINT32 != 0 {
        let mut local: *mut mft_registration = ::core::ptr::null_mut::<mft_registration>();

        EnterCriticalSection(&raw mut local_mfts_section);

        local = (local_mfts.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        while &raw mut (*local).entry != &raw mut local_mfts {
            if mft_is_type_info_match(
                local,
                &raw mut category,
                flags,
                plugin_control_local,
                input_type,
                output_type,
            ) != 0
            {
                mft = calloc(
                    (1 as size_t) as _,
                    (::core::mem::size_of::<mft_registration>() as size_t) as _,
                ) as *mut mft_registration;

                (*mft).clsid = (*local).clsid;
                (*mft).factory = (*local).factory;
                if !(*mft).factory.is_null() {
                    IClassFactory_AddRef((*mft).factory);
                }
                (*mft).flags = (*local).flags;
                (*mft).local = (*local).local;

                list_add_tail(&raw mut mfts, &raw mut (*mft).entry);
            }
            local = ((*local).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(mft_registration, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut mft_registration;
        }

        LeaveCriticalSection(&raw mut local_mfts_section);
    }

    list_init(&raw mut mfts_sorted);

    if flags & MFT_ENUM_FLAG_SORTANDFILTER as UINT32 != 0 {
        /* Local registrations. */
        mft = (mfts.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        while &raw mut (*mft).entry != &raw mut mfts {
            if (*mft).local != 0 {
                list_remove(&raw mut (*mft).entry);
                list_add_tail(&raw mut mfts_sorted, &raw mut (*mft).entry);
            }
            mft = mft2;
            mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(mft_registration, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut mft_registration;
        }

        /* FIXME: Sort by merit value, for the ones that got it. Currently not handled. */

        /* Preferred transforms. */
        mft = (mfts.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        while &raw mut (*mft).entry != &raw mut mfts {
            if (*mft).factory.is_null()
                && mft_is_preferred(plugin_control_local, &raw mut (*mft).clsid) != 0
            {
                list_remove(&raw mut (*mft).entry);
                list_add_tail(&raw mut mfts_sorted, &raw mut (*mft).entry);
            }
            mft = mft2;
            mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(mft_registration, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut mft_registration;
        }

        /* Append the rest. */
        mft = (mfts.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        while &raw mut (*mft).entry != &raw mut mfts {
            list_remove(&raw mut (*mft).entry);
            list_add_tail(&raw mut mfts_sorted, &raw mut (*mft).entry);
            mft = mft2;
            mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(mft_registration, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut mft_registration;
        }

        result = &raw mut mfts_sorted;
    }

    IMFPluginControl_Release(plugin_control_local);

    /* Create activation objects from CLSID/IClassFactory. */

    obj_count = list_count(result) as u32;

    if obj_count != 0 {
        *activate = CoTaskMemAlloc(
            (obj_count as SIZE_T)
                .wrapping_mul(::core::mem::size_of::<IMFActivate>() as SIZE_T),
        ) as *mut *mut IMFActivate;
        if (*activate).is_null() {
            hr = E_OUTOFMEMORY as HRESULT;
        }

        obj_count = 0 as ::core::ffi::c_uint;

        mft = ((*result).next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(mft_registration, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut mft_registration;
        while &raw mut (*mft).entry != result {
            let mut mft_activate: *mut IMFActivate = ::core::ptr::null_mut::<IMFActivate>();

            if !(*activate).is_null() {
                if create_transform_activate((*mft).factory, &raw mut mft_activate)
                    >= 0 as HRESULT
                {
                    *(*activate).offset(obj_count as isize) = mft_activate;

                    if (*mft).local != 0 {
                        IMFActivate_SetUINT32(
                            mft_activate,
                            &raw const MFT_PROCESS_LOCAL_Attribute,
                            1 as UINT32,
                        );
                    } else {
                        if !(*mft).name.is_null() {
                            IMFActivate_SetString(
                                mft_activate,
                                &raw const MFT_FRIENDLY_NAME_Attribute,
                                (*mft).name as LPCWSTR,
                            );
                        }
                        if !(*mft).input_types.is_null() {
                            IMFActivate_SetBlob(
                                mft_activate,
                                &raw const MFT_INPUT_TYPES_Attributes,
                                (*mft).input_types as *const UINT8,
                                (::core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as usize)
                                    .wrapping_mul((*mft).input_types_count as usize)
                                    as UINT32,
                            );
                        }
                        if !(*mft).output_types.is_null() {
                            IMFActivate_SetBlob(
                                mft_activate,
                                &raw const MFT_OUTPUT_TYPES_Attributes,
                                (*mft).output_types as *const UINT8,
                                (::core::mem::size_of::<MFT_REGISTER_TYPE_INFO>() as usize)
                                    .wrapping_mul((*mft).output_types_count as usize)
                                    as UINT32,
                            );
                        }
                    }

                    if (*mft).factory.is_null() {
                        IMFActivate_SetGUID(
                            mft_activate,
                            &raw const MFT_TRANSFORM_CLSID_Attribute,
                            &raw mut (*mft).clsid,
                        );
                    }

                    IMFActivate_SetUINT32(
                        mft_activate,
                        &raw const MF_TRANSFORM_FLAGS_Attribute,
                        (*mft).flags as UINT32,
                    );
                    IMFActivate_SetGUID(
                        mft_activate,
                        &raw const MF_TRANSFORM_CATEGORY_Attribute,
                        &raw mut (*mft).category,
                    );

                    obj_count = obj_count.wrapping_add(1);
                }
            }

            list_remove(&raw mut (*mft).entry);
            release_mft_registration(mft);
            mft = mft2;
            mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(mft_registration, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut mft_registration;
        }
    }

    if obj_count == 0 {
        CoTaskMemFree(*activate as LPVOID);
        *activate = ::core::ptr::null_mut::<*mut IMFActivate>();
    }
    *count = obj_count as UINT32;

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFTEnum(
    mut category: GUID,
    mut flags: UINT32,
    mut input_type: *mut MFT_REGISTER_TYPE_INFO,
    mut output_type: *mut MFT_REGISTER_TYPE_INFO,
    mut attributes: *mut IMFAttributes,
    mut clsids: *mut *mut CLSID,
    mut count: *mut UINT32,
) -> HRESULT {
    let mut mft: *mut mft_registration = ::core::ptr::null_mut::<mft_registration>();
    let mut mft2: *mut mft_registration = ::core::ptr::null_mut::<mft_registration>();
    let mut mft_count: ::core::ffi::c_uint = 0;
    let mut mfts: list = list {
        next: ::core::ptr::null_mut::<list>(),
        prev: ::core::ptr::null_mut::<list>(),
    };
    let mut hr: HRESULT = 0;

    if clsids.is_null() || count.is_null() {
        return E_INVALIDARG;
    }

    *count = 0 as UINT32;
    *clsids = ::core::ptr::null_mut::<CLSID>();

    list_init(&raw mut mfts);

    hr = mft_collect_machine_reg(
        &raw mut mfts,
        &raw mut category,
        MFT_ENUM_FLAG_SYNCMFT as UINT32,
        ::core::ptr::null_mut::<IMFPluginControl>(),
        input_type,
        output_type,
    );
    if hr < 0 as HRESULT {
        return hr;
    }

    mft_count = list_count(&raw mut mfts);

    if mft_count != 0 {
        *clsids = CoTaskMemAlloc(
            (mft_count as SIZE_T).wrapping_mul(::core::mem::size_of::<CLSID>() as SIZE_T),
        ) as *mut CLSID;
        if (*clsids).is_null() {
            hr = E_OUTOFMEMORY;
        }

        mft_count = 0 as ::core::ffi::c_uint;
        mft = (mfts.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(mft_registration, entry) as isize))
            as *mut mft_registration;
        mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(mft_registration, entry) as isize))
            as *mut mft_registration;
        while &raw mut (*mft).entry != &raw mut mfts {
            if !(*clsids).is_null() {
                let fresh22 = mft_count;
                mft_count = mft_count.wrapping_add(1);
                *(*clsids).offset(fresh22 as isize) = (*mft).clsid;
            }
            list_remove(&raw mut (*mft).entry);
            release_mft_registration(mft);
            mft = mft2;
            mft2 = ((*mft).entry.next as *mut ::core::ffi::c_char)
                .offset(-(core::mem::offset_of!(mft_registration, entry) as isize))
                as *mut mft_registration;
        }
    }

    if mft_count == 0 {
        CoTaskMemFree(*clsids as LPVOID);
        *clsids = ::core::ptr::null_mut::<CLSID>();
    }
    *count = mft_count as UINT32;

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFTEnumEx(
    mut category: GUID,
    mut flags: UINT32,
    mut input_type: *const MFT_REGISTER_TYPE_INFO,
    mut output_type: *const MFT_REGISTER_TYPE_INFO,
    mut activate: *mut *mut *mut IMFActivate,
    mut count: *mut UINT32,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"MFTEnumEx\0" as *const u8 as *const i8,
            1395i32,
            b"%s, %#x, %s, %s, %p, %p.\n\0" as *const u8 as *const i8,
            debugstr_mf_guid(core::ptr::addr_of_mut!(category)),
            flags,
            debugstr_reg_typeinfo(input_type),
            debugstr_reg_typeinfo(output_type),
            activate,
            count,
        );
    }
    return mft_enum(
        category,
        flags,
        input_type,
        output_type,
        core::ptr::null_mut::<IMFAttributes>(),
        activate,
        count,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MFTEnum2(
    mut category: GUID,
    mut flags: UINT32,
    mut input_type: *const MFT_REGISTER_TYPE_INFO,
    mut output_type: *const MFT_REGISTER_TYPE_INFO,
    mut attributes: *mut IMFAttributes,
    mut activate: *mut *mut *mut IMFActivate,
    mut count: *mut UINT32,
) -> HRESULT {
    if !attributes.is_null() {
        FIXME!("Ignoring attributes.\n");
    }

    return mft_enum(
        category,
        flags,
        input_type,
        output_type,
        attributes,
        activate,
        count,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MFTUnregister(mut clsid: CLSID) -> HRESULT {
    let mut buffer: [WCHAR; 64] = [0; 64];
    let mut category: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut htransform: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut hcategory: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut htmp: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut size: DWORD = MAX_PATH as DWORD;
    let mut index: DWORD = 0 as DWORD;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFTUnregister\0" as *const u8 as *const ::core::ffi::c_char,
            1425 as ::core::ffi::c_int,
            b"(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_guid(&raw mut clsid),
        );
    }
    guid_to_string(&raw mut buffer as *mut WCHAR, &raw mut clsid);
    if RegOpenKeyW(
        HKEY_CLASSES_ROOT,
        &raw const transform_keyW as LPCWSTR,
        &raw mut htransform,
    ) == 0
    {
        RegDeleteKeyW(htransform, &raw mut buffer as *mut WCHAR as LPCWSTR);
        RegCloseKey(htransform);
    }
    if RegOpenKeyW(
        HKEY_CLASSES_ROOT,
        &raw const categories_keyW as LPCWSTR,
        &raw mut hcategory,
    ) == 0
    {
        while RegEnumKeyExW(
            hcategory,
            index,
            &raw mut category as LPWSTR,
            &raw mut size,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<WCHAR>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<_FILETIME>(),
        ) == ERROR_SUCCESS as LSTATUS
        {
            if RegOpenKeyW(
                hcategory,
                &raw mut category as *mut WCHAR as LPCWSTR,
                &raw mut htmp,
            ) == 0
            {
                RegDeleteKeyW(htmp, &raw mut buffer as *mut WCHAR as LPCWSTR);
                RegCloseKey(htmp);
            }
            size = MAX_PATH as DWORD;
            index = index.wrapping_add(1);
        }
        RegCloseKey(hcategory);
    }
    return S_OK;
}

unsafe extern "C" fn mft_get_name(hkey: HKEY, name: *mut *mut WCHAR) -> HRESULT {
    let mut size: DWORD = 0;

    if name.is_null() {
        return S_OK;
    }

    *name = core::ptr::null_mut();

    if RegQueryValueExW(
        hkey,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
    ) == 0
    {
        *name = CoTaskMemAlloc((size) as SIZE_T) as *mut WCHAR;
        if (*name).is_null() {
            return E_OUTOFMEMORY;
        }

        RegQueryValueExW(
            hkey,
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            *name as *mut BYTE,
            core::ptr::addr_of_mut!(size),
        );
    }

    return S_OK;
}

unsafe extern "C" fn mft_get_reg_type_info(clsid: *const WCHAR, key: *const WCHAR, ret_types: *mut *mut MFT_REGISTER_TYPE_INFO, ret_count: *mut UINT32) -> HRESULT {
    let mut types: *mut MFT_REGISTER_TYPE_INFO = core::ptr::null_mut();
    let mut count: UINT32 = 0;

    if ret_types.is_null() {
        return S_OK;
    }

    mft_get_reg_type_info_internal(clsid, key, core::ptr::addr_of_mut!(types), core::ptr::addr_of_mut!(count));
    if count != 0 {
        *ret_types = CoTaskMemAlloc(((count as usize) * core::mem::size_of::<MFT_REGISTER_TYPE_INFO>()) as SIZE_T) as *mut MFT_REGISTER_TYPE_INFO;
        if (*ret_types).is_null() {
            free(types as *mut core::ffi::c_void);
            return E_OUTOFMEMORY;
        }

        memcpy(
            *ret_types as *mut core::ffi::c_void,
            types as *const core::ffi::c_void,
            (count as usize) * core::mem::size_of::<MFT_REGISTER_TYPE_INFO>(),
        );
        *ret_count = count;
    }

    free(types as *mut core::ffi::c_void);

    return S_OK;
}

unsafe extern "C" fn mft_get_attributes(hkey: HKEY, ret: *mut *mut IMFAttributes) -> HRESULT {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    let mut blob: *mut UINT8 = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut hr: HRESULT = 0;

    if ret.is_null() {
        return S_OK;
    }

    hr = MFCreateAttributes(core::ptr::addr_of_mut!(attributes), 0);
    if hr < 0 {
        return hr;
    }

    if RegQueryValueExW(
        hkey,
        RUSTCONV_WSTR_A38E93E48CB3.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
    ) == 0
        && size != 0
    {
        blob = malloc(size as size_t) as *mut UINT8;
        if blob.is_null() {
            IMFAttributes_Release(attributes);
            return E_OUTOFMEMORY;
        }

        if RegQueryValueExW(
            hkey,
            RUSTCONV_WSTR_A38E93E48CB3.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            blob as LPBYTE,
            core::ptr::addr_of_mut!(size),
        ) == 0
        {
            hr = MFInitAttributesFromBlob(attributes, blob, size as UINT);
            if hr < 0 {
                WARN!("Failed to initialize attributes, hr %#lx.\n", hr);
            }
        }

        free(blob as *mut core::ffi::c_void);
    }

    if hr >= 0 {
        *ret = attributes;
        IMFAttributes_AddRef(*ret);
    }

    IMFAttributes_Release(attributes);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFTGetInfo(
    mut clsid: CLSID,
    mut name: *mut *mut WCHAR,
    mut input_types: *mut *mut MFT_REGISTER_TYPE_INFO,
    mut input_types_count: *mut UINT32,
    mut output_types: *mut *mut MFT_REGISTER_TYPE_INFO,
    mut output_types_count: *mut UINT32,
    mut attributes: *mut *mut IMFAttributes,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut clsidW: [WCHAR; 64] = [0; 64];
    let mut hroot: HKEY = core::ptr::null_mut();
    let mut hmft: HKEY = core::ptr::null_mut();
    let mut ret: DWORD = 0;

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8
                as *const i8,
            b"MFTGetInfo\0" as *const u8 as *const i8,
            1554 as i32,
            b"%s, %p, %p, %p, %p, %p, %p.\n\0" as *const u8 as *const i8,
            debugstr_guid(core::ptr::addr_of_mut!(clsid)),
            name,
            input_types,
            input_types_count,
            output_types,
            output_types_count,
            attributes,
        );
    }

    guid_to_string(core::ptr::addr_of_mut!(clsidW) as *mut WCHAR, core::ptr::addr_of_mut!(clsid));

    ret = RegOpenKeyW(
        HKEY_CLASSES_ROOT,
        core::ptr::addr_of!(transform_keyW) as LPCWSTR,
        core::ptr::addr_of_mut!(hroot),
    ) as DWORD;
    if ret != 0 {
        return HRESULT_FROM_WIN32!(ret);
    }

    ret = RegOpenKeyW(
        hroot,
        core::ptr::addr_of_mut!(clsidW) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(hmft),
    ) as DWORD;
    RegCloseKey(hroot);
    if ret != 0 {
        return HRESULT_FROM_WIN32!(ret);
    }

    if !input_types_count.is_null() {
        *input_types_count = 0;
    }

    if !output_types_count.is_null() {
        *output_types_count = 0;
    }

    hr = mft_get_name(hmft, name);

    if hr >= 0 {
        hr = mft_get_reg_type_info(
            core::ptr::addr_of_mut!(clsidW) as *mut WCHAR,
            RUSTCONV_WSTR_5D5100CABDC1.as_ptr(),
            input_types,
            input_types_count,
        );
    }

    if hr >= 0 {
        hr = mft_get_reg_type_info(
            core::ptr::addr_of_mut!(clsidW) as *mut WCHAR,
            RUSTCONV_WSTR_B17B36FA88F7.as_ptr(),
            output_types,
            output_types_count,
        );
    }

    if hr >= 0 {
        hr = mft_get_attributes(hmft, attributes);
    }

    RegCloseKey(hmft);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFStartup(version: ULONG, flags: DWORD) -> HRESULT {
    TRACE!("%#lx, %#lx.\n", version, flags);

    if version != (MF_VERSION_XP as u32) && version != (MF_VERSION_WIN7 as u32) {
        return MF_E_BAD_STARTUP_VERSION;
    }

    RtwqStartup();

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFShutdown() -> HRESULT {
    RtwqShutdown();

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCopyImage(
    mut dest: *mut BYTE,
    mut deststride: LONG,
    mut src: *const BYTE,
    mut srcstride: LONG,
    mut width: DWORD,
    mut lines: DWORD,
) -> HRESULT {
    while {
        let fresh = lines;
        lines = lines.wrapping_sub(1);
        fresh != 0
    } {
        memcpy((dest).cast(), (src).cast(), (width) as _);
        dest = dest.offset(deststride as isize);
        src = src.offset(srcstride as isize);
    }

    return S_OK;
}

unsafe extern "C" fn debug_compare_guid(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> i32 {
    let guid: *const GUID = a as *const GUID;
    let guid_def: *const guid_def = b as *const guid_def;
    return memcmp(
        guid as *const core::ffi::c_void,
        (*guid_def).guid as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn debugstr_mf_guid(mut guid: *const GUID) -> *const ::core::ffi::c_char {
    static mut guid_defs: [guid_def; 127] = unsafe {
        [
            guid_def { guid: &raw const MFAudioFormat_ADTS, name: b"MFAudioFormat_ADTS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_PCM, name: b"MFAudioFormat_PCM\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_PCM_HDCP, name: b"MFAudioFormat_PCM_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Float, name: b"MFAudioFormat_Float\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_DTS, name: b"MFAudioFormat_DTS\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_DRM, name: b"MFAudioFormat_DRM\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_MSP1, name: b"MFAudioFormat_MSP1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Vorbis, name: b"MFAudioFormat_Vorbis\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_AAC, name: b"MFAudioFormat_AAC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_RGB24, name: b"MFVideoFormat_RGB24\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_ARGB32, name: b"MFVideoFormat_ARGB32\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_RGB32, name: b"MFVideoFormat_RGB32\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_RGB565, name: b"MFVideoFormat_RGB565\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_RGB555, name: b"MFVideoFormat_RGB555\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_GUID, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_GUID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_MULTIPLEXER, name: b"MFT_CATEGORY_MULTIPLEXER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_A2R10G10B10, name: b"MFVideoFormat_A2R10G10B10\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_VIDEO_EFFECT, name: b"MFT_CATEGORY_VIDEO_EFFECT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Script, name: b"MFMediaType_Script\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Image, name: b"MFMediaType_Image\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_HTML, name: b"MFMediaType_HTML\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Binary, name: b"MFMediaType_Binary\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MPEG2, name: b"MFVideoFormat_MPEG2\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_FileTransfer, name: b"MFMediaType_FileTransfer\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_RGB8, name: b"MFVideoFormat_RGB8\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Dolby_AC3, name: b"MFAudioFormat_Dolby_AC3\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_L8, name: b"MFVideoFormat_L8\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_LPCM, name: b"MFAudioFormat_LPCM\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_420O, name: b"MFVideoFormat_420O\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_AI44, name: b"MFVideoFormat_AI44\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_AV1, name: b"MFVideoFormat_AV1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_AYUV, name: b"MFVideoFormat_AYUV\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_H263, name: b"MFVideoFormat_H263\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_H264, name: b"MFVideoFormat_H264\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_H265, name: b"MFVideoFormat_H265\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_HEVC, name: b"MFVideoFormat_HEVC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_HEVC_ES, name: b"MFVideoFormat_HEVC_ES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_AUDIO_EFFECT, name: b"MFT_CATEGORY_AUDIO_EFFECT\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_I420, name: b"MFVideoFormat_I420\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_IYUV, name: b"MFVideoFormat_IYUV\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_VIDEO_DECODER, name: b"MFT_CATEGORY_VIDEO_DECODER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_M4S2, name: b"MFVideoFormat_M4S2\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MJPG, name: b"MFVideoFormat_MJPG\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MP43, name: b"MFVideoFormat_MP43\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MP4S, name: b"MFVideoFormat_MP4S\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MP4V, name: b"MFVideoFormat_MP4V\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MPG1, name: b"MFVideoFormat_MPG1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MSS1, name: b"MFVideoFormat_MSS1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_MSS2, name: b"MFVideoFormat_MSS2\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_NV11, name: b"MFVideoFormat_NV11\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_NV12, name: b"MFVideoFormat_NV12\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_ORAW, name: b"MFVideoFormat_ORAW\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Opus, name: b"MFAudioFormat_Opus\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_MPEG, name: b"MFAudioFormat_MPEG\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_D16, name: b"MFVideoFormat_D16\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_P010, name: b"MFVideoFormat_P010\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_P016, name: b"MFVideoFormat_P016\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_P210, name: b"MFVideoFormat_P210\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_P216, name: b"MFVideoFormat_P216\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_L16, name: b"MFVideoFormat_L16\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_MP3, name: b"MFAudioFormat_MP3\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_UYVY, name: b"MFVideoFormat_UYVY\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_VP10, name: b"MFVideoFormat_VP10\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_VP80, name: b"MFVideoFormat_VP80\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_VP90, name: b"MFVideoFormat_VP90\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_WMV1, name: b"MFVideoFormat_WMV1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_WMV2, name: b"MFVideoFormat_WMV2\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_WMV3, name: b"MFVideoFormat_WMV3\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_WVC1, name: b"MFVideoFormat_WVC1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_OTHER, name: b"MFT_CATEGORY_OTHER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Y210, name: b"MFVideoFormat_Y210\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Y216, name: b"MFVideoFormat_Y216\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Y410, name: b"MFVideoFormat_Y410\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Y416, name: b"MFVideoFormat_Y416\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Y41P, name: b"MFVideoFormat_Y41P\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Y41T, name: b"MFVideoFormat_Y41T\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Y42T, name: b"MFVideoFormat_Y42T\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_YUY2, name: b"MFVideoFormat_YUY2\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_YV12, name: b"MFVideoFormat_YV12\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_YVU9, name: b"MFVideoFormat_YVU9\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_YVYU, name: b"MFVideoFormat_YVYU\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_WMAudioV8, name: b"MFAudioFormat_WMAudioV8\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_ALAC, name: b"MFAudioFormat_ALAC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_AMR_NB, name: b"MFAudioFormat_AMR_NB\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Audio, name: b"MFMediaType_Audio\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_WMAudioV9, name: b"MFAudioFormat_WMAudioV9\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_AMR_WB, name: b"MFAudioFormat_AMR_WB\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_WMAudio_Lossless, name: b"MFAudioFormat_WMAudio_Lossless\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_AMR_WP, name: b"MFAudioFormat_AMR_WP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_WMASPDIF, name: b"MFAudioFormat_WMASPDIF\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_DV25, name: b"MFVideoFormat_DV25\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_DV50, name: b"MFVideoFormat_DV50\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_DVC, name: b"MFVideoFormat_DVC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_DVH1, name: b"MFVideoFormat_DVH1\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_DVHD, name: b"MFVideoFormat_DVHD\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_DVSD, name: b"MFVideoFormat_DVSD\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_DVSL, name: b"MFVideoFormat_DVSL\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_A16B16G16R16F, name: b"MFVideoFormat_A16B16G16R16F\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_v210, name: b"MFVideoFormat_v210\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_v216, name: b"MFVideoFormat_v216\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_v410, name: b"MFVideoFormat_v410\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Video, name: b"MFMediaType_Video\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_AAC_HDCP, name: b"MFAudioFormat_AAC_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_DEMULTIPLEXER, name: b"MFT_CATEGORY_DEMULTIPLEXER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID, name: b"MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_VIDEO_ENCODER, name: b"MFT_CATEGORY_VIDEO_ENCODER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Dolby_AC3_HDCP, name: b"MFAudioFormat_Dolby_AC3_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Subtitle, name: b"MFMediaType_Subtitle\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Stream, name: b"MFMediaType_Stream\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Dolby_AC3_SPDIF, name: b"MFAudioFormat_Dolby_AC3_SPDIF\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Float_SpatialObjects, name: b"MFAudioFormat_Float_SpatialObjects\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_SAMI, name: b"MFMediaType_SAMI\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_ADTS_HDCP, name: b"MFAudioFormat_ADTS_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_FLAC, name: b"MFAudioFormat_FLAC\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Dolby_DDPlus, name: b"MFAudioFormat_Dolby_DDPlus\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_MultiplexedFrames, name: b"MFMediaType_MultiplexedFrames\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_AUDIO_DECODER, name: b"MFT_CATEGORY_AUDIO_DECODER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFAudioFormat_Base_HDCP, name: b"MFAudioFormat_Base_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_AUDIO_ENCODER, name: b"MFT_CATEGORY_AUDIO_ENCODER\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_Base_HDCP, name: b"MFVideoFormat_Base_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_H264_HDCP, name: b"MFVideoFormat_H264_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_HEVC_HDCP, name: b"MFVideoFormat_HEVC_HDCP\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Default, name: b"MFMediaType_Default\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Protected, name: b"MFMediaType_Protected\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFVideoFormat_H264_ES, name: b"MFVideoFormat_H264_ES\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFMediaType_Perception, name: b"MFMediaType_Perception\0" as *const u8 as *const ::core::ffi::c_char },
            guid_def { guid: &raw const MFT_CATEGORY_VIDEO_PROCESSOR, name: b"MFT_CATEGORY_VIDEO_PROCESSOR\0" as *const u8 as *const ::core::ffi::c_char },
        ]
    };
    let mut ret: *mut guid_def = ::core::ptr::null_mut::<guid_def>();
    if !guid.is_null() {
        ret = bsearch(
            guid as *const ::core::ffi::c_void,
            &raw const guid_defs as *const guid_def as *const ::core::ffi::c_void,
            (::core::mem::size_of::<[guid_def; 127]>() as size_t)
                .wrapping_div(::core::mem::size_of::<guid_def>() as size_t),
            ::core::mem::size_of::<guid_def>() as size_t,
            Some(
                debug_compare_guid
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        ) as *mut guid_def;
    }
    return if !ret.is_null() {
        wine_dbg_sprintf(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*ret).name,
        )
    } else {
        wine_dbgstr_guid(guid)
    };
}

unsafe extern "C" fn debug_event_id(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    let id: *const DWORD = a as *const DWORD;
    let event_id: *const event_id = b as *const event_id;
    return (*id).wrapping_sub((*event_id).id) as core::ffi::c_int;
}

unsafe extern "C" fn debugstr_eventid(mut event: DWORD) -> *const ::core::ffi::c_char {
    static mut event_ids: [event_id; 112] = [
        event_id { id: MEUnknown as ::core::ffi::c_int as DWORD, name: b"MEUnknown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEError as ::core::ffi::c_int as DWORD, name: b"MEError\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEExtendedType as ::core::ffi::c_int as DWORD, name: b"MEExtendedType\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MENonFatalError as ::core::ffi::c_int as DWORD, name: b"MENonFatalError\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionUnknown as ::core::ffi::c_int as DWORD, name: b"MESessionUnknown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionTopologySet as ::core::ffi::c_int as DWORD, name: b"MESessionTopologySet\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionTopologiesCleared as ::core::ffi::c_int as DWORD, name: b"MESessionTopologiesCleared\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionStarted as ::core::ffi::c_int as DWORD, name: b"MESessionStarted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionPaused as ::core::ffi::c_int as DWORD, name: b"MESessionPaused\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionStopped as ::core::ffi::c_int as DWORD, name: b"MESessionStopped\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionClosed as ::core::ffi::c_int as DWORD, name: b"MESessionClosed\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionEnded as ::core::ffi::c_int as DWORD, name: b"MESessionEnded\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionRateChanged as ::core::ffi::c_int as DWORD, name: b"MESessionRateChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionScrubSampleComplete as ::core::ffi::c_int as DWORD, name: b"MESessionScrubSampleComplete\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionCapabilitiesChanged as ::core::ffi::c_int as DWORD, name: b"MESessionCapabilitiesChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionTopologyStatus as ::core::ffi::c_int as DWORD, name: b"MESessionTopologyStatus\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionNotifyPresentationTime as ::core::ffi::c_int as DWORD, name: b"MESessionNotifyPresentationTime\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MENewPresentation as ::core::ffi::c_int as DWORD, name: b"MENewPresentation\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MELicenseAcquisitionStart as ::core::ffi::c_int as DWORD, name: b"MELicenseAcquisitionStart\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MELicenseAcquisitionCompleted as ::core::ffi::c_int as DWORD, name: b"MELicenseAcquisitionCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEIndividualizationStart as ::core::ffi::c_int as DWORD, name: b"MEIndividualizationStart\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEIndividualizationCompleted as ::core::ffi::c_int as DWORD, name: b"MEIndividualizationCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEEnablerProgress as ::core::ffi::c_int as DWORD, name: b"MEEnablerProgress\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEEnablerCompleted as ::core::ffi::c_int as DWORD, name: b"MEEnablerCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEPolicyError as ::core::ffi::c_int as DWORD, name: b"MEPolicyError\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEPolicyReport as ::core::ffi::c_int as DWORD, name: b"MEPolicyReport\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEBufferingStarted as ::core::ffi::c_int as DWORD, name: b"MEBufferingStarted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEBufferingStopped as ::core::ffi::c_int as DWORD, name: b"MEBufferingStopped\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEConnectStart as ::core::ffi::c_int as DWORD, name: b"MEConnectStart\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEConnectEnd as ::core::ffi::c_int as DWORD, name: b"MEConnectEnd\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEReconnectStart as ::core::ffi::c_int as DWORD, name: b"MEReconnectStart\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEReconnectEnd as ::core::ffi::c_int as DWORD, name: b"MEReconnectEnd\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MERendererEvent as ::core::ffi::c_int as DWORD, name: b"MERendererEvent\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESessionStreamSinkFormatChanged as ::core::ffi::c_int as DWORD, name: b"MESessionStreamSinkFormatChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceUnknown as ::core::ffi::c_int as DWORD, name: b"MESourceUnknown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceStarted as ::core::ffi::c_int as DWORD, name: b"MESourceStarted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamStarted as ::core::ffi::c_int as DWORD, name: b"MEStreamStarted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceSeeked as ::core::ffi::c_int as DWORD, name: b"MESourceSeeked\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSeeked as ::core::ffi::c_int as DWORD, name: b"MEStreamSeeked\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MENewStream as ::core::ffi::c_int as DWORD, name: b"MENewStream\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEUpdatedStream as ::core::ffi::c_int as DWORD, name: b"MEUpdatedStream\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceStopped as ::core::ffi::c_int as DWORD, name: b"MESourceStopped\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamStopped as ::core::ffi::c_int as DWORD, name: b"MEStreamStopped\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourcePaused as ::core::ffi::c_int as DWORD, name: b"MESourcePaused\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamPaused as ::core::ffi::c_int as DWORD, name: b"MEStreamPaused\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEEndOfPresentation as ::core::ffi::c_int as DWORD, name: b"MEEndOfPresentation\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEEndOfStream as ::core::ffi::c_int as DWORD, name: b"MEEndOfStream\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEMediaSample as ::core::ffi::c_int as DWORD, name: b"MEMediaSample\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamTick as ::core::ffi::c_int as DWORD, name: b"MEStreamTick\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamThinMode as ::core::ffi::c_int as DWORD, name: b"MEStreamThinMode\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamFormatChanged as ::core::ffi::c_int as DWORD, name: b"MEStreamFormatChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceRateChanged as ::core::ffi::c_int as DWORD, name: b"MESourceRateChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEEndOfPresentationSegment as ::core::ffi::c_int as DWORD, name: b"MEEndOfPresentationSegment\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceCharacteristicsChanged as ::core::ffi::c_int as DWORD, name: b"MESourceCharacteristicsChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceRateChangeRequested as ::core::ffi::c_int as DWORD, name: b"MESourceRateChangeRequested\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESourceMetadataChanged as ::core::ffi::c_int as DWORD, name: b"MESourceMetadataChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESequencerSourceTopologyUpdated as ::core::ffi::c_int as DWORD, name: b"MESequencerSourceTopologyUpdated\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESinkUnknown as ::core::ffi::c_int as DWORD, name: b"MESinkUnknown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkStarted as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkStarted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkStopped as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkStopped\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkPaused as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkPaused\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkRateChanged as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkRateChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkRequestSample as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkRequestSample\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkMarker as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkMarker\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkPrerolled as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkPrerolled\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkScrubSampleComplete as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkScrubSampleComplete\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkFormatChanged as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkFormatChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkDeviceChanged as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkDeviceChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEQualityNotify as ::core::ffi::c_int as DWORD, name: b"MEQualityNotify\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MESinkInvalidated as ::core::ffi::c_int as DWORD, name: b"MESinkInvalidated\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionNameChanged as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionNameChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionVolumeChanged as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionVolumeChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionDeviceRemoved as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionDeviceRemoved\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionServerShutdown as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionServerShutdown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionGroupingParamChanged as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionGroupingParamChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionIconChanged as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionIconChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionFormatChanged as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionFormatChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionDisconnected as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionDisconnected\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEAudioSessionExclusiveModeOverride as ::core::ffi::c_int as DWORD, name: b"MEAudioSessionExclusiveModeOverride\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MECaptureAudioSessionVolumeChanged as ::core::ffi::c_int as DWORD, name: b"MECaptureAudioSessionVolumeChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MECaptureAudioSessionDeviceRemoved as ::core::ffi::c_int as DWORD, name: b"MECaptureAudioSessionDeviceRemoved\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MECaptureAudioSessionFormatChanged as ::core::ffi::c_int as DWORD, name: b"MECaptureAudioSessionFormatChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MECaptureAudioSessionDisconnected as ::core::ffi::c_int as DWORD, name: b"MECaptureAudioSessionDisconnected\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MECaptureAudioSessionExclusiveModeOverride as ::core::ffi::c_int as DWORD, name: b"MECaptureAudioSessionExclusiveModeOverride\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MECaptureAudioSessionServerShutdown as ::core::ffi::c_int as DWORD, name: b"MECaptureAudioSessionServerShutdown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: METrustUnknown as ::core::ffi::c_int as DWORD, name: b"METrustUnknown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEPolicyChanged as ::core::ffi::c_int as DWORD, name: b"MEPolicyChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEContentProtectionMessage as ::core::ffi::c_int as DWORD, name: b"MEContentProtectionMessage\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEPolicySet as ::core::ffi::c_int as DWORD, name: b"MEPolicySet\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMLicenseBackupCompleted as ::core::ffi::c_int as DWORD, name: b"MEWMDRMLicenseBackupCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMLicenseBackupProgress as ::core::ffi::c_int as DWORD, name: b"MEWMDRMLicenseBackupProgress\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMLicenseRestoreCompleted as ::core::ffi::c_int as DWORD, name: b"MEWMDRMLicenseRestoreCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMLicenseRestoreProgress as ::core::ffi::c_int as DWORD, name: b"MEWMDRMLicenseRestoreProgress\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMLicenseAcquisitionCompleted as ::core::ffi::c_int as DWORD, name: b"MEWMDRMLicenseAcquisitionCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMIndividualizationCompleted as ::core::ffi::c_int as DWORD, name: b"MEWMDRMIndividualizationCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMIndividualizationProgress as ::core::ffi::c_int as DWORD, name: b"MEWMDRMIndividualizationProgress\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMProximityCompleted as ::core::ffi::c_int as DWORD, name: b"MEWMDRMProximityCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMLicenseStoreCleaned as ::core::ffi::c_int as DWORD, name: b"MEWMDRMLicenseStoreCleaned\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEWMDRMRevocationDownloadCompleted as ::core::ffi::c_int as DWORD, name: b"MEWMDRMRevocationDownloadCompleted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: METransformUnknown as ::core::ffi::c_int as DWORD, name: b"METransformUnknown\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: METransformNeedInput as ::core::ffi::c_int as DWORD, name: b"METransformNeedInput\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: METransformHaveOutput as ::core::ffi::c_int as DWORD, name: b"METransformHaveOutput\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: METransformDrainComplete as ::core::ffi::c_int as DWORD, name: b"METransformDrainComplete\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: METransformMarker as ::core::ffi::c_int as DWORD, name: b"METransformMarker\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: METransformInputStreamStateChanged as ::core::ffi::c_int as DWORD, name: b"METransformInputStreamStateChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEByteStreamCharacteristicsChanged as ::core::ffi::c_int as DWORD, name: b"MEByteStreamCharacteristicsChanged\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEVideoCaptureDeviceRemoved as ::core::ffi::c_int as DWORD, name: b"MEVideoCaptureDeviceRemoved\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEVideoCaptureDevicePreempted as ::core::ffi::c_int as DWORD, name: b"MEVideoCaptureDevicePreempted\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEStreamSinkFormatInvalidated as ::core::ffi::c_int as DWORD, name: b"MEStreamSinkFormatInvalidated\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEEncodingParameters as ::core::ffi::c_int as DWORD, name: b"MEEncodingParameters\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEContentProtectionMetadata as ::core::ffi::c_int as DWORD, name: b"MEContentProtectionMetadata\0" as *const u8 as *const ::core::ffi::c_char },
        event_id { id: MEDeviceThermalStateChanged as ::core::ffi::c_int as DWORD, name: b"MEDeviceThermalStateChanged\0" as *const u8 as *const ::core::ffi::c_char },
    ];
    let mut ret: *mut event_id = bsearch(
        &raw mut event as *const ::core::ffi::c_void,
        &raw const event_ids as *const event_id as *const ::core::ffi::c_void,
        ARRAY_SIZE!(event_ids) as size_t,
        ::core::mem::size_of::<event_id>() as size_t,
        Some(
            debug_event_id
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *mut event_id;
    return if !ret.is_null() {
        wine_dbg_sprintf(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*ret).name,
        )
    } else {
        wine_dbg_sprintf(b"%lu\0" as *const u8 as *const ::core::ffi::c_char, event)
    };
}

unsafe extern "C" fn impl_from_IMFAttributes(iface: *mut IMFAttributes) -> *mut attributes {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(attributes, IMFAttributes_iface),
    ) as *mut attributes;
}

unsafe extern "system" fn mfattributes_QueryInterface(iface: *mut IMFAttributes, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        *out = iface as *mut core::ffi::c_void;
        IMFAttributes_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn mfattributes_AddRef(iface: *mut IMFAttributes) -> ULONG {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*attributes).ref_) as *mut core::ffi::c_long,
    ) as ULONG;

    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"mfattributes_AddRef\0" as *const u8 as *const core::ffi::c_char,
            2366 as core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }

    return refcount;
}

unsafe extern "system" fn mfattributes_Release(iface: *mut IMFAttributes) -> ULONG {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*attributes).ref_) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfattributes_Release\0" as *const u8 as *const ::core::ffi::c_char,
            2376 as ::core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        clear_attributes_object(attributes);
        free(attributes as *mut ::core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "C" fn attributes_find_item(
    attributes: *mut attributes,
    key: *const GUID,
    index: *mut size_t,
) -> *mut attribute {
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < (*attributes).count {
        if IsEqualGUID!(key, core::ptr::addr_of!((*(*attributes).attributes.add(i as usize)).key)) != false {
            if !index.is_null() {
                *index = i;
            }
            return (*attributes).attributes.add(i as usize) as *mut attribute;
        }
        i += 1;
    }
    return core::ptr::null_mut::<attribute>();
}

unsafe extern "C" fn attributes_get_item(attributes: *mut attributes, key: *const GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if !attribute.is_null() {
        if (*attribute).value.vt == (*value).vt && !((*value).vt == VT_UNKNOWN && (*attribute).value.__bindgen_anon_1.punkVal.is_null()) {
            hr = PropVariantCopy(value, core::ptr::addr_of_mut!((*attribute).value));
        } else {
            hr = MF_E_INVALIDTYPE;
        }
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetItem(
    attributes: *mut attributes,
    key: *const GUID,
    value: *mut PROPVARIANT,
) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if !attribute.is_null() {
        hr = if !value.is_null() {
            PropVariantCopy(value, core::ptr::addr_of_mut!((*attribute).value))
        } else {
            S_OK
        };
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetItemType(
    attributes: *mut attributes,
    key: *const GUID,
    type_: *mut MF_ATTRIBUTE_TYPE,
) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if !attribute.is_null() {
        *type_ = (*attribute).value.vt as MF_ATTRIBUTE_TYPE;
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_CompareItem(
    attributes: *mut attributes,
    key: REFGUID,
    value: REFPROPVARIANT,
    result: *mut BOOL,
) -> HRESULT {
    let mut attribute: *mut attribute;

    *result = (FALSE as i32);

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if !attribute.is_null() {
        *result = ((*attribute).value.vt == (*value).vt
            && PropVariantCompareEx(
                core::ptr::addr_of_mut!((*attribute).value),
                value,
                (PVCU_DEFAULT) as PROPVAR_COMPARE_UNIT,
                (PVCF_DEFAULT) as PROPVAR_COMPARE_FLAGS,
            ) == 0) as BOOL;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_Compare(
    attributes: *mut attributes,
    theirs: *mut IMFAttributes,
    match_type: MF_ATTRIBUTES_MATCH_TYPE,
    ret: *mut BOOL,
) -> HRESULT {
    let mut smaller: *mut IMFAttributes = core::ptr::null_mut();
    let mut other: *mut IMFAttributes = core::ptr::null_mut();
    let mut type_0: MF_ATTRIBUTE_TYPE = 0 as MF_ATTRIBUTE_TYPE;
    let mut hr: HRESULT = S_OK;
    let mut count: UINT32 = 0;
    let mut result: BOOL = 0;
    let mut i: size_t = 0;

    hr = IMFAttributes_GetCount(theirs, core::ptr::addr_of_mut!(count));
    if hr < 0 {
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    result = (TRUE as i32);

    match match_type as ::core::ffi::c_uint {
        MF_ATTRIBUTES_MATCH_OUR_ITEMS => {
            i = 0;
            while i < (*attributes).count {
                hr = IMFAttributes_CompareItem(
                    theirs,
                    core::ptr::addr_of_mut!((*(*attributes).attributes.add(i)).key),
                    core::ptr::addr_of_mut!((*(*attributes).attributes.add(i)).value),
                    core::ptr::addr_of_mut!(result),
                );
                if hr < 0 {
                    break;
                }
                if result == 0 {
                    break;
                }
                i = i.wrapping_add(1);
            }
        }
        MF_ATTRIBUTES_MATCH_THEIR_ITEMS => {
            hr = IMFAttributes_Compare(
                theirs,
                core::ptr::addr_of_mut!((*attributes).IMFAttributes_iface),
                MF_ATTRIBUTES_MATCH_OUR_ITEMS,
                core::ptr::addr_of_mut!(result),
            );
        }
        MF_ATTRIBUTES_MATCH_ALL_ITEMS => {
            if count as size_t != (*attributes).count {
                result = (FALSE as i32);
            } else {
                i = 0;
                while i < count as size_t {
                    hr = IMFAttributes_CompareItem(
                        theirs,
                        core::ptr::addr_of_mut!((*(*attributes).attributes.add(i)).key),
                        core::ptr::addr_of_mut!((*(*attributes).attributes.add(i)).value),
                        core::ptr::addr_of_mut!(result),
                    );
                    if hr < 0 {
                        break;
                    }
                    if result == 0 {
                        break;
                    }
                    i = i.wrapping_add(1);
                }
            }
        }
        MF_ATTRIBUTES_MATCH_INTERSECTION => {
            i = 0;
            while i < (*attributes).count {
                if !(IMFAttributes_GetItemType(
                    theirs,
                    core::ptr::addr_of_mut!((*(*attributes).attributes.add(i)).key),
                    core::ptr::addr_of_mut!(type_0),
                ) < 0)
                {
                    hr = IMFAttributes_CompareItem(
                        theirs,
                        core::ptr::addr_of_mut!((*(*attributes).attributes.add(i)).key),
                        core::ptr::addr_of_mut!((*(*attributes).attributes.add(i)).value),
                        core::ptr::addr_of_mut!(result),
                    );
                    if hr < 0 {
                        break;
                    }
                    if result == 0 {
                        break;
                    }
                }
                i = i.wrapping_add(1);
            }
        }
        MF_ATTRIBUTES_MATCH_SMALLER => {
            smaller = if (*attributes).count > count as size_t {
                theirs
            } else {
                core::ptr::addr_of_mut!((*attributes).IMFAttributes_iface)
            };
            other = if (*attributes).count > count as size_t {
                core::ptr::addr_of_mut!((*attributes).IMFAttributes_iface)
            } else {
                theirs
            };
            hr = IMFAttributes_Compare(
                smaller,
                other,
                MF_ATTRIBUTES_MATCH_OUR_ITEMS,
                core::ptr::addr_of_mut!(result),
            );
        }
        _ => {
            WARN!("Unknown match type %d.\n", match_type);
            hr = E_INVALIDARG;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    if hr >= 0 {
        *ret = result;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetUINT32(attributes: *mut attributes, key: REFGUID, value: *mut UINT32) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT;

    PropVariantInit!(core::ptr::addr_of_mut!(attrval));
    attrval.vt = (VT_UI4 as u16);
    hr = attributes_get_item(attributes, key, core::ptr::addr_of_mut!(attrval));
    if hr >= 0 {
        *value = attrval.__bindgen_anon_1.ulVal;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetUINT64(attributes: *mut attributes, key: REFGUID, value: *mut UINT64) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT;

    PropVariantInit!(core::ptr::addr_of_mut!(attrval));
    attrval.vt = VT_UI8 as VARTYPE;
    hr = attributes_get_item(attributes, key, core::ptr::addr_of_mut!(attrval));
    if hr >= 0 as HRESULT {
        *value = attrval.__bindgen_anon_1.uhVal.QuadPart;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetDouble(
    attributes: *mut attributes,
    key: *const GUID,
    value: *mut f64,
) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    PropVariantInit!(core::ptr::addr_of_mut!(attrval));
    attrval.vt = VT_R8 as VARTYPE;
    hr = attributes_get_item(attributes, key, core::ptr::addr_of_mut!(attrval));
    if hr >= 0 {
        *value = attrval.__bindgen_anon_1.dblVal;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetGUID(attributes: *mut attributes, key: *const GUID, value: *mut GUID) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if !attribute.is_null() {
        if (*attribute).value.vt as ::core::ffi::c_int == MF_ATTRIBUTE_GUID as ::core::ffi::c_int {
            *value = *(*attribute).value.__bindgen_anon_1.puuid;
        } else {
            hr = MF_E_INVALIDTYPE;
        }
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetStringLength(attributes: *mut attributes, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut::<attribute>();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut::<size_t>());
    if !attribute.is_null() {
        if ((*attribute).value.vt as ::core::ffi::c_int) as u32 == MF_ATTRIBUTE_STRING {
            *length = lstrlenW((*attribute).value.__bindgen_anon_1.pwszVal as LPCWSTR) as UINT32;
        } else {
            hr = MF_E_INVALIDTYPE;
        }
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetString(
    mut attributes: *mut attributes,
    key: *const GUID,
    mut value: *mut WCHAR,
    mut size: UINT32,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut::<attribute>();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut::<size_t>());
    if !attribute.is_null() {
        if (*attribute).value.vt as core::ffi::c_int == MF_ATTRIBUTE_STRING as core::ffi::c_int {
            let mut len: core::ffi::c_int =
                lstrlenW((*attribute).value.__bindgen_anon_1.pwszVal as LPCWSTR);

            if !length.is_null() {
                *length = len as UINT32;
            }

            if size <= len as UINT32 {
                hr = STRSAFE_E_INSUFFICIENT_BUFFER;
            } else {
                memcpy(
                    value as *mut core::ffi::c_void,
                    (*attribute).value.__bindgen_anon_1.pwszVal as *const core::ffi::c_void,
                    ((len + 1 as core::ffi::c_int) as size_t)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                );
            }
        } else {
            hr = MF_E_INVALIDTYPE;
        }
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetAllocatedString(attributes: *mut attributes, key: *const GUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT;

    PropVariantInit!(core::ptr::addr_of_mut!(attrval));
    attrval.vt = VT_LPWSTR;
    hr = attributes_get_item(attributes, key, core::ptr::addr_of_mut!(attrval));
    if hr >= 0 {
        *value = attrval.__bindgen_anon_1.pwszVal as *mut WCHAR;
        if !length.is_null() {
            *length = lstrlenW(*value as LPCWSTR) as UINT32;
        }
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetBlobSize(
    mut attributes: *mut attributes,
    key: *const GUID,
    mut size: *mut UINT32,
) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if !attribute.is_null() {
        if (*attribute).value.vt as core::ffi::c_int == MF_ATTRIBUTE_BLOB as core::ffi::c_int {
            *size = (*attribute).value.__bindgen_anon_1.caub.cElems as UINT32;
        } else {
            hr = MF_E_INVALIDTYPE;
        }
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetBlob(attributes: *mut attributes, key: *const GUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if !attribute.is_null() {
        if (*attribute).value.vt as ::core::ffi::c_int == MF_ATTRIBUTE_BLOB as ::core::ffi::c_int {
            let mut size: UINT32 = (*attribute).value.__bindgen_anon_1.caub.cElems as UINT32;

            if bufsize >= size {
                hr = PropVariantToBuffer(
                    core::ptr::addr_of_mut!((*attribute).value),
                    buf as *mut core::ffi::c_void,
                    size,
                );
            } else {
                hr = E_NOT_SUFFICIENT_BUFFER;
            }

            if !blobsize.is_null() {
                *blobsize = size;
            }
        } else {
            hr = MF_E_INVALIDTYPE;
        }
    } else {
        hr = MF_E_ATTRIBUTENOTFOUND;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetAllocatedBlob(attributes: *mut attributes, key: *const GUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    attrval.vt = (VT_VECTOR | VT_UI1) as VARTYPE;
    hr = attributes_get_item(attributes, key, core::ptr::addr_of_mut!(attrval));
    if hr >= 0 {
        *buf = attrval.__bindgen_anon_1.caub.pElems as *mut UINT8;
        if !size.is_null() {
            *size = attrval.__bindgen_anon_1.caub.cElems as UINT32;
        }
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetUnknown(
    attributes: *mut attributes,
    key: *const GUID,
    riid: *const IID,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT;

    PropVariantInit!(core::ptr::addr_of_mut!(attrval));
    attrval.vt = VT_UNKNOWN as VARTYPE;
    hr = attributes_get_item(attributes, key, core::ptr::addr_of_mut!(attrval));
    if hr >= 0 as HRESULT {
        hr = IUnknown_QueryInterface(attrval.__bindgen_anon_1.punkVal, riid, out);
    }
    PropVariantClear(core::ptr::addr_of_mut!(attrval));
    return hr;
}

unsafe extern "C" fn attributes_set_item(attributes: *mut attributes, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    attribute = attributes_find_item(attributes, key, core::ptr::null_mut());
    if attribute.is_null() {
        if mf_array_reserve(
            core::ptr::addr_of_mut!((*attributes).attributes) as *mut *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*attributes).capacity),
            (*attributes).count.wrapping_add(1),
            core::mem::size_of::<attribute>(),
        ) == 0
        {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));
            return E_OUTOFMEMORY;
        }
        (*(*attributes).attributes.add((*attributes).count)).key = *key;
        let fresh15 = (*attributes).count;
        (*attributes).count = fresh15.wrapping_add(1);
        attribute = (*attributes).attributes.add(fresh15);
    } else {
        PropVariantClear(core::ptr::addr_of_mut!((*attribute).value));
    }

    PropVariantCopy(core::ptr::addr_of_mut!((*attribute).value), value);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetItem(attributes: *mut attributes, key: REFGUID, value: REFPROPVARIANT) -> HRESULT {
    let mut empty: PROPVARIANT = core::mem::zeroed();

    match ((*value).vt as ::core::ffi::c_int) as u32 {
        MF_ATTRIBUTE_UINT32 | MF_ATTRIBUTE_UINT64 | MF_ATTRIBUTE_DOUBLE | MF_ATTRIBUTE_GUID | MF_ATTRIBUTE_STRING | MF_ATTRIBUTE_BLOB | MF_ATTRIBUTE_IUNKNOWN => {
            return attributes_set_item(attributes, key, value);
        }
        _ => {
            PropVariantInit!(core::ptr::addr_of_mut!(empty));
            attributes_set_item(attributes, key, core::ptr::addr_of_mut!(empty));
            return MF_E_INVALIDTYPE;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn attributes_DeleteItem(attributes: *mut attributes, key: *const GUID) -> HRESULT {
    let mut attribute: *mut attribute = core::ptr::null_mut::<attribute>();
    let mut index: usize = 0;

    EnterCriticalSection(&raw mut (*attributes).cs);

    attribute = attributes_find_item(attributes, key, &raw mut index);
    if !attribute.is_null() {
        let mut count: usize;

        PropVariantClear(&raw mut (*attribute).value);

        (*attributes).count = (*attributes).count.wrapping_sub(1);
        count = (*attributes).count.wrapping_sub(index);
        if count != 0 {
            memmove(
                (*attributes).attributes.add(index) as *mut core::ffi::c_void,
                (*attributes).attributes.add(index + 1) as *const core::ffi::c_void,
                count.wrapping_mul(core::mem::size_of::<attribute>()),
            );
        }
    }

    LeaveCriticalSection(&raw mut (*attributes).cs);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_DeleteAllItems(attributes: *mut attributes) -> HRESULT {
    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    while (*attributes).count != 0 {
        (*attributes).count = (*attributes).count.wrapping_sub(1);
        PropVariantClear(core::ptr::addr_of_mut!(
            (*(*attributes).attributes.offset((*attributes).count as isize)).value
        ));
    }
    free((*attributes).attributes as *mut core::ffi::c_void);
    (*attributes).attributes = core::ptr::null_mut();
    (*attributes).capacity = 0 as size_t;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetUINT32(attributes: *mut attributes, key: REFGUID, value: UINT32) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();

    attrval.vt = (VT_UI4 as u16);
    attrval.__bindgen_anon_1.ulVal = value;
    return attributes_set_item(attributes, key, core::ptr::addr_of_mut!(attrval));
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetUINT64(
    attributes: *mut attributes,
    key: REFGUID,
    value: UINT64,
) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    attrval.vt = (VT_UI8 as u16);
    attrval.__bindgen_anon_1.uhVal.QuadPart = value;
    return attributes_set_item(attributes, key, &mut attrval);
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetDouble(attributes: *mut attributes, key: REFGUID, value: f64) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    attrval.vt = VT_R8 as VARTYPE;
    (*(&raw mut attrval)).__bindgen_anon_1.dblVal = value;
    return attributes_set_item(attributes, key, &raw mut attrval);
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetGUID(
    attributes: *mut attributes,
    key: *const GUID,
    value: *const GUID,
) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();

    attrval.vt = VT_CLSID as VARTYPE;
    attrval.__bindgen_anon_1.puuid = value as *mut CLSID;
    return attributes_set_item(attributes, key, core::ptr::addr_of_mut!(attrval));
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetString(attributes: *mut attributes, key: REFGUID, value: *const WCHAR) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();
    attrval.vt = VT_LPWSTR as VARTYPE;
    attrval.__bindgen_anon_1.pwszVal = value as *mut WCHAR;
    return attributes_set_item(attributes, key, &mut attrval);
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetBlob(
    attributes: *mut attributes,
    key: *const GUID,
    buf: *const UINT8,
    size: UINT32,
) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed();

    attrval.vt = (VT_VECTOR | VT_UI1) as VARTYPE;
    attrval.__bindgen_anon_1.caub.cElems = size;
    attrval.__bindgen_anon_1.caub.pElems = buf as *mut UINT8;
    return attributes_set_item(attributes, key, core::ptr::addr_of_mut!(attrval));
}
#[no_mangle]
pub unsafe extern "C" fn attributes_SetUnknown(attributes: *mut attributes, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mut attrval: PROPVARIANT = core::mem::zeroed::<PROPVARIANT>();
    attrval.vt = VT_UNKNOWN as VARTYPE;
    attrval.__bindgen_anon_1.punkVal = unknown;
    return attributes_set_item(attributes, key, core::ptr::addr_of_mut!(attrval));
}
#[no_mangle]
pub unsafe extern "C" fn attributes_LockStore(attributes: *mut attributes) -> HRESULT {
    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_UnlockStore(attributes: *mut attributes) -> HRESULT {
    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetCount(
    attributes: *mut attributes,
    count: *mut UINT32,
) -> HRESULT {
    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));
    *count = (*attributes).count as UINT32;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn attributes_GetItemByIndex(attributes: *mut attributes, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    if (index as usize) < (*attributes).count {
        *key = (*(*attributes).attributes.add(index as usize)).key;
        if !value.is_null() {
            PropVariantCopy(value, core::ptr::addr_of_mut!((*(*attributes).attributes.add(index as usize)).value));
        }
    } else {
        hr = E_INVALIDARG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_CopyAllItems(
    attributes: *mut attributes,
    dest: *mut IMFAttributes,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut i: size_t;

    EnterCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    IMFAttributes_LockStore(dest);

    IMFAttributes_DeleteAllItems(dest);

    i = 0;
    while i < (*attributes).count {
        hr = IMFAttributes_SetItem(
            dest,
            core::ptr::addr_of_mut!((*(*attributes).attributes.offset(i as isize)).key),
            core::ptr::addr_of_mut!((*(*attributes).attributes.offset(i as isize)).value),
        );
        if (hr) < 0 {
            break;
        }
        i = i.wrapping_add(1);
    }

    IMFAttributes_UnlockStore(dest);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*attributes).cs));

    return hr;
}

unsafe extern "system" fn mfattributes_GetItem(iface: *mut IMFAttributes, key: *const GUID, value: *mut PROPVARIANT) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);
    return attributes_GetItem(attributes, key, value);
}

unsafe extern "system" fn mfattributes_GetItemType(iface: *mut IMFAttributes, key: REFGUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);

    return attributes_GetItemType(attributes, key, type_);
}

unsafe extern "system" fn mfattributes_CompareItem(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    value: *const PROPVARIANT,
    mut result: *mut BOOL,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfattributes_CompareItem\0" as *const u8 as *const ::core::ffi::c_char,
            3032 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return attributes_CompareItem(attributes, key, value, result);
}

unsafe extern "system" fn mfattributes_Compare(
    mut iface: *mut IMFAttributes,
    mut theirs: *mut IMFAttributes,
    mut match_type: MF_ATTRIBUTES_MATCH_TYPE,
    mut ret: *mut BOOL,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfattributes_Compare\0" as *const u8 as *const ::core::ffi::c_char,
            3042 as ::core::ffi::c_int,
            b"%p, %p, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            theirs,
            match_type as ::core::ffi::c_uint,
            ret,
        );
    }
    return attributes_Compare(attributes, theirs, match_type, ret);
}

unsafe extern "system" fn mfattributes_GetUINT32(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut UINT32,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_GetUINT32\0".as_ptr() as *const i8,
            3051i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT32(attributes, key, value);
}

unsafe extern "system" fn mfattributes_GetUINT64(iface: *mut IMFAttributes, key: REFGUID, value: *mut UINT64) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);

    return attributes_GetUINT64(attributes, key, value);
}
unsafe extern "system" fn mfattributes_GetDouble(iface: *mut IMFAttributes, key: *const GUID, value: *mut f64) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_GetDouble\0".as_ptr() as *const i8,
            3069i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetDouble(attributes, key, value);
}
unsafe extern "system" fn mfattributes_GetGUID(
    iface: *mut IMFAttributes,
    key: REFGUID,
    value: *mut GUID,
) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (3 as i32))) != 0 {
        // __WINE_DBCL_TRACE == 3
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = 3 as __wine_debug_class;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const i8,
            b"mfattributes_GetGUID\0".as_ptr() as *const i8,
            3078 as i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetGUID(attributes, key, value);
}

unsafe extern "system" fn mfattributes_GetStringLength(iface: *mut IMFAttributes, key: REFGUID, length: *mut UINT32) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_GetStringLength\0".as_ptr() as *const i8,
            3087 as i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            length,
        );
    }
    return attributes_GetStringLength(attributes, key, length);
}
unsafe extern "system" fn mfattributes_GetString(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut WCHAR,
    mut size: UINT32,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfattributes_GetString\0" as *const u8 as *const ::core::ffi::c_char,
            3097 as ::core::ffi::c_int,
            b"%p, %s, %p, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
            size,
            length,
        );
    }
    return attributes_GetString(attributes, key, value, size, length);
}

unsafe extern "system" fn mfattributes_GetAllocatedString(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut *mut WCHAR,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"mfattributes_GetAllocatedString\0" as *const u8 as *const i8,
            3106 as i32,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
            length,
        );
    }
    return attributes_GetAllocatedString(attributes, key, value, length);
}

unsafe extern "system" fn mfattributes_GetBlobSize(iface: *mut IMFAttributes, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfattributes_GetBlobSize\0" as *const u8 as *const ::core::ffi::c_char,
            3115 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            size,
        );
    }
    return attributes_GetBlobSize(attributes, key, size);
}

unsafe extern "system" fn mfattributes_GetBlob(
    iface: *mut IMFAttributes,
    key: *const GUID,
    buf: *mut UINT8,
    bufsize: UINT32,
    blobsize: *mut UINT32,
) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"mfattributes_GetBlob\0" as *const u8 as *const i8,
            3125 as i32,
            b"%p, %s, %p, %d, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            buf,
            bufsize,
            blobsize,
        );
    }
    return attributes_GetBlob(attributes, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn mfattributes_GetAllocatedBlob(
    iface: *mut IMFAttributes,
    key: REFGUID,
    buf: *mut *mut UINT8,
    size: *mut UINT32,
) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if (*(*core::ptr::addr_of!(__wine_dbch___default))).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = core::ptr::addr_of!(__wine_dbch___default) as *mut __wine_debug_channel;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"mfattributes_GetAllocatedBlob\0" as *const u8 as *const core::ffi::c_char,
            3134 as core::ffi::c_int,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            buf,
            size,
        );
    }
    return attributes_GetAllocatedBlob(attributes, key, buf, size);
}

unsafe extern "system" fn mfattributes_GetUnknown(
    iface: *mut IMFAttributes,
    key: *const GUID,
    riid: *const IID,
    out: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfattributes_GetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            3143 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_guid(riid as *const GUID),
            out,
        );
    }
    return attributes_GetUnknown(attributes, key, riid, out);
}

unsafe extern "system" fn mfattributes_SetItem(iface: *mut IMFAttributes, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_SetItem\0".as_ptr() as *const i8,
            3152i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
        );
    }
    return attributes_SetItem(attributes, key, value);
}

unsafe extern "system" fn mfattributes_DeleteItem(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_DeleteItem\0".as_ptr() as *const i8,
            3161 as i32,
            b"%p, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
        );
    }
    return attributes_DeleteItem(attributes, key);
}

unsafe extern "system" fn mfattributes_DeleteAllItems(iface: *mut IMFAttributes) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);

    return attributes_DeleteAllItems(attributes);
}

unsafe extern "system" fn mfattributes_SetUINT32(iface: *mut IMFAttributes, key: REFGUID, value: UINT32) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);

    return attributes_SetUINT32(attributes, key, value);
}

unsafe extern "system" fn mfattributes_SetUINT64(iface: *mut IMFAttributes, key: REFGUID, value: UINT64) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_SetUINT64\0".as_ptr() as *const i8,
            3188i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return attributes_SetUINT64(attributes, key, value);
}
unsafe extern "system" fn mfattributes_SetDouble(iface: *mut IMFAttributes, key: REFGUID, value: f64) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    TRACE!("%p, %s, %f.\n", iface, debugstr_attr(key), value);
    return attributes_SetDouble(attributes, key, value);
}

unsafe extern "system" fn mfattributes_SetGUID(iface: *mut IMFAttributes, key: *const GUID, value: *const GUID) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    TRACE!("%p, %s, %s.\n", iface, debugstr_attr(key), debugstr_mf_guid(value));
    return attributes_SetGUID(attributes, key, value);
}

unsafe extern "system" fn mfattributes_SetString(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *const WCHAR,
) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int)
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"mfattributes_SetString\0" as *const u8 as *const core::ffi::c_char,
            3215 as core::ffi::c_int,
            b"%p, %s, %s.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_w(value),
        );
    }
    return attributes_SetString(attributes, key, value);
}

unsafe extern "system" fn mfattributes_SetBlob(
    iface: *mut IMFAttributes,
    key: REFGUID,
    buf: *const UINT8,
    size: UINT32,
) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_SetBlob\0".as_ptr() as *const i8,
            3224 as i32,
            b"%p, %s, %p, %u.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            buf,
            size,
        );
    }
    return attributes_SetBlob(attributes, key, buf, size);
}

unsafe extern "system" fn mfattributes_SetUnknown(iface: *mut IMFAttributes, key: REFGUID, unknown: *mut IUnknown) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);

    return attributes_SetUnknown(attributes, key, unknown);
}

unsafe extern "system" fn mfattributes_LockStore(iface: *mut IMFAttributes) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);

    return attributes_LockStore(attributes);
}

unsafe extern "system" fn mfattributes_UnlockStore(iface: *mut IMFAttributes) -> HRESULT {
    let attributes: *mut attributes = impl_from_IMFAttributes(iface);

    return attributes_UnlockStore(attributes);
}

unsafe extern "system" fn mfattributes_GetCount(
    iface: *mut IMFAttributes,
    count: *mut UINT32,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const ::core::ffi::c_char,
            b"mfattributes_GetCount\0".as_ptr() as *const ::core::ffi::c_char,
            3260 as ::core::ffi::c_int,
            b"%p, %p.\n\0".as_ptr() as *const ::core::ffi::c_char,
            iface,
            count,
        );
    }
    return attributes_GetCount(attributes, count);
}

unsafe extern "system" fn mfattributes_GetItemByIndex(iface: *mut IMFAttributes, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    TRACE!("%p, %u, %p, %p.\n", iface, index, key, value);
    return attributes_GetItemByIndex(attributes, index, key, value);
}

unsafe extern "system" fn mfattributes_CopyAllItems(
    mut iface: *mut IMFAttributes,
    mut dest: *mut IMFAttributes,
) -> HRESULT {
    let mut attributes: *mut attributes = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfattributes_CopyAllItems\0".as_ptr() as *const i8,
            3278 as i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface,
            dest,
        );
    }
    return attributes_CopyAllItems(attributes, dest);
}
#[no_mangle]
pub unsafe extern "C" fn init_attributes_object(object: *mut attributes, size: UINT32) -> HRESULT {
    (*object).IMFAttributes_iface.lpVtbl = &raw const mfattributes_vtbl;
    (*object).ref_ = 1 as LONG;
    InitializeCriticalSection(&raw mut (*object).cs);

    (*object).attributes = core::ptr::null_mut();
    (*object).count = 0 as size_t;
    (*object).capacity = 0 as size_t;
    if mf_array_reserve(
        &raw mut (*object).attributes as *mut *mut core::ffi::c_void,
        &raw mut (*object).capacity,
        size as size_t,
        core::mem::size_of::<attribute>() as size_t,
    ) == 0
    {
        DeleteCriticalSection(&raw mut (*object).cs);
        return E_OUTOFMEMORY;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn clear_attributes_object(object: *mut attributes) {
    let mut i: size_t = 0;
    i = 0;
    while i < (*object).count {
        PropVariantClear(core::ptr::addr_of_mut!((*(*object).attributes.add(i as usize)).value));
        i = i.wrapping_add(1);
    }
    free((*object).attributes as *mut core::ffi::c_void);

    DeleteCriticalSection(core::ptr::addr_of_mut!((*object).cs));
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateAttributes(attributes: *mut *mut IMFAttributes, size: UINT32) -> HRESULT {
    let mut object: *mut attributes = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    object = calloc(1, (core::mem::size_of::<attributes>()) as _) as *mut attributes;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(object, size);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }
    *attributes = core::ptr::addr_of_mut!((*object).IMFAttributes_iface);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFGetAttributesAsBlobSize(attributes: *mut IMFAttributes, size: *mut UINT32) -> HRESULT {
    let mut i: UINT32;
    let mut count: UINT32 = 0;
    let mut length: UINT32 = 0;
    let mut hr: HRESULT = 0;
    let mut key: GUID = core::mem::zeroed();

    IMFAttributes_LockStore(attributes);

    hr = IMFAttributes_GetCount(attributes, core::ptr::addr_of_mut!(count));

    *size = core::mem::size_of::<attributes_store_header>() as UINT32;

    i = 0 as UINT32;
    while i < count {
        let mut type_: MF_ATTRIBUTE_TYPE = 0;

        hr = IMFAttributes_GetItemByIndex(attributes, i, core::ptr::addr_of_mut!(key), core::ptr::null_mut());
        if hr < 0 {
            break;
        }

        *size = (*size).wrapping_add(core::mem::size_of::<attributes_store_item>() as UINT32);

        IMFAttributes_GetItemType(attributes, core::ptr::addr_of_mut!(key), core::ptr::addr_of_mut!(type_));

        match type_ as UINT32 {
            t if t == MF_ATTRIBUTE_GUID as UINT32 => {
                *size = (*size).wrapping_add(core::mem::size_of::<GUID>() as UINT32);
            }
            t if t == MF_ATTRIBUTE_STRING as UINT32 => {
                IMFAttributes_GetStringLength(attributes, core::ptr::addr_of_mut!(key), core::ptr::addr_of_mut!(length));
                *size = (*size).wrapping_add(
                    length.wrapping_add(1 as UINT32)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as UINT32),
                );
            }
            t if t == MF_ATTRIBUTE_BLOB as UINT32 => {
                IMFAttributes_GetBlobSize(attributes, core::ptr::addr_of_mut!(key), core::ptr::addr_of_mut!(length));
                *size = (*size).wrapping_add(length);
            }
            t if t == MF_ATTRIBUTE_UINT32 as UINT32
                || t == MF_ATTRIBUTE_UINT64 as UINT32
                || t == MF_ATTRIBUTE_DOUBLE as UINT32
                || t == MF_ATTRIBUTE_IUNKNOWN as UINT32 =>
            {
                ;
            }
            _ => {
                ;
            }
        }

        i = i.wrapping_add(1);
    }

    IMFAttributes_UnlockStore(attributes);

    return hr;
}

unsafe extern "C" fn attributes_serialize_write(
    context: *mut attr_serialize_context,
    value: *const core::ffi::c_void,
    size: u32,
) {
    memcpy(
        (*context).ptr as *mut core::ffi::c_void,
        value,
        size as usize,
    );
    (*context).ptr = (*context).ptr.offset(size as isize);
}

unsafe extern "C" fn attributes_serialize_write_item(
    mut context: *mut attr_serialize_context,
    mut item: *mut attributes_store_item,
    mut value: *const ::core::ffi::c_void,
) -> BOOL {
    match (*item).type_ {
        __c if __c == MF_ATTRIBUTE_UINT32 as _ || __c == MF_ATTRIBUTE_UINT64 as _ || __c == MF_ATTRIBUTE_DOUBLE as _ => {
            attributes_serialize_write(
                context,
                item as *const ::core::ffi::c_void,
                ::core::mem::size_of::<attributes_store_item>() as ::core::ffi::c_uint,
            );
        }
        __c if __c == MF_ATTRIBUTE_GUID as _ || __c == MF_ATTRIBUTE_STRING as _ || __c == MF_ATTRIBUTE_BLOB as _ => {
            (*item).u.subheader.offset =
                ((*context).size as DWORD).wrapping_sub((*item).u.subheader.size);
            attributes_serialize_write(
                context,
                item as *const ::core::ffi::c_void,
                ::core::mem::size_of::<attributes_store_item>() as ::core::ffi::c_uint,
            );
            memcpy(
                (*context).buffer.offset((*item).u.subheader.offset as isize)
                    as *mut ::core::ffi::c_void,
                value,
                (*item).u.subheader.size as usize,
            );
            (*context).size = ((*context).size as DWORD).wrapping_sub((*item).u.subheader.size)
                as UINT32 as UINT32;
        }
        _ => return (FALSE as i32),
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MFGetAttributesAsBlob(attributes: *mut IMFAttributes, buffer: *mut UINT8, size: UINT) -> HRESULT {
    let mut header: attributes_store_header = attributes_store_header { magic: 0, count: 0 };
    let mut context: attr_serialize_context = attr_serialize_context {
        buffer: core::ptr::null_mut::<UINT8>(),
        ptr: core::ptr::null_mut::<UINT8>(),
        size: 0,
    };
    let mut required_size: core::ffi::c_uint = 0;
    let mut i: core::ffi::c_uint = 0;
    let mut value: PROPVARIANT = core::mem::zeroed();
    let mut count: UINT32 = 0;
    let mut hr: HRESULT = 0;

    hr = MFGetAttributesAsBlobSize(attributes, core::ptr::addr_of_mut!(required_size));
    if hr < 0 {
        return hr;
    }

    if required_size > size {
        return MF_E_BUFFERTOOSMALL;
    }

    context.buffer = buffer;
    context.ptr = buffer;
    context.size = required_size as UINT32;

    IMFAttributes_LockStore(attributes);

    header.magic = ATTRIBUTES_STORE_MAGIC as DWORD;
    header.count = 0 as UINT32; /* Will be updated later */
    IMFAttributes_GetCount(attributes, core::ptr::addr_of_mut!(count));

    attributes_serialize_write(
        core::ptr::addr_of_mut!(context),
        core::ptr::addr_of_mut!(header) as *const core::ffi::c_void,
        core::mem::size_of::<attributes_store_header>() as core::ffi::c_uint,
    );

    i = 0 as core::ffi::c_uint;
    while i < count {
        let mut item: attributes_store_item = core::mem::zeroed();
        let mut data: *const core::ffi::c_void = core::ptr::null::<core::ffi::c_void>();

        hr = IMFAttributes_GetItemByIndex(
            attributes,
            i as UINT32,
            core::ptr::addr_of_mut!(item.key),
            core::ptr::addr_of_mut!(value),
        );
        if hr < 0 {
            break;
        }

        item.type_ = value.vt as QWORD;

        match (value.vt as core::ffi::c_int) as u32 {
            MF_ATTRIBUTE_UINT32 | MF_ATTRIBUTE_UINT64 => {
                item.u.i64_0 = value.__bindgen_anon_1.uhVal.QuadPart as UINT64;
            }
            MF_ATTRIBUTE_DOUBLE => {
                item.u.f = value.__bindgen_anon_1.dblVal;
            }
            MF_ATTRIBUTE_GUID => {
                item.u.subheader.size = core::mem::size_of::<CLSID>() as DWORD;
                data = value.__bindgen_anon_1.puuid as *const core::ffi::c_void;
            }
            MF_ATTRIBUTE_STRING => {
                item.u.subheader.size = ((lstrlenW(value.__bindgen_anon_1.pwszVal as LPCWSTR)
                    + 1 as core::ffi::c_int) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
                    as DWORD;
                data = value.__bindgen_anon_1.pwszVal as *const core::ffi::c_void;
            }
            MF_ATTRIBUTE_BLOB => {
                item.u.subheader.size = value.__bindgen_anon_1.caub.cElems as DWORD;
                data = value.__bindgen_anon_1.caub.pElems as *const core::ffi::c_void;
            }
            MF_ATTRIBUTE_IUNKNOWN => {}
            _ => {
                WARN!("Unknown attribute type %#x.\n", value.vt as core::ffi::c_int);
            }
        }

        if attributes_serialize_write_item(
            core::ptr::addr_of_mut!(context),
            core::ptr::addr_of_mut!(item),
            data,
        ) != 0
        {
            header.count = header.count.wrapping_add(1);
        }

        PropVariantClear(core::ptr::addr_of_mut!(value));
        i = i.wrapping_add(1);
    }

    memcpy(
        context.buffer as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(header) as *const core::ffi::c_void,
        core::mem::size_of::<attributes_store_header>() as size_t,
    );

    IMFAttributes_UnlockStore(attributes);

    return S_OK;
}

unsafe extern "C" fn attributes_deserialize_read(context: *mut attr_serialize_context, value: *mut core::ffi::c_void, size: UINT32) -> HRESULT {
    if ((*context).size as i64) < (((*context).ptr.offset_from((*context).buffer)) as i64 + size as i64) {
        return E_INVALIDARG;
    }

    memcpy(value, (*context).ptr as *const core::ffi::c_void, size as usize);
    (*context).ptr = (*context).ptr.offset(size as isize);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFInitAttributesFromBlob(dest: *mut IMFAttributes, buffer: *const UINT8, size: UINT) -> HRESULT {
    let mut context: attr_serialize_context = attr_serialize_context {
        buffer: buffer as *mut UINT8,
        ptr: buffer as *mut UINT8,
        size: size as UINT32,
    };
    let mut header: attributes_store_header = attributes_store_header { magic: 0, count: 0 };
    let mut item: attributes_store_item = core::mem::zeroed();
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    let mut i: core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    context.buffer = buffer as *mut UINT8;
    context.ptr = buffer as *mut UINT8;
    context.size = size as UINT32;

    hr = attributes_deserialize_read(
        core::ptr::addr_of_mut!(context),
        core::ptr::addr_of_mut!(header) as *mut core::ffi::c_void,
        core::mem::size_of::<attributes_store_header>() as core::ffi::c_uint,
    );
    if hr < 0 {
        return hr;
    }

    if header.magic != ATTRIBUTES_STORE_MAGIC as DWORD {
        return E_UNEXPECTED;
    }

    hr = MFCreateAttributes(core::ptr::addr_of_mut!(attributes), header.count);
    if hr < 0 {
        return hr;
    }

    i = 0 as core::ffi::c_uint;
    while i < header.count {
        hr = attributes_deserialize_read(
            core::ptr::addr_of_mut!(context),
            core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
            core::mem::size_of::<attributes_store_item>() as core::ffi::c_uint,
        );
        if hr < 0 {
            break;
        }

        hr = E_UNEXPECTED;

        match item.type_ {
            __c if __c == MF_ATTRIBUTE_UINT32 as _ => {
                hr = IMFAttributes_SetUINT32(attributes, core::ptr::addr_of_mut!(item.key), item.u.i32_0);
            }
            __c if __c == MF_ATTRIBUTE_UINT64 as _ => {
                hr = IMFAttributes_SetUINT64(attributes, core::ptr::addr_of_mut!(item.key), item.u.i64_0);
            }
            __c if __c == MF_ATTRIBUTE_DOUBLE as _ => {
                hr = IMFAttributes_SetDouble(attributes, core::ptr::addr_of_mut!(item.key), item.u.f);
            }
            __c if __c == MF_ATTRIBUTE_GUID as _ => {
                if (item.u.subheader.size as usize) == core::mem::size_of::<GUID>()
                    && item.u.subheader.offset.wrapping_add(item.u.subheader.size) <= context.size as DWORD
                {
                    hr = IMFAttributes_SetGUID(
                        attributes,
                        core::ptr::addr_of_mut!(item.key),
                        context.buffer.add(item.u.subheader.offset as usize) as *const GUID,
                    );
                }
            }
            __c if __c == MF_ATTRIBUTE_STRING as _ => {
                if (item.u.subheader.size as usize) >= core::mem::size_of::<WCHAR>()
                    && item.u.subheader.offset.wrapping_add(item.u.subheader.size) <= context.size as DWORD
                {
                    hr = IMFAttributes_SetString(
                        attributes,
                        core::ptr::addr_of_mut!(item.key),
                        context.buffer.add(item.u.subheader.offset as usize) as *const WCHAR,
                    );
                }
            }
            __c if __c == MF_ATTRIBUTE_BLOB as _ => {
                if item.u.subheader.size > 0 as DWORD
                    && item.u.subheader.offset.wrapping_add(item.u.subheader.size) <= context.size as DWORD
                {
                    hr = IMFAttributes_SetBlob(
                        attributes,
                        core::ptr::addr_of_mut!(item.key),
                        context.buffer.add(item.u.subheader.offset as usize) as *const UINT8,
                        item.u.subheader.size as UINT32,
                    );
                }
            }
            _ => {}
        }

        if hr < 0 {
            break;
        }
        i = i.wrapping_add(1);
    }

    if hr >= 0 {
        IMFAttributes_DeleteAllItems(dest);
        hr = IMFAttributes_CopyAllItems(attributes, dest);
    }

    IMFAttributes_Release(attributes);

    return hr;
}

unsafe extern "C" fn impl_from_IMFByteStream(iface: *mut IMFByteStream) -> *mut bytestream {
    return CONTAINING_RECORD!(iface, bytestream, IMFByteStream_iface);
}

unsafe extern "C" fn impl_bytestream_from_IMFGetService(iface: *mut IMFGetService) -> *mut bytestream {
    return CONTAINING_RECORD!(iface, bytestream, IMFGetService_iface);
}

unsafe extern "C" fn impl_from_read_callback_IRtwqAsyncCallback(iface: *mut IRtwqAsyncCallback) -> *mut bytestream {
    return (iface as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(bytestream, read_callback) as ULONG_PTR)
        as *mut bytestream;
}

unsafe extern "C" fn impl_from_write_callback_IRtwqAsyncCallback(
    iface: *mut IRtwqAsyncCallback,
) -> *mut bytestream {
    return (iface as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(bytestream, write_callback) as ULONG_PTR)
        as *mut bytestream;
}

unsafe extern "C" fn impl_async_stream_op_from_IUnknown(iface: *mut IUnknown) -> *mut async_stream_op {
    return CONTAINING_RECORD!(iface, async_stream_op, IUnknown_iface);
}
unsafe extern "system" fn async_stream_op_QueryInterface(iface: *mut IUnknown, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn async_stream_op_AddRef(iface: *mut IUnknown) -> ULONG {
    let op: *mut async_stream_op = impl_async_stream_op_from_IUnknown(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*op).refcount)) as ULONG;
    TRACE!("%p, refcount %ld.\n", iface, refcount);
    return refcount;
}

unsafe extern "system" fn async_stream_op_Release(iface: *mut IUnknown) -> ULONG {
    let op: *mut async_stream_op = impl_async_stream_op_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*op).refcount) as *mut i32) as ULONG;

    if refcount == 0 {
        if !(*op).caller.is_null() {
            IMFAsyncResult_Release((*op).caller);
        }
        free(op as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn bytestream_create_io_request(
    stream: *mut bytestream,
    type_0: async_stream_op_type,
    data: *const BYTE,
    size: ULONG,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let mut op: *mut async_stream_op = ::core::ptr::null_mut::<async_stream_op>();
    let mut request: *mut IRtwqAsyncResult = ::core::ptr::null_mut::<IRtwqAsyncResult>();
    let mut hr: HRESULT = 0;

    op = malloc(core::mem::size_of::<async_stream_op>() as usize) as *mut async_stream_op;
    if op.is_null() {
        return E_OUTOFMEMORY as HRESULT;
    }

    (*op).IUnknown_iface.lpVtbl = core::ptr::addr_of!(async_stream_op_vtbl);
    (*op).refcount = 1 as LONG;
    (*op).u.src = data;
    (*op).position = (*stream).position;
    (*op).requested_length = size;
    (*op).r#type = type_0 as i32;

    'failed: {
        hr = RtwqCreateAsyncResult(
            core::ptr::addr_of_mut!((*stream).IMFByteStream_iface) as *mut IUnknown,
            callback as *mut IRtwqAsyncCallback,
            state,
            core::ptr::addr_of_mut!((*op).caller) as *mut *mut IRtwqAsyncResult,
        );
        if hr < 0 as HRESULT {
            break 'failed;
        }

        hr = RtwqCreateAsyncResult(
            core::ptr::addr_of_mut!((*op).IUnknown_iface),
            (if type_0 as ::core::ffi::c_uint
                == ASYNC_STREAM_OP_READ as ::core::ffi::c_uint
            {
                core::ptr::addr_of_mut!((*stream).read_callback)
            } else {
                core::ptr::addr_of_mut!((*stream).write_callback)
            }),
            ::core::ptr::null_mut::<IUnknown>(),
            core::ptr::addr_of_mut!(request),
        );
        if hr < 0 as HRESULT {
            break 'failed;
        }

        RtwqPutWorkItem(
            MFASYNC_CALLBACK_QUEUE_IO as DWORD,
            0 as LONG,
            request,
        );
        IRtwqAsyncResult_Release(request);
    }

    IUnknown_Release(core::ptr::addr_of_mut!((*op).IUnknown_iface));
    return hr;
}

unsafe extern "C" fn bytestream_complete_io_request(
    mut stream: *mut bytestream,
    mut type_0: async_stream_op_type,
    mut result: *mut IMFAsyncResult,
    mut actual_length: *mut ULONG,
) -> HRESULT {
    let mut op: *mut async_stream_op = core::ptr::null_mut::<async_stream_op>();
    let mut cur: *mut async_stream_op = core::ptr::null_mut::<async_stream_op>();
    let mut hr: HRESULT = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
    cur = ((*stream).pending.next as *mut u8)
        .sub(core::mem::offset_of!(async_stream_op, entry)) as *mut async_stream_op;
    while core::ptr::addr_of_mut!((*cur).entry) != core::ptr::addr_of_mut!((*stream).pending) {
        if (*cur).caller == result
            && (*cur).r#type as core::ffi::c_uint == type_0 as core::ffi::c_uint
        {
            op = cur;
            list_remove(core::ptr::addr_of_mut!((*cur).entry));
            break;
        } else {
            cur = ((*cur).entry.next as *mut u8)
                .sub(core::mem::offset_of!(async_stream_op, entry)) as *mut async_stream_op;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    if op.is_null() {
        return E_INVALIDARG;
    }

    hr = IMFAsyncResult_GetStatus(result);
    if hr >= 0 {
        *actual_length = (*op).actual_length;
    }

    IUnknown_Release(core::ptr::addr_of_mut!((*op).IUnknown_iface));

    return hr;
}
unsafe extern "system" fn bytestream_callback_QueryInterface(iface: *mut IRtwqAsyncCallback, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IRtwqAsyncCallback)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IRtwqAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn bytestream_read_callback_AddRef(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let stream: *mut bytestream = impl_from_read_callback_IRtwqAsyncCallback(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_read_callback_Release(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let stream: *mut bytestream = impl_from_read_callback_IRtwqAsyncCallback(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}
unsafe extern "system" fn bytestream_callback_GetParameters(
    iface: *mut IRtwqAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn bytestream_write_callback_AddRef(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let mut stream: *mut bytestream = impl_from_write_callback_IRtwqAsyncCallback(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_write_callback_Release(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let mut stream: *mut bytestream = impl_from_write_callback_IRtwqAsyncCallback(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_QueryInterface(iface: *mut IMFByteStream, riid: REFIID, out: *mut *mut c_void) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFByteStream))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
    {
        *out = core::ptr::addr_of_mut!((*stream).IMFByteStream_iface) as *mut c_void;
    }
    else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes))) != 0) {
        *out = core::ptr::addr_of_mut!((*stream).attributes.IMFAttributes_iface) as *mut c_void;
    }
    else if !(*stream).IMFGetService_iface.lpVtbl.is_null()
        && ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService))) != 0)
    {
        *out = core::ptr::addr_of_mut!((*stream).IMFGetService_iface) as *mut c_void;
    }
    else {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn bytestream_AddRef(iface: *mut IMFByteStream) -> ULONG {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*stream).attributes.ref_) as *mut i32,
    ) as ULONG;

    return refcount;
}

unsafe extern "system" fn bytestream_Release(iface: *mut IMFByteStream) -> ULONG {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*stream).attributes.ref_) as *mut core::ffi::c_long,
    ) as ULONG;
    let mut cur: *mut async_stream_op = core::ptr::null_mut();
    let mut cur2: *mut async_stream_op = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"bytestream_Release\0" as *const u8 as *const core::ffi::c_char,
            3939 as core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }

    if refcount == 0 {
        clear_attributes_object(core::ptr::addr_of_mut!((*stream).attributes));
        cur = ((*stream).pending.next as *mut u8)
            .sub(core::mem::offset_of!(async_stream_op, entry)) as *mut async_stream_op;
        cur2 = ((*cur).entry.next as *mut u8)
            .sub(core::mem::offset_of!(async_stream_op, entry)) as *mut async_stream_op;
        while core::ptr::addr_of_mut!((*cur).entry) != core::ptr::addr_of_mut!((*stream).pending) {
            list_remove(core::ptr::addr_of_mut!((*cur).entry));
            IUnknown_Release(core::ptr::addr_of_mut!((*cur).IUnknown_iface));
            cur = cur2;
            cur2 = ((*cur).entry.next as *mut u8)
                .sub(core::mem::offset_of!(async_stream_op, entry)) as *mut async_stream_op;
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
        if !(*stream).stream.is_null() {
            IStream_Release((*stream).stream);
        }
        if !(*stream).hfile.is_null() {
            CloseHandle((*stream).hfile);
        }
        free(stream as *mut core::ffi::c_void);
    }

    return refcount;
}
unsafe extern "system" fn bytestream_stream_GetCapabilities(iface: *mut IMFByteStream, capabilities: *mut DWORD) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut stat: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT;

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_stream_GetCapabilities\0".as_ptr() as *const i8,
            3966 as i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface,
            capabilities,
        );
    }

    hr = IStream_Stat((*stream).stream, &raw mut stat, STATFLAG_NONAME);
    if hr < 0 {
        return hr;
    }

    *capabilities = MFBYTESTREAM_IS_READABLE | MFBYTESTREAM_IS_SEEKABLE;
    if (stat.grfMode & ((STGM_WRITE | STGM_READWRITE) as DWORD)) != 0 {
        *capabilities |= MFBYTESTREAM_IS_WRITABLE;
    }

    return S_OK;
}
unsafe extern "system" fn bytestream_file_GetCapabilities(mut iface: *mut IMFByteStream, mut capabilities: *mut DWORD) -> HRESULT {
    let mut stream: *mut bytestream = impl_from_IMFByteStream(iface);
    *capabilities = (*stream).capabilities;
    return S_OK;
}
unsafe extern "system" fn bytestream_file_SetLength(
    iface: *mut IMFByteStream,
    length: QWORD,
) -> HRESULT {
    // FIXME("%p, %s\n", iface, wine_dbgstr_longlong(length));
    E_NOTIMPL
}

unsafe extern "system" fn bytestream_GetCurrentPosition(iface: *mut IMFByteStream, position: *mut QWORD) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);

    if position.is_null() {
        return E_INVALIDARG;
    }

    *position = (*stream).position;

    return S_OK;
}

unsafe extern "system" fn bytestream_file_GetLength(iface: *mut IMFByteStream, length: *mut QWORD) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut li: LARGE_INTEGER = core::mem::zeroed();

    if length.is_null() {
        return E_INVALIDARG;
    }

    if GetFileSizeEx((*stream).hfile, core::ptr::addr_of_mut!(li)) != 0 {
        *length = li.QuadPart as QWORD;
    } else {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    return S_OK;
}

unsafe extern "system" fn bytestream_file_IsEndOfStream(iface: *mut IMFByteStream, ret: *mut BOOL) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut length: LARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    position.QuadPart = 0;
    if SetFilePointerEx((*stream).hfile, position, core::ptr::addr_of_mut!(length), FILE_END) != 0 {
        *ret = ((*stream).position >= length.QuadPart as QWORD) as BOOL;
    } else {
        hr = HRESULT_FROM_WIN32!(GetLastError());
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn bytestream_file_Read(iface: *mut IMFByteStream, buffer: *mut BYTE, size: ULONG, read_len: *mut ULONG) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;
    let mut ret: BOOL;

    TRACE!("%p, %p, %lu, %p.\n", iface, buffer, size, read_len);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    position.QuadPart = (*stream).position as LONGLONG;
    ret = SetFilePointerEx((*stream).hfile, position, core::ptr::null_mut(), FILE_BEGIN as DWORD);
    if ret != 0 {
        ret = ReadFile((*stream).hfile, buffer as PVOID, size as DWORD, read_len as PDWORD, core::ptr::null_mut());
        if ret != 0 {
            (*stream).position = (*stream).position.wrapping_add(*read_len as QWORD);
        }
    }

    if ret == 0 {
        hr = HRESULT_FROM_WIN32!(GetLastError());
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn bytestream_BeginRead(iface: *mut IMFByteStream, data: *mut BYTE, size: ULONG, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);

    return bytestream_create_io_request(stream, (ASYNC_STREAM_OP_READ) as async_stream_op_type, data, size, callback, state);
}

unsafe extern "system" fn bytestream_EndRead(
    iface: *mut IMFByteStream,
    result: *mut IMFAsyncResult,
    byte_read: *mut ULONG,
) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_EndRead\0" as *const u8 as *const ::core::ffi::c_char,
            4089 as ::core::ffi::c_int,
            b"%p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            result,
            byte_read,
        );
    }
    return bytestream_complete_io_request(stream, (ASYNC_STREAM_OP_READ) as async_stream_op_type, result, byte_read);
}

unsafe extern "system" fn bytestream_file_Write(iface: *mut IMFByteStream, data: *const BYTE, size: ULONG, written: *mut ULONG) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;
    let mut ret: BOOL = 0;

    TRACE!("%p, %p, %lu, %p\n", iface, data, size, written);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    position.QuadPart = (*stream).position as LONGLONG;
    ret = SetFilePointerEx((*stream).hfile, position, core::ptr::null_mut(), FILE_BEGIN);
    if ret != 0 {
        ret = WriteFile((*stream).hfile, data as LPCVOID, size as DWORD, written as LPDWORD, core::ptr::null_mut());
        if ret != 0 {
            (*stream).position = (*stream).position.wrapping_add(*written as QWORD);
        }
    }

    if ret == 0 {
        hr = HRESULT_FROM_WIN32!(GetLastError());
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn bytestream_BeginWrite(
    iface: *mut IMFByteStream,
    data: *const BYTE,
    size: ULONG,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"bytestream_BeginWrite\0" as *const u8 as *const i8,
            4125i32,
            b"%p, %p, %lu, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            data,
            size,
            callback,
            state,
        );
    }
    return bytestream_create_io_request(
        stream,
        (ASYNC_STREAM_OP_WRITE) as async_stream_op_type,
        data,
        size,
        callback,
        state,
    );
}

unsafe extern "system" fn bytestream_EndWrite(iface: *mut IMFByteStream, result: *mut IMFAsyncResult, written: *mut ULONG) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_EndWrite\0".as_ptr() as *const i8,
            4134i32,
            b"%p, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            result,
            written,
        );
    }
    return bytestream_complete_io_request(stream, (ASYNC_STREAM_OP_WRITE) as async_stream_op_type, result, written);
}

unsafe extern "system" fn bytestream_Seek(iface: *mut IMFByteStream, origin: MFBYTESTREAM_SEEK_ORIGIN, offset: LONGLONG, flags: DWORD, current: *mut QWORD) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    match origin as u32 {
        0 => {
            (*stream).position = offset as QWORD;
        }
        1 => {
            (*stream).position = (*stream).position.wrapping_add(offset as QWORD);
        }
        _ => {
            WARN!("Unknown origin mode %d.\n", origin);
            hr = E_INVALIDARG;
        }
    }

    *current = (*stream).position;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}
unsafe extern "system" fn bytestream_file_Flush(iface: *mut IMFByteStream) -> HRESULT {
    // FIXME("%p\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn bytestream_file_Close(iface: *mut IMFByteStream) -> HRESULT {
    // FIXME("%p\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn bytestream_SetCurrentPosition(iface: *mut IMFByteStream, position: QWORD) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
    (*stream).position = position;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return S_OK;
}

unsafe extern "system" fn bytestream_stream_GetLength(iface: *mut IMFByteStream, length: *mut QWORD) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut statstg: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    hr = IStream_Stat((*stream).stream, core::ptr::addr_of_mut!(statstg), STATFLAG_NONAME);
    if hr < 0 {
        return hr;
    }

    *length = statstg.cbSize.QuadPart;

    return S_OK;
}

unsafe extern "system" fn bytestream_stream_SetLength(iface: *mut IMFByteStream, length: QWORD) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut size: ULARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    size.QuadPart = length as UINT64;
    hr = IStream_SetSize((*stream).stream, size);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn bytestream_stream_IsEndOfStream(iface: *mut IMFByteStream, ret: *mut BOOL) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut statstg: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    hr = IStream_Stat((*stream).stream, core::ptr::addr_of_mut!(statstg), STATFLAG_NONAME);
    if hr >= 0 {
        *ret = ((*stream).position >= statstg.cbSize.QuadPart) as BOOL;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn bytestream_stream_Read(iface: *mut IMFByteStream, buffer: *mut BYTE, size: ULONG, read_len: *mut ULONG) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT;

    if ((*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_stream_Read\0".as_ptr() as *const i8,
            4276i32,
            b"%p, %p, %lu, %p.\n\0".as_ptr() as *const i8,
            iface,
            buffer,
            size,
            read_len,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    position.QuadPart = (*stream).position as LONGLONG;
    hr = IStream_Seek(
        (*stream).stream,
        position,
        STREAM_SEEK_SET as DWORD,
        core::ptr::null_mut::<ULARGE_INTEGER>(),
    );
    if hr >= 0 as HRESULT {
        hr = IStream_Read(
            (*stream).stream,
            buffer as *mut core::ffi::c_void,
            size,
            read_len,
        );
        if hr >= 0 as HRESULT {
            (*stream).position = (*stream).position.wrapping_add(*read_len as QWORD);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn bytestream_stream_Write(iface: *mut IMFByteStream, buffer: *const BYTE, size: ULONG, written: *mut ULONG) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    TRACE!("%p, %p, %lu, %p.\n", iface, buffer, size, written);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    position.QuadPart = (*stream).position as LONGLONG;
    hr = IStream_Seek(
        (*stream).stream,
        position,
        STREAM_SEEK_SET,
        core::ptr::null_mut(),
    );
    if hr >= 0 {
        hr = IStream_Write((*stream).stream, (buffer).cast(), size, written);
        if hr >= 0 {
            (*stream).position = (*stream).position.wrapping_add(*written as QWORD);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn bytestream_stream_Flush(iface: *mut IMFByteStream) -> HRESULT {
    let stream: *mut bytestream = impl_from_IMFByteStream(iface);

    return IStream_Commit((*stream).stream, STGC_DEFAULT);
}
unsafe extern "system" fn bytestream_stream_Close(iface: *mut IMFByteStream) -> HRESULT {
    // TRACE("%p.\n", iface);
    S_OK
}

unsafe extern "C" fn impl_from_IMFByteStream_IMFAttributes(iface: *mut IMFAttributes) -> *mut bytestream {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(bytestream, attributes.IMFAttributes_iface)) as *mut bytestream;
}

unsafe extern "system" fn bytestream_attributes_QueryInterface(iface: *mut IMFAttributes, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut stream: *mut bytestream = impl_from_IMFByteStream_IMFAttributes(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface), riid, out);
}

unsafe extern "system" fn bytestream_attributes_AddRef(iface: *mut IMFAttributes) -> ULONG {
    let stream: *mut bytestream = impl_from_IMFByteStream_IMFAttributes(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_attributes_Release(iface: *mut IMFAttributes) -> ULONG {
    let stream: *mut bytestream = impl_from_IMFByteStream_IMFAttributes(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}
unsafe extern "system" fn bytestream_read_callback_Invoke(iface: *mut IRtwqAsyncCallback, result: *mut IRtwqAsyncResult) -> HRESULT {
    let mut stream: *mut bytestream = impl_from_read_callback_IRtwqAsyncCallback(iface);
    let mut op: *mut async_stream_op;
    let mut object: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
    let mut hr: HRESULT;

    hr = IRtwqAsyncResult_GetObject(result, &mut object);
    if hr < 0 {
        return hr;
    }

    op = impl_async_stream_op_from_IUnknown(object);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    hr = IMFByteStream_Read(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface), (*op).u.dest, (*op).requested_length, core::ptr::addr_of_mut!((*op).actual_length));
    if hr < 0 {
        TRACE!("Read failed: %#lx\n", hr);
    }
    IMFAsyncResult_SetStatus((*op).caller, hr);
    list_add_tail(core::ptr::addr_of_mut!((*stream).pending), core::ptr::addr_of_mut!((*op).entry));

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    MFInvokeCallback((*op).caller);

    return S_OK;
}

unsafe extern "system" fn bytestream_write_callback_Invoke(iface: *mut IRtwqAsyncCallback, result: *mut IRtwqAsyncResult) -> HRESULT {
    let stream: *mut bytestream = impl_from_write_callback_IRtwqAsyncCallback(iface);
    let mut op: *mut async_stream_op;
    let mut object: *mut IUnknown = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = IRtwqAsyncResult_GetObject(result, core::ptr::addr_of_mut!(object));
    if hr < 0 {
        return hr;
    }

    op = impl_async_stream_op_from_IUnknown(object);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    hr = IMFByteStream_Write(
        core::ptr::addr_of_mut!((*stream).IMFByteStream_iface),
        (*op).u.src,
        (*op).requested_length,
        core::ptr::addr_of_mut!((*op).actual_length),
    );
    if hr < 0 {
        TRACE!("Write failed: %#lx\n", hr);
    }
    IMFAsyncResult_SetStatus((*op).caller, hr);
    list_add_tail(core::ptr::addr_of_mut!((*stream).pending), core::ptr::addr_of_mut!((*op).entry));

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    MFInvokeCallback((*op).caller);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMFByteStreamOnStream(stream: *mut IStream, bytestream: *mut *mut IMFByteStream) -> HRESULT {
    let mut object: *mut bytestream = core::ptr::null_mut();
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut stat: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    if (((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"MFCreateMFByteStreamOnStream\0".as_ptr() as *const i8,
            4492 as i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            stream,
            bytestream,
        );
    }

    object = calloc((1 as size_t) as _, (core::mem::size_of::<bytestream>() as size_t) as _) as *mut bytestream;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(core::ptr::addr_of_mut!((*object).attributes), 0);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    (*object).IMFByteStream_iface.lpVtbl = core::ptr::addr_of!(bytestream_stream_vtbl);
    (*object).attributes.IMFAttributes_iface.lpVtbl = core::ptr::addr_of!(bytestream_attributes_vtbl);
    (*object).read_callback.lpVtbl = core::ptr::addr_of!(bytestream_read_callback_vtbl);
    (*object).write_callback.lpVtbl = core::ptr::addr_of!(bytestream_write_callback_vtbl);
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));
    list_init(core::ptr::addr_of_mut!((*object).pending));

    (*object).stream = stream;
    IStream_AddRef((*object).stream);
    position.QuadPart = 0 as LONGLONG;
    IStream_Seek(
        (*object).stream,
        position,
        STREAM_SEEK_SET as DWORD,
        core::ptr::null_mut::<ULARGE_INTEGER>(),
    );

    if IStream_Stat((*object).stream, core::ptr::addr_of_mut!(stat), 0 as DWORD) >= 0 {
        if !stat.pwcsName.is_null() {
            IMFAttributes_SetString(
                core::ptr::addr_of_mut!((*object).attributes.IMFAttributes_iface),
                core::ptr::addr_of!(MF_BYTESTREAM_ORIGIN_NAME),
                stat.pwcsName as LPCWSTR,
            );
            CoTaskMemFree(stat.pwcsName as LPVOID);
        }
    }

    *bytestream = core::ptr::addr_of_mut!((*object).IMFByteStream_iface);

    return S_OK;
}

unsafe extern "system" fn bytestream_file_getservice_QueryInterface(iface: *mut IMFGetService, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let stream: *mut bytestream = impl_bytestream_from_IMFGetService(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_file_getservice_AddRef(iface: *mut IMFGetService) -> ULONG {
    let mut stream: *mut bytestream = impl_bytestream_from_IMFGetService(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_file_getservice_Release(iface: *mut IMFGetService) -> ULONG {
    let stream: *mut bytestream = impl_bytestream_from_IMFGetService(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*stream).IMFByteStream_iface));
}
unsafe extern "system" fn bytestream_file_getservice_GetService(
    iface: *mut IMFGetService,
    service: REFGUID,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%p, %s, %s, %p.\n", iface, debugstr_guid(service), debugstr_guid(riid), obj);
    E_NOTIMPL
}

unsafe extern "C" fn create_file_bytestream(
    mut accessmode: MF_FILE_ACCESSMODE,
    mut openmode: MF_FILE_OPENMODE,
    mut flags: MF_FILE_FLAGS,
    mut path: *const WCHAR,
    mut is_tempfile: BOOL,
    mut bytestream: *mut *mut IMFByteStream,
) -> HRESULT {
    let mut capabilities: DWORD = (MFBYTESTREAM_IS_SEEKABLE | MFBYTESTREAM_DOES_NOT_USE_NETWORK) as DWORD;
    let mut filecreation_disposition: DWORD = 0 as DWORD;
    let mut fileaccessmode: DWORD = 0 as DWORD;
    let mut fileattributes: DWORD = 0 as DWORD;
    let mut filesharemode: DWORD = (FILE_SHARE_READ | FILE_SHARE_DELETE) as DWORD;
    let mut object: *mut bytestream = core::ptr::null_mut::<bytestream>();
    let mut writetime: FILETIME = core::mem::zeroed::<FILETIME>();
    let mut file: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut hr: HRESULT = 0;

    match accessmode as core::ffi::c_uint {
        MF_ACCESSMODE_READ => {
            fileaccessmode = GENERIC_READ as DWORD;
            capabilities |= MFBYTESTREAM_IS_READABLE as DWORD;
        }
        MF_ACCESSMODE_WRITE => {
            fileaccessmode = GENERIC_WRITE as DWORD;
            capabilities |= MFBYTESTREAM_IS_WRITABLE as DWORD;
        }
        MF_ACCESSMODE_READWRITE => {
            fileaccessmode = (GENERIC_READ | GENERIC_WRITE) as DWORD;
            capabilities |= (MFBYTESTREAM_IS_READABLE | MFBYTESTREAM_IS_WRITABLE) as DWORD;
        }
        _ => {}
    }

    match (openmode as core::ffi::c_uint) as i32 {
        MF_OPENMODE_FAIL_IF_NOT_EXIST => {
            filecreation_disposition = OPEN_EXISTING as DWORD;
        }
        MF_OPENMODE_FAIL_IF_EXIST => {
            filecreation_disposition = CREATE_NEW as DWORD;
        }
        MF_OPENMODE_RESET_IF_EXIST => {
            filecreation_disposition = TRUNCATE_EXISTING as DWORD;
        }
        MF_OPENMODE_APPEND_IF_EXIST => {
            filecreation_disposition = OPEN_ALWAYS as DWORD;
            fileaccessmode |= FILE_APPEND_DATA as DWORD;
        }
        MF_OPENMODE_DELETE_IF_EXIST => {
            filecreation_disposition = CREATE_ALWAYS as DWORD;
        }
        _ => {}
    }

    if (flags as core::ffi::c_uint & MF_FILEFLAGS_NOBUFFERING as core::ffi::c_uint) != 0 {
        fileattributes |= FILE_FLAG_NO_BUFFERING as DWORD;
    }
    if is_tempfile != 0 {
        fileattributes |= FILE_FLAG_DELETE_ON_CLOSE as DWORD;
    }

    /* Open HANDLE to file */
    file = CreateFileW(
        path as LPCWSTR,
        fileaccessmode,
        filesharemode,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        filecreation_disposition,
        fileattributes,
        core::ptr::null_mut(),
    );
    if file == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    object = calloc(
        1,
        (core::mem::size_of::<bytestream>() as size_t) as _,
    ) as *mut bytestream;
    if object.is_null() {
        CloseHandle(file);
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(&raw mut (*object).attributes, 2);
    if hr < 0 {
        CloseHandle(file);
        free(object as *mut core::ffi::c_void);
        return hr;
    }
    (*object).IMFByteStream_iface.lpVtbl = &raw const bytestream_file_vtbl;
    (*object).attributes.IMFAttributes_iface.lpVtbl = &raw const bytestream_attributes_vtbl;
    (*object).IMFGetService_iface.lpVtbl = &raw const bytestream_file_getservice_vtbl;
    (*object).read_callback.lpVtbl = &raw const bytestream_read_callback_vtbl;
    (*object).write_callback.lpVtbl = &raw const bytestream_write_callback_vtbl;
    InitializeCriticalSection(&raw mut (*object).cs);
    list_init(&raw mut (*object).pending);
    (*object).capabilities = capabilities;
    (*object).hfile = file;

    if is_tempfile == 0
        && GetFileTime(
            file,
            core::ptr::null_mut::<_FILETIME>(),
            core::ptr::null_mut::<_FILETIME>(),
            &raw mut writetime,
        ) != 0
    {
        IMFAttributes_SetBlob(
            &raw mut (*object).attributes.IMFAttributes_iface,
            &raw const MF_BYTESTREAM_LAST_MODIFIED_TIME,
            &raw mut writetime as *const UINT8,
            core::mem::size_of::<FILETIME>() as UINT32,
        );
    }

    IMFAttributes_SetString(
        &raw mut (*object).attributes.IMFAttributes_iface,
        &raw const MF_BYTESTREAM_ORIGIN_NAME,
        path as LPCWSTR,
    );

    *bytestream = &raw mut (*object).IMFByteStream_iface;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateFile(
    accessmode: MF_FILE_ACCESSMODE,
    openmode: MF_FILE_OPENMODE,
    flags: MF_FILE_FLAGS,
    path: *const WCHAR,
    bytestream: *mut *mut IMFByteStream,
) -> HRESULT {
    return create_file_bytestream(accessmode, openmode, flags, path, (FALSE) as BOOL, bytestream);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateTempFile(
    mut accessmode: MF_FILE_ACCESSMODE,
    mut openmode: MF_FILE_OPENMODE,
    mut flags: MF_FILE_FLAGS,
    mut bytestream: *mut *mut IMFByteStream,
) -> HRESULT {
    let mut name: [WCHAR; 24] = [0; 24];
    let mut tmppath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut rnd: ULONG64 = 0;
    let mut len: size_t = 0;
    let mut hr: HRESULT = 0;

    BCryptGenRandom(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(rnd) as *mut UCHAR,
        core::mem::size_of::<ULONG64>() as ULONG,
        BCRYPT_USE_SYSTEM_PREFERRED_RNG,
    );
    swprintf(
        core::ptr::addr_of_mut!(name) as *mut WCHAR,
        (ARRAY_SIZE!(name)) as _,
        RUSTCONV_WSTR_3C86F98BD527.as_ptr(),
        rnd,
    );
    GetTempPathW(
        ARRAY_SIZE!(tmppath),
        core::ptr::addr_of_mut!(tmppath) as *mut WCHAR,
    );

    len = wcslen(core::ptr::addr_of_mut!(tmppath) as *mut WCHAR)
        + wcslen(core::ptr::addr_of_mut!(name) as *mut WCHAR)
        + 2 as size_t;
    path = malloc(len * core::mem::size_of::<WCHAR>() as size_t) as *mut WCHAR;
    if path.is_null() {
        return E_OUTOFMEMORY;
    }

    wcscpy(path, core::ptr::addr_of_mut!(tmppath) as *mut WCHAR);
    PathCchAppend(
        path as PWSTR,
        len,
        core::ptr::addr_of_mut!(name) as *mut WCHAR as PCWSTR,
    );

    hr = create_file_bytestream(
        accessmode,
        openmode,
        flags,
        path,
        (TRUE) as BOOL,
        bytestream,
    );

    free(path as *mut core::ffi::c_void);

    return hr;
}

unsafe extern "C" fn impl_wrapper_from_IMFByteStream(iface: *mut IMFByteStream) -> *mut bytestream_wrapper {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(bytestream_wrapper, IMFByteStream_iface)) as *mut bytestream_wrapper;
}

unsafe extern "C" fn impl_wrapper_from_IMFByteStreamCacheControl(iface: *mut IMFByteStreamCacheControl) -> *mut bytestream_wrapper {
    return (iface as *mut u8).sub(core::mem::offset_of!(bytestream_wrapper, IMFByteStreamCacheControl_iface)) as *mut bytestream_wrapper;
}

unsafe extern "C" fn impl_wrapper_from_IMFByteStreamBuffering(
    iface: *mut IMFByteStreamBuffering,
) -> *mut bytestream_wrapper {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(bytestream_wrapper, IMFByteStreamBuffering_iface),
    ) as *mut bytestream_wrapper;
}

unsafe extern "C" fn impl_wrapper_from_IMFMediaEventGenerator(
    iface: *mut IMFMediaEventGenerator,
) -> *mut bytestream_wrapper {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(bytestream_wrapper, IMFMediaEventGenerator_iface),
    ) as *mut bytestream_wrapper;
}

unsafe extern "C" fn impl_wrapper_from_IMFByteStreamTimeSeek(
    iface: *mut IMFByteStreamTimeSeek,
) -> *mut bytestream_wrapper {
    return CONTAINING_RECORD!(iface, bytestream_wrapper, IMFByteStreamTimeSeek_iface);
}

unsafe extern "C" fn impl_wrapper_from_IMFSampleOutputStream(iface: *mut IMFSampleOutputStream) -> *mut bytestream_wrapper {
    return (iface as *mut u8).sub(core::mem::offset_of!(bytestream_wrapper, IMFSampleOutputStream_iface)) as *mut bytestream_wrapper;
}

unsafe extern "C" fn impl_wrapper_from_IPropertyStore(iface: *mut IPropertyStore) -> *mut bytestream_wrapper {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(bytestream_wrapper, IPropertyStore_iface)) as *mut bytestream_wrapper;
}

unsafe extern "C" fn impl_wrapper_from_IMFAttributes(iface: *mut IMFAttributes) -> *mut bytestream_wrapper {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(bytestream_wrapper, IMFAttributes_iface)) as *mut bytestream_wrapper;
}

unsafe extern "system" fn bytestream_wrapper_QueryInterface(iface: *mut IMFByteStream, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFByteStream)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface) as *mut core::ffi::c_void;
    } else if !(*wrapper).cache_control.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFByteStreamCacheControl)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFByteStreamCacheControl_iface) as *mut core::ffi::c_void;
    } else if !(*wrapper).stream_buffering.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFByteStreamBuffering)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFByteStreamBuffering_iface) as *mut core::ffi::c_void;
    } else if !(*wrapper).event_generator.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaEventGenerator)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFMediaEventGenerator_iface) as *mut core::ffi::c_void;
    } else if !(*wrapper).time_seek.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFByteStreamTimeSeek)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFByteStreamTimeSeek_iface) as *mut core::ffi::c_void;
    } else if !(*wrapper).sample_output.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSampleOutputStream)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFSampleOutputStream_iface) as *mut core::ffi::c_void;
    } else if !(*wrapper).propstore.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IPropertyStore)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IPropertyStore_iface) as *mut core::ffi::c_void;
    } else if !(*wrapper).attributes.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFAttributes_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn bytestream_wrapper_AddRef(iface: *mut IMFByteStream) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*wrapper).refcount) as *mut i32) as ULONG;

    TRACE!("%p, refcount %ld.\n", iface, refcount);

    return refcount;
}

unsafe extern "system" fn bytestream_wrapper_Release(iface: *mut IMFByteStream) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*wrapper).refcount) as *mut LONG) as ULONG;

    if refcount == 0 {
        if !(*wrapper).cache_control.is_null() {
            IMFByteStreamCacheControl_Release((*wrapper).cache_control);
        }
        if !(*wrapper).stream_buffering.is_null() {
            IMFByteStreamBuffering_Release((*wrapper).stream_buffering);
        }
        if !(*wrapper).event_generator.is_null() {
            IMFMediaEventGenerator_Release((*wrapper).event_generator);
        }
        if !(*wrapper).time_seek.is_null() {
            IMFByteStreamTimeSeek_Release((*wrapper).time_seek);
        }
        if !(*wrapper).sample_output.is_null() {
            IMFSampleOutputStream_Release((*wrapper).sample_output);
        }
        if !(*wrapper).propstore.is_null() {
            IPropertyStore_Release((*wrapper).propstore);
        }
        if !(*wrapper).attributes.is_null() {
            IMFAttributes_Release((*wrapper).attributes);
        }
        IMFByteStream_Release((*wrapper).stream);
        free(wrapper as *mut core::ffi::c_void);
    }

    return refcount;
}
unsafe extern "system" fn bytestream_wrapper_GetCapabilities(
    mut iface: *mut IMFByteStream,
    mut capabilities: *mut DWORD,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_GetCapabilities\0" as *const u8 as *const ::core::ffi::c_char,
            4855 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            capabilities,
        );
    }
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_GetCapabilities((*wrapper).stream, capabilities)
    };
}

unsafe extern "system" fn bytestream_wrapper_GetLength(iface: *mut IMFByteStream, length: *mut QWORD) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);

    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_GetLength((*wrapper).stream, length)
    };
}

unsafe extern "system" fn bytestream_wrapper_SetLength(iface: *mut IMFByteStream, length: QWORD) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);

    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_SetLength((*wrapper).stream, length)
    };
}

unsafe extern "system" fn bytestream_wrapper_GetCurrentPosition(iface: *mut IMFByteStream, position: *mut QWORD) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_GetCurrentPosition((*wrapper).stream, position)
    };
}
unsafe extern "system" fn bytestream_wrapper_SetCurrentPosition(iface: *mut IMFByteStream, position: QWORD) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"bytestream_wrapper_SetCurrentPosition\0" as *const u8 as *const i8,
            4895 as i32,
            b"%p, %s.\n\0" as *const u8 as *const i8,
            iface,
            wine_dbgstr_longlong(position as ULONGLONG),
        );
    }
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_SetCurrentPosition((*wrapper).stream, position)
    };
}

unsafe extern "system" fn bytestream_wrapper_IsEndOfStream(iface: *mut IMFByteStream, eos: *mut BOOL) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_IsEndOfStream\0" as *const u8 as *const ::core::ffi::c_char,
            4905 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            eos,
        );
    }
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_IsEndOfStream((*wrapper).stream, eos)
    };
}
unsafe extern "system" fn bytestream_wrapper_Read(iface: *mut IMFByteStream, data: *mut BYTE, count: ULONG, byte_read: *mut ULONG) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);

    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_Read((*wrapper).stream, data, count, byte_read)
    };
}

unsafe extern "system" fn bytestream_wrapper_BeginRead(iface: *mut IMFByteStream, data: *mut BYTE, size: ULONG, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    return if (*wrapper).is_closed != 0 { MF_E_INVALIDREQUEST } else { IMFByteStream_BeginRead((*wrapper).stream, data, size, callback, state) };
}

unsafe extern "system" fn bytestream_wrapper_EndRead(iface: *mut IMFByteStream, result: *mut IMFAsyncResult, byte_read: *mut ULONG) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    TRACE!("%p, %p, %p.\n", iface, result, byte_read);
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_EndRead((*wrapper).stream, result, byte_read)
    };
}

unsafe extern "system" fn bytestream_wrapper_Write(iface: *mut IMFByteStream, data: *const BYTE, count: ULONG, written: *mut ULONG) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);

    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_Write((*wrapper).stream, data, count, written)
    };
}

unsafe extern "system" fn bytestream_wrapper_BeginWrite(iface: *mut IMFByteStream, data: *const BYTE, size: ULONG, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    return if (*wrapper).is_closed != 0 { MF_E_INVALIDREQUEST } else { IMFByteStream_BeginWrite((*wrapper).stream, data, size, callback, state) };
}

unsafe extern "system" fn bytestream_wrapper_EndWrite(iface: *mut IMFByteStream, result: *mut IMFAsyncResult, written: *mut ULONG) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    TRACE!("%p, %p, %p.\n", iface, result, written);
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_EndWrite((*wrapper).stream, result, written)
    };
}

unsafe extern "system" fn bytestream_wrapper_Seek(
    iface: *mut IMFByteStream,
    seek: MFBYTESTREAM_SEEK_ORIGIN,
    offset: LONGLONG,
    flags: DWORD,
    current: *mut QWORD,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_Seek((*wrapper).stream, seek, offset, flags, current)
    };
}
unsafe extern "system" fn bytestream_wrapper_Flush(mut iface: *mut IMFByteStream) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_Flush\0" as *const u8 as *const ::core::ffi::c_char,
            4988 as ::core::ffi::c_int,
            b"%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
        );
    }
    return if (*wrapper).is_closed != 0 {
        MF_E_INVALIDREQUEST
    } else {
        IMFByteStream_Flush((*wrapper).stream)
    };
}

unsafe extern "system" fn bytestream_wrapper_Close(iface: *mut IMFByteStream) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStream(iface);

    (*wrapper).is_closed = (TRUE as i32);

    return S_OK;
}

unsafe extern "system" fn bytestream_wrapper_cache_control_QueryInterface(iface: *mut IMFByteStreamCacheControl, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamCacheControl(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_wrapper_cache_control_AddRef(iface: *mut IMFByteStreamCacheControl) -> ULONG {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamCacheControl(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_cache_control_Release(iface: *mut IMFByteStreamCacheControl) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamCacheControl(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_cache_control_StopBackgroundTransfer(iface: *mut IMFByteStreamCacheControl) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamCacheControl(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_cache_control_StopBackgroundTransfer\0".as_ptr() as *const i8,
            5049 as i32,
            b"%p.\n\0".as_ptr() as *const i8,
            iface,
        );
    }
    return IMFByteStreamCacheControl_StopBackgroundTransfer((*wrapper).cache_control);
}

unsafe extern "system" fn bytestream_wrapper_buffering_QueryInterface(
    iface: *mut IMFByteStreamBuffering,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamBuffering(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_wrapper_buffering_AddRef(iface: *mut IMFByteStreamBuffering) -> ULONG {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamBuffering(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_buffering_Release(iface: *mut IMFByteStreamBuffering) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamBuffering(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}
unsafe extern "system" fn bytestream_wrapper_buffering_SetBufferingParams(mut iface: *mut IMFByteStreamBuffering, mut params: *mut MFBYTESTREAM_BUFFERING_PARAMS) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamBuffering(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_buffering_SetBufferingParams\0" as *const u8
                as *const ::core::ffi::c_char,
            5086 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            params,
        );
    }
    return IMFByteStreamBuffering_SetBufferingParams((*wrapper).stream_buffering, params);
}

unsafe extern "system" fn bytestream_wrapper_buffering_EnableBuffering(
    iface: *mut IMFByteStreamBuffering,
    enable: BOOL,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamBuffering(iface);

    return IMFByteStreamBuffering_EnableBuffering((*wrapper).stream_buffering, enable);
}

unsafe extern "system" fn bytestream_wrapper_buffering_StopBuffering(iface: *mut IMFByteStreamBuffering) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamBuffering(iface);

    return IMFByteStreamBuffering_StopBuffering((*wrapper).stream_buffering);
}

unsafe extern "system" fn bytestream_wrapper_timeseek_QueryInterface(iface: *mut IMFByteStreamTimeSeek, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamTimeSeek(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_wrapper_timeseek_AddRef(iface: *mut IMFByteStreamTimeSeek) -> ULONG {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamTimeSeek(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_timeseek_Release(iface: *mut IMFByteStreamTimeSeek) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamTimeSeek(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}
unsafe extern "system" fn bytestream_wrapper_timeseek_IsTimeSeekSupported(
    iface: *mut IMFByteStreamTimeSeek,
    result: *mut BOOL,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamTimeSeek(iface);

    if (((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"bytestream_wrapper_timeseek_IsTimeSeekSupported\0" as *const u8 as *const i8,
            5143i32,
            b"%p, %p.\n\0" as *const u8 as *const i8,
            iface,
            result,
        );
    }

    return IMFByteStreamTimeSeek_IsTimeSeekSupported((*wrapper).time_seek, result);
}

unsafe extern "system" fn bytestream_wrapper_timeseek_TimeSeek(iface: *mut IMFByteStreamTimeSeek, position: QWORD) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamTimeSeek(iface);

    return IMFByteStreamTimeSeek_TimeSeek((*wrapper).time_seek, position);
}

unsafe extern "system" fn bytestream_wrapper_timeseek_GetTimeSeekResult(
    iface: *mut IMFByteStreamTimeSeek,
    start_time: *mut QWORD,
    stop_time: *mut QWORD,
    duration: *mut QWORD,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFByteStreamTimeSeek(iface);

    return IMFByteStreamTimeSeek_GetTimeSeekResult((*wrapper).time_seek, start_time, stop_time, duration);
}

unsafe extern "system" fn bytestream_wrapper_events_QueryInterface(iface: *mut IMFMediaEventGenerator, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFMediaEventGenerator(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_wrapper_events_AddRef(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFMediaEventGenerator(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_events_Release(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFMediaEventGenerator(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_events_GetEvent(iface: *mut IMFMediaEventGenerator, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFMediaEventGenerator(iface);
    TRACE!("%p, %#lx, %p.\n", iface, flags, event);
    return IMFMediaEventGenerator_GetEvent((*wrapper).event_generator, flags, event);
}

unsafe extern "system" fn bytestream_wrapper_events_BeginGetEvent(
    iface: *mut IMFMediaEventGenerator,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFMediaEventGenerator(iface);
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                .as_ptr() as *const i8,
            b"bytestream_wrapper_events_BeginGetEvent\0".as_ptr() as *const i8,
            5208 as i32,
            b"%p, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            callback,
            state,
        );
    }
    return IMFMediaEventGenerator_BeginGetEvent((*wrapper).event_generator, callback, state);
}
unsafe extern "system" fn bytestream_wrapper_events_EndGetEvent(iface: *mut IMFMediaEventGenerator, result: *mut IMFAsyncResult, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventGenerator_EndGetEvent((*wrapper).event_generator, result, event);
}
unsafe extern "system" fn bytestream_wrapper_events_QueueEvent(iface: *mut IMFMediaEventGenerator, type_: MediaEventType, ext_type: REFGUID, hr: HRESULT, value: *const PROPVARIANT) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventGenerator_QueueEvent((*wrapper).event_generator, type_, ext_type, hr, value);
}

unsafe extern "system" fn bytestream_wrapper_sample_output_QueryInterface(iface: *mut IMFSampleOutputStream, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFSampleOutputStream(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_wrapper_sample_output_AddRef(iface: *mut IMFSampleOutputStream) -> ULONG {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFSampleOutputStream(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_sample_output_Release(iface: *mut IMFSampleOutputStream) -> ULONG {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFSampleOutputStream(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_sample_output_BeginWriteSample(
    iface: *mut IMFSampleOutputStream,
    sample: *mut IMFSample,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFSampleOutputStream(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_sample_output_BeginWriteSample\0" as *const u8
                as *const ::core::ffi::c_char,
            5266 as ::core::ffi::c_int,
            b"%p, %p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            sample,
            callback,
            state,
        );
    }
    return IMFSampleOutputStream_BeginWriteSample(
        (*wrapper).sample_output,
        sample,
        callback,
        state,
    );
}

unsafe extern "system" fn bytestream_wrapper_sample_output_EndWriteSample(
    iface: *mut IMFSampleOutputStream,
    result: *mut IMFAsyncResult,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFSampleOutputStream(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8
                as *const i8,
            b"bytestream_wrapper_sample_output_EndWriteSample\0" as *const u8 as *const i8,
            5275 as i32,
            b"%p, %p.\n\0" as *const u8 as *const i8,
            iface,
            result,
        );
    }
    return IMFSampleOutputStream_EndWriteSample((*wrapper).sample_output, result);
}

unsafe extern "system" fn bytestream_wrapper_sample_output_Close(iface: *mut IMFSampleOutputStream) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFSampleOutputStream(iface);

    TRACE!("%p.\n", iface);

    return IMFSampleOutputStream_Close((*wrapper).sample_output);
}

unsafe extern "system" fn bytestream_wrapper_propstore_QueryInterface(iface: *mut IPropertyStore, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_wrapper_propstore_AddRef(iface: *mut IPropertyStore) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_propstore_Release(iface: *mut IPropertyStore) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_propstore_GetCount(
    iface: *mut IPropertyStore,
    count: *mut DWORD,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);

    return IPropertyStore_GetCount((*wrapper).propstore, count);
}

unsafe extern "system" fn bytestream_wrapper_propstore_GetAt(iface: *mut IPropertyStore, prop: DWORD, key: *mut PROPERTYKEY) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);

    return IPropertyStore_GetAt((*wrapper).propstore, prop, key);
}

unsafe extern "system" fn bytestream_wrapper_propstore_GetValue(iface: *mut IPropertyStore, key: *const PROPERTYKEY, value: *mut PROPVARIANT) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << 3)) != 0 {
        // 3 == __WINE_DBCL_TRACE
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = 3; // __WINE_DBCL_TRACE
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_propstore_GetValue\0".as_ptr() as *const i8,
            5339i32,
            b"%p, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            key,
            value,
        );
    }
    return IPropertyStore_GetValue((*wrapper).propstore, key, value);
}

unsafe extern "system" fn bytestream_wrapper_propstore_SetValue(
    iface: *mut IPropertyStore,
    key: REFPROPERTYKEY,
    value: *const PROPVARIANT,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"bytestream_wrapper_propstore_SetValue\0" as *const u8 as *const i8,
            5349 as i32,
            b"%p, %p, %s.\n\0" as *const u8 as *const i8,
            iface,
            key,
            debugstr_propvar(value),
        );
    }
    return IPropertyStore_SetValue((*wrapper).propstore, key, value);
}

unsafe extern "system" fn bytestream_wrapper_propstore_Commit(iface: *mut IPropertyStore) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IPropertyStore(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"bytestream_wrapper_propstore_Commit\0" as *const u8 as *const i8,
            5358 as i32,
            b"%p.\n\0" as *const u8 as *const i8,
            iface,
        );
    }
    return IPropertyStore_Commit((*wrapper).propstore);
}

unsafe extern "system" fn bytestream_wrapper_attributes_QueryInterface(iface: *mut IMFAttributes, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    return IMFByteStream_QueryInterface(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface), riid, obj);
}

unsafe extern "system" fn bytestream_wrapper_attributes_AddRef(iface: *mut IMFAttributes) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    return IMFByteStream_AddRef(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_attributes_Release(iface: *mut IMFAttributes) -> ULONG {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    return IMFByteStream_Release(core::ptr::addr_of_mut!((*wrapper).IMFByteStream_iface));
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetItem(iface: *mut IMFAttributes, key: REFGUID, value: *mut PROPVARIANT) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);

    return IMFAttributes_GetItem((*wrapper).attributes, key, value);
}
unsafe extern "system" fn bytestream_wrapper_attributes_GetItemType(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut type_0: *mut MF_ATTRIBUTE_TYPE,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_GetItemType\0" as *const u8
                as *const ::core::ffi::c_char,
            5406 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            type_0,
        );
    }
    return IMFAttributes_GetItemType((*wrapper).attributes, key, type_0);
}

unsafe extern "system" fn bytestream_wrapper_attributes_CompareItem(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *const PROPVARIANT,
    result: *mut BOOL,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"bytestream_wrapper_attributes_CompareItem\0" as *const u8 as *const i8,
            5416 as i32,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return IMFAttributes_CompareItem((*wrapper).attributes, key, value, result);
}

unsafe extern "system" fn bytestream_wrapper_attributes_Compare(iface: *mut IMFAttributes, theirs: *mut IMFAttributes, match_type: MF_ATTRIBUTES_MATCH_TYPE, ret: *mut BOOL) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);

    return IMFAttributes_Compare((*wrapper).attributes, theirs, match_type, ret);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetUINT32(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *mut UINT32,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const i8,
            b"bytestream_wrapper_attributes_GetUINT32\0".as_ptr() as *const i8,
            5435i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return IMFAttributes_GetUINT32((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetUINT64(iface: *mut IMFAttributes, key: REFGUID, value: *mut UINT64) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_GetUINT64\0" as *const u8 as *const ::core::ffi::c_char,
            5444 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return IMFAttributes_GetUINT64((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetDouble(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *mut f64,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*(*core::ptr::addr_of!(__wine_dbch___default))).flags as i32
        & (1i32 << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = core::ptr::addr_of!(__wine_dbch___default) as *mut __wine_debug_channel;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_attributes_GetDouble\0".as_ptr() as *const i8,
            5453i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return IMFAttributes_GetDouble((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetGUID(iface: *mut IMFAttributes, key: REFGUID, value: *mut GUID) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);

    return IMFAttributes_GetGUID((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetStringLength(iface: *mut IMFAttributes, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), length);
    return IMFAttributes_GetStringLength((*wrapper).attributes, key, length);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetString(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *mut WCHAR,
    size: UINT32,
    length: *mut UINT32,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_TRACE))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: i32 = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_attributes_GetString\0".as_ptr() as *const i8,
            5481,
            b"%p, %s, %p, %d, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
            size,
            length,
        );
    }
    return IMFAttributes_GetString((*wrapper).attributes, key, value as LPWSTR, size, length);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetAllocatedString(iface: *mut IMFAttributes, key: REFGUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_GetAllocatedString\0" as *const u8 as *const ::core::ffi::c_char,
            5490 as ::core::ffi::c_int,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
            length,
        );
    }
    return IMFAttributes_GetAllocatedString((*wrapper).attributes, key, value as *mut LPWSTR, length);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetBlobSize(
    iface: *mut IMFAttributes,
    key: *const GUID,
    size: *mut UINT32,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const i8,
            b"bytestream_wrapper_attributes_GetBlobSize\0".as_ptr() as *const i8,
            5499i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            size,
        );
    }
    return IMFAttributes_GetBlobSize((*wrapper).attributes, key, size);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetBlob(
    iface: *mut IMFAttributes,
    key: *const GUID,
    buf: *mut UINT8,
    bufsize: UINT32,
    blobsize: *mut UINT32,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    TRACE!(
        "%p, %s, %p, %d, %p.\n",
        iface,
        debugstr_attr(key),
        buf,
        bufsize,
        blobsize
    );
    return IMFAttributes_GetBlob((*wrapper).attributes, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetAllocatedBlob(
    iface: *mut IMFAttributes,
    key: *const GUID,
    buf: *mut *mut UINT8,
    size: *mut UINT32,
) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8
                as *const i8,
            b"bytestream_wrapper_attributes_GetAllocatedBlob\0" as *const u8 as *const i8,
            5518 as i32,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            buf,
            size,
        );
    }
    return IMFAttributes_GetAllocatedBlob((*wrapper).attributes, key, buf, size);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetUnknown(
    iface: *mut IMFAttributes,
    key: *const GUID,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"bytestream_wrapper_attributes_GetUnknown\0" as *const u8
                as *const core::ffi::c_char,
            5527 as core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_guid(riid as *const GUID),
            obj,
        );
    }
    return IMFAttributes_GetUnknown((*wrapper).attributes, key, riid, obj as *mut LPVOID);
}

unsafe extern "system" fn bytestream_wrapper_attributes_SetItem(iface: *mut IMFAttributes, key: REFGUID, value: REFPROPVARIANT) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
            5536 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
        );
    }
    return IMFAttributes_SetItem((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_DeleteItem(iface: *mut IMFAttributes, key: REFGUID) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);

    return IMFAttributes_DeleteItem((*wrapper).attributes, key);
}
unsafe extern "system" fn bytestream_wrapper_attributes_DeleteAllItems(iface: *mut IMFAttributes) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_DeleteAllItems\0" as *const u8
                as *const ::core::ffi::c_char,
            5554 as ::core::ffi::c_int,
            b"%p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
        );
    }
    return IMFAttributes_DeleteAllItems((*wrapper).attributes);
}

unsafe extern "system" fn bytestream_wrapper_attributes_SetUINT32(iface: *mut IMFAttributes, key: *const GUID, value: UINT32) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_attributes_SetUINT32\0".as_ptr() as *const i8,
            5563 as i32,
            b"%p, %s, %d.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return IMFAttributes_SetUINT32((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_SetUINT64(iface: *mut IMFAttributes, key: REFGUID, value: UINT64) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_attributes_SetUINT64\0".as_ptr() as *const i8,
            5572i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return IMFAttributes_SetUINT64((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_SetDouble(iface: *mut IMFAttributes, key: *const GUID, value: f64) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"bytestream_wrapper_attributes_SetDouble\0" as *const u8 as *const i8,
            5581 as i32,
            b"%p, %s, %f.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return IMFAttributes_SetDouble((*wrapper).attributes, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_SetGUID(iface: *mut IMFAttributes, key: *const GUID, value: *const GUID) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_attributes_SetGUID\0".as_ptr() as *const i8,
            5590 as i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_mf_guid(value),
        );
    }
    return IMFAttributes_SetGUID((*wrapper).attributes, key, value);
}
unsafe extern "system" fn bytestream_wrapper_attributes_SetString(iface: *mut IMFAttributes, key: REFGUID, value: *const WCHAR) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"bytestream_wrapper_attributes_SetString\0".as_ptr() as *const i8,
            5599 as i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_w(value),
        );
    }
    return IMFAttributes_SetString((*wrapper).attributes, key, value as LPCWSTR);
}

unsafe extern "system" fn bytestream_wrapper_attributes_SetBlob(iface: *mut IMFAttributes, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_SetBlob\0" as *const u8 as *const ::core::ffi::c_char,
            5608 as ::core::ffi::c_int,
            b"%p, %s, %p, %u.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            buf,
            size,
        );
    }
    return IMFAttributes_SetBlob((*wrapper).attributes, key, buf, size);
}

unsafe extern "system" fn bytestream_wrapper_attributes_SetUnknown(iface: *mut IMFAttributes, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_SetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            5617 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            unknown,
        );
    }
    return IMFAttributes_SetUnknown((*wrapper).attributes, key, unknown);
}

unsafe extern "system" fn bytestream_wrapper_attributes_LockStore(mut iface: *mut IMFAttributes) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"bytestream_wrapper_attributes_LockStore\0" as *const u8 as *const i8,
            5626 as i32,
            b"%p.\n\0" as *const u8 as *const i8,
            iface,
        );
    }
    return IMFAttributes_LockStore((*wrapper).attributes);
}

unsafe extern "system" fn bytestream_wrapper_attributes_UnlockStore(iface: *mut IMFAttributes) -> HRESULT {
    let wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);

    return IMFAttributes_UnlockStore((*wrapper).attributes);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetCount(iface: *mut IMFAttributes, count: *mut UINT32) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);

    return IMFAttributes_GetCount((*wrapper).attributes, count);
}

unsafe extern "system" fn bytestream_wrapper_attributes_GetItemByIndex(iface: *mut IMFAttributes, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"bytestream_wrapper_attributes_GetItemByIndex\0" as *const u8 as *const ::core::ffi::c_char,
            5653 as ::core::ffi::c_int,
            b"%p, %u, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            key,
            value,
        );
    }
    return IMFAttributes_GetItemByIndex((*wrapper).attributes, index, key, value);
}

unsafe extern "system" fn bytestream_wrapper_attributes_CopyAllItems(
    mut iface: *mut IMFAttributes,
    mut dest: *mut IMFAttributes,
) -> HRESULT {
    let mut wrapper: *mut bytestream_wrapper = impl_wrapper_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const i8,
            b"bytestream_wrapper_attributes_CopyAllItems\0".as_ptr() as *const i8,
            5662 as i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface,
            dest,
        );
    }
    return IMFAttributes_CopyAllItems((*wrapper).attributes, dest);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMFByteStreamWrapper(stream: *mut IMFByteStream, wrapper: *mut *mut IMFByteStream) -> HRESULT {
    let mut object: *mut bytestream_wrapper = core::ptr::null_mut::<bytestream_wrapper>();
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"MFCreateMFByteStreamWrapper\0" as *const u8 as *const i8,
            5711i32,
            b"%p, %p.\n\0" as *const u8 as *const i8,
            stream,
            wrapper,
        );
    }
    object = calloc(1, (core::mem::size_of::<bytestream_wrapper>() as usize) as _) as *mut bytestream_wrapper;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFByteStreamCacheControl_iface.lpVtbl = &raw const bytestream_wrapper_cache_control_vtbl;
    (*object).IMFByteStreamBuffering_iface.lpVtbl = &raw const bytestream_wrapper_buffering_vtbl;
    (*object).IMFMediaEventGenerator_iface.lpVtbl = &raw const bytestream_wrapper_events_vtbl;
    (*object).IMFByteStreamTimeSeek_iface.lpVtbl = &raw const bytestream_wrapper_timeseek_vtbl;
    (*object).IMFSampleOutputStream_iface.lpVtbl = &raw const bytestream_wrapper_sample_output_vtbl;
    (*object).IMFByteStream_iface.lpVtbl = &raw const bytestream_wrapper_vtbl;
    (*object).IPropertyStore_iface.lpVtbl = &raw const bytestream_wrapper_propstore_vtbl;
    (*object).IMFAttributes_iface.lpVtbl = &raw const bytestream_wrapper_attributes_vtbl;

    IMFByteStream_QueryInterface(
        stream,
        &raw const IID_IMFByteStreamCacheControl,
        &raw mut (*object).cache_control as *mut *mut core::ffi::c_void,
    );
    IMFByteStream_QueryInterface(
        stream,
        &raw const IID_IMFByteStreamBuffering,
        &raw mut (*object).stream_buffering as *mut *mut core::ffi::c_void,
    );
    IMFByteStream_QueryInterface(
        stream,
        &raw const IID_IMFMediaEventGenerator,
        &raw mut (*object).event_generator as *mut *mut core::ffi::c_void,
    );
    IMFByteStream_QueryInterface(
        stream,
        &raw const IID_IMFByteStreamTimeSeek,
        &raw mut (*object).time_seek as *mut *mut core::ffi::c_void,
    );
    IMFByteStream_QueryInterface(
        stream,
        &raw const IID_IMFSampleOutputStream,
        &raw mut (*object).sample_output as *mut *mut core::ffi::c_void,
    );
    IMFByteStream_QueryInterface(
        stream,
        &raw const IID_IPropertyStore,
        &raw mut (*object).propstore as *mut *mut core::ffi::c_void,
    );
    IMFByteStream_QueryInterface(
        stream,
        &raw const IID_IMFAttributes,
        &raw mut (*object).attributes as *mut *mut core::ffi::c_void,
    );

    (*object).stream = stream;
    IMFByteStream_AddRef((*object).stream);

    (*object).refcount = 1;

    *wrapper = &raw mut (*object).IMFByteStream_iface;

    return S_OK;
}

unsafe extern "system" fn MFPluginControl_QueryInterface(
    iface: *mut IMFPluginControl,
    riid: REFIID,
    ppv: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    if IsEqualGUID!(riid, &raw const IID_IUnknown) != false {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                (__dbcl) as u32,
                __dbch,
                b"dll/win32/mfplat/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MFPluginControl_QueryInterface\0" as *const u8 as *const ::core::ffi::c_char,
                5746 as ::core::ffi::c_int,
                b"(IID_IUnknown %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
                ppv,
            );
        }
        *ppv = iface as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(riid, &raw const IID_IMFPluginControl) != false {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                (__dbcl_0) as u32,
                __dbch_0,
                b"dll/win32/mfplat/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MFPluginControl_QueryInterface\0" as *const u8 as *const ::core::ffi::c_char,
                5749 as ::core::ffi::c_int,
                b"(IID_IMFPluginControl %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
                ppv,
            );
        }
        *ppv = iface as *mut ::core::ffi::c_void;
    } else {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_FIXME as ::core::ffi::c_int)))
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl_1) as u32,
                __dbch_1,
                b"dll/win32/mfplat/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MFPluginControl_QueryInterface\0" as *const u8 as *const ::core::ffi::c_char,
                5752 as ::core::ffi::c_int,
                b"(%s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_guid(riid as *const _GUID),
                ppv,
            );
        }
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppv as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn MFPluginControl_AddRef(iface: *mut IMFPluginControl) -> ULONG {
    // TRACE("\n");
    2
}
unsafe extern "system" fn MFPluginControl_Release(iface: *mut IMFPluginControl) -> ULONG {
    // TRACE("\n");
    1
}
unsafe extern "system" fn MFPluginControl_GetPreferredClsid(
    iface: *mut IMFPluginControl,
    plugin_type: DWORD,
    selector: *const WCHAR,
    clsid: *mut CLSID,
) -> HRESULT {
    // FIXME("(%ld %s %p)\n", plugin_type, debugstr_w(selector), clsid);
    E_NOTIMPL
}
unsafe extern "system" fn MFPluginControl_GetPreferredClsidByIndex(
    iface: *mut IMFPluginControl,
    plugin_type: DWORD,
    index: DWORD,
    selector: *mut *mut WCHAR,
    clsid: *mut CLSID,
) -> HRESULT {
    // FIXME("(%ld %ld %p %p)\n", plugin_type, index, selector, clsid);
    E_NOTIMPL
}
unsafe extern "system" fn MFPluginControl_SetPreferredClsid(
    iface: *mut IMFPluginControl,
    plugin_type: DWORD,
    selector: *const WCHAR,
    clsid: *const CLSID,
) -> HRESULT {
    // FIXME("(%ld %s %s)\n", plugin_type, debugstr_w(selector), debugstr_guid(clsid));
    E_NOTIMPL
}
unsafe extern "system" fn MFPluginControl_IsDisabled(
    iface: *mut IMFPluginControl,
    plugin_type: DWORD,
    clsid: REFCLSID,
) -> HRESULT {
    // FIXME("(%ld %s)\n", plugin_type, debugstr_guid(clsid));
    E_NOTIMPL
}
unsafe extern "system" fn MFPluginControl_GetDisabledByIndex(
    iface: *mut IMFPluginControl,
    plugin_type: DWORD,
    index: DWORD,
    clsid: *mut CLSID,
) -> HRESULT {
    // FIXME("(%ld %ld %p)\n", plugin_type, index, clsid);
    E_NOTIMPL
}
unsafe extern "system" fn MFPluginControl_SetDisabled(
    iface: *mut IMFPluginControl,
    plugin_type: DWORD,
    clsid: REFCLSID,
    disabled: BOOL,
) -> HRESULT {
    // FIXME("(%ld %s %x)\n", plugin_type, debugstr_guid(clsid), disabled);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn MFGetPluginControl(ret: *mut *mut IMFPluginControl) -> HRESULT {
    *ret = core::ptr::addr_of_mut!(plugin_control) as *mut IMFPluginControl;
    return S_OK;
}

unsafe extern "C" fn impl_from_IMFSourceResolver(iface: *mut IMFSourceResolver) -> *mut source_resolver {
    return CONTAINING_RECORD!(iface, source_resolver, IMFSourceResolver_iface);
}

unsafe extern "C" fn impl_from_stream_IRtwqAsyncCallback(
    iface: *mut IRtwqAsyncCallback,
) -> *mut source_resolver {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(source_resolver, stream_callback),
    ) as *mut source_resolver;
}

unsafe extern "C" fn impl_from_url_IRtwqAsyncCallback(
    iface: *mut IRtwqAsyncCallback,
) -> *mut source_resolver {
    return (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(source_resolver, url_callback) as ULONG_PTR,
    ) as *mut source_resolver;
}

/// resolver_handler_end_create's local anonymous union.
#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed_73 {
    pub handler: *mut IUnknown,
    pub stream_handler: *mut IMFByteStreamHandler,
    pub scheme_handler: *mut IMFSchemeHandler,
}

unsafe extern "C" fn resolver_handler_end_create(
    mut resolver: *mut source_resolver,
    mut origin: resolved_object_origin,
    mut result: *mut IRtwqAsyncResult,
) -> HRESULT {
    let mut inner_result: *mut IRtwqAsyncResult =
        IRtwqAsyncResult_GetStateNoAddRef(result) as *mut IRtwqAsyncResult;
    let mut data: *mut RTWQASYNCRESULT = inner_result as *mut RTWQASYNCRESULT;
    let mut queued_result: *mut resolver_queued_result =
        ::core::ptr::null_mut::<resolver_queued_result>();
    let mut handler: C2RustUnnamed_73 = C2RustUnnamed_73 {
        handler: ::core::ptr::null_mut::<IUnknown>(),
    };
    queued_result = calloc(
        1,
        (::core::mem::size_of::<resolver_queued_result>() as size_t) as _,
    ) as *mut resolver_queued_result;
    if queued_result.is_null() {
        return E_OUTOFMEMORY;
    }
    (*queued_result).origin = (origin) as i32;
    IRtwqAsyncResult_GetObject(inner_result, &raw mut handler.handler);
    match (origin as ::core::ffi::c_uint) as i32 {
        OBJECT_FROM_BYTESTREAM => {
            (*queued_result).hr = IMFByteStreamHandler_EndCreateObject(
                handler.stream_handler,
                result as *mut IMFAsyncResult,
                &raw mut (*queued_result).obj_type,
                &raw mut (*queued_result).object,
            );
        }
        OBJECT_FROM_URL => {
            (*queued_result).hr = IMFSchemeHandler_EndCreateObject(
                handler.scheme_handler,
                result as *mut IMFAsyncResult,
                &raw mut (*queued_result).obj_type,
                &raw mut (*queued_result).object,
            );
        }
        _ => {
            (*queued_result).hr = E_FAIL;
        }
    }
    IUnknown_Release(handler.handler);
    if !(*data).hEvent.is_null() {
        (*queued_result).inner_result = inner_result;
        IRtwqAsyncResult_AddRef((*queued_result).inner_result);
    }
    /* Push resolved object type and created object, so we don't have to guess on End*() call. */
    EnterCriticalSection(&raw mut (*resolver).cs);
    list_add_tail(
        &raw mut (*resolver).pending,
        &raw mut (*queued_result).entry,
    );
    LeaveCriticalSection(&raw mut (*resolver).cs);
    if !(*data).hEvent.is_null() {
        SetEvent((*data).hEvent);
    } else {
        let mut caller_state: *mut IUnknown = IRtwqAsyncResult_GetStateNoAddRef(inner_result);
        let mut caller_result: *mut IRtwqAsyncResult =
            ::core::ptr::null_mut::<IRtwqAsyncResult>();
        if (RtwqCreateAsyncResult(
            (*queued_result).object,
            (*data).pCallback,
            caller_state,
            &raw mut caller_result,
        )) >= 0
        {
            RtwqInvokeCallback(caller_result);
            IRtwqAsyncResult_Release(caller_result);
        }
    }
    return S_OK;
}

unsafe extern "C" fn impl_cancel_obj_from_IUnknown(iface: *mut IUnknown) -> *mut resolver_cancel_object {
    return (iface as *mut u8).sub(core::mem::offset_of!(resolver_cancel_object, IUnknown_iface)) as *mut resolver_cancel_object;
}

unsafe extern "system" fn resolver_cancel_object_QueryInterface(iface: *mut IUnknown, riid: *const IID, obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut ::core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn resolver_cancel_object_AddRef(iface: *mut IUnknown) -> ULONG {
    let object: *mut resolver_cancel_object = impl_cancel_obj_from_IUnknown(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*object).refcount)) as ULONG;
}

unsafe extern "system" fn resolver_cancel_object_Release(iface: *mut IUnknown) -> ULONG {
    let object: *mut resolver_cancel_object = impl_cancel_obj_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*object).refcount) as *mut i32) as ULONG;

    if refcount == 0 {
        if !(*object).cancel_cookie.is_null() {
            IUnknown_Release((*object).cancel_cookie);
        }
        IUnknown_Release((*object).u.handler);
        free(object as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn unsafe_impl_cancel_obj_from_IUnknown(iface: *mut IUnknown) -> *mut resolver_cancel_object {
    if iface.is_null() {
        return core::ptr::null_mut();
    }

    return if (*iface).lpVtbl == core::ptr::addr_of!(resolver_cancel_object_vtbl) as *const _ {
        (iface as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(resolver_cancel_object, IUnknown_iface) as ULONG_PTR,
        ) as *mut resolver_cancel_object
    } else {
        core::ptr::null_mut()
    };
}

unsafe extern "C" fn resolver_create_cancel_object(
    mut handler: *mut IUnknown,
    mut origin: resolved_object_origin,
    mut cancel_cookie: *mut IUnknown,
    mut cancel_object: *mut *mut IUnknown,
) -> HRESULT {
    let mut object: *mut resolver_cancel_object = core::ptr::null_mut();

    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<resolver_cancel_object>() as size_t) as _,
    ) as *mut resolver_cancel_object;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IUnknown_iface.lpVtbl = core::ptr::addr_of!(resolver_cancel_object_vtbl);
    (*object).refcount = 1 as LONG;
    (*object).u.handler = handler;
    IUnknown_AddRef((*object).u.handler);
    (*object).cancel_cookie = cancel_cookie;
    IUnknown_AddRef((*object).cancel_cookie);
    (*object).origin = (origin) as i32;

    *cancel_object = core::ptr::addr_of_mut!((*object).IUnknown_iface);

    return S_OK;
}

unsafe extern "system" fn source_resolver_callback_QueryInterface(iface: *mut IRtwqAsyncCallback, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IRtwqAsyncCallback)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IRtwqAsyncCallback_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn source_resolver_callback_stream_AddRef(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let resolver: *mut source_resolver = impl_from_stream_IRtwqAsyncCallback(iface);
    return IMFSourceResolver_AddRef(core::ptr::addr_of_mut!((*resolver).IMFSourceResolver_iface));
}

unsafe extern "system" fn source_resolver_callback_stream_Release(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let resolver: *mut source_resolver = impl_from_stream_IRtwqAsyncCallback(iface);
    return IMFSourceResolver_Release(core::ptr::addr_of_mut!((*resolver).IMFSourceResolver_iface));
}
unsafe extern "system" fn source_resolver_callback_GetParameters(
    iface: *mut IRtwqAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn source_resolver_callback_stream_Invoke(iface: *mut IRtwqAsyncCallback, result: *mut IRtwqAsyncResult) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_stream_IRtwqAsyncCallback(iface);

    return resolver_handler_end_create(resolver, (OBJECT_FROM_BYTESTREAM) as resolved_object_origin, result);
}

unsafe extern "system" fn source_resolver_callback_url_AddRef(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let resolver: *mut source_resolver = impl_from_url_IRtwqAsyncCallback(iface);
    return IMFSourceResolver_AddRef(core::ptr::addr_of_mut!((*resolver).IMFSourceResolver_iface));
}

unsafe extern "system" fn source_resolver_callback_url_Release(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let resolver: *mut source_resolver = impl_from_url_IRtwqAsyncCallback(iface);
    return IMFSourceResolver_Release(core::ptr::addr_of_mut!((*resolver).IMFSourceResolver_iface));
}

unsafe extern "system" fn source_resolver_callback_url_Invoke(iface: *mut IRtwqAsyncCallback, result: *mut IRtwqAsyncResult) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_url_IRtwqAsyncCallback(iface);

    return resolver_handler_end_create(resolver, (OBJECT_FROM_URL) as resolved_object_origin, result);
}

unsafe extern "C" fn resolver_create_registered_handler(
    mut hkey: HKEY,
    riid: *const IID,
    mut handler: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut name_length: DWORD = 0;
    let mut type_0: DWORD = 0;
    let mut hr: HRESULT = E_FAIL;
    let mut j: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut clsidW: [WCHAR; 39] = [0; 39];
    let mut clsid: CLSID = _GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    name_length = (::core::mem::size_of::<[WCHAR; 39]>() as usize)
        .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
    loop {
        let fresh25 = j;
        j = j.wrapping_add(1);
        if !(RegEnumValueW(
            hkey,
            fresh25 as DWORD,
            &raw mut clsidW as LPWSTR,
            &raw mut name_length,
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut type_0,
            ::core::ptr::null_mut::<BYTE>(),
            ::core::ptr::null_mut::<DWORD>(),
        ) == 0)
        {
            break;
        }
        if type_0 == REG_SZ as DWORD {
            if CLSIDFromString(&raw mut clsidW as *mut WCHAR as LPCOLESTR, &raw mut clsid)
                >= 0 as HRESULT
            {
                hr = CoCreateInstance(
                    &raw mut clsid,
                    ::core::ptr::null_mut::<IUnknown>(),
                    CLSCTX_INPROC_SERVER as ::core::ffi::c_int as DWORD,
                    riid,
                    handler as *mut LPVOID,
                );
                if hr >= 0 as HRESULT {
                    break;
                }
            }
        }
        name_length = (::core::mem::size_of::<[WCHAR; 39]>() as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
    }
    return hr;
}

unsafe extern "C" fn resolver_create_bytestream_handler(stream: *mut IMFByteStream, flags: DWORD, mime: *const WCHAR, extension: *const WCHAR, handler: *mut *mut IMFByteStreamHandler) -> HRESULT {
    static mut hkey_roots: [HKEY; 2] = [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE];
    let mut hr: HRESULT = E_FAIL;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;

    *handler = ::core::ptr::null_mut();

    if (flags & MF_RESOLUTION_DISABLE_LOCAL_PLUGINS) == 0 {
        let mut local_handler: *mut local_handler = ::core::ptr::null_mut();

        EnterCriticalSection(::core::ptr::addr_of_mut!(local_handlers_section));

        local_handler = ((*::core::ptr::addr_of_mut!(local_bytestream_handlers)).next as *mut ::core::ffi::c_char)
            .offset(-(::core::mem::offset_of!(local_handler, entry) as isize)) as *mut local_handler;

        while ::core::ptr::addr_of_mut!((*local_handler).entry) != ::core::ptr::addr_of_mut!(local_bytestream_handlers) {
            if (!mime.is_null() && lstrcmpiW(mime, (*local_handler).u.bytestream.mime) == 0)
                || (!extension.is_null() && lstrcmpiW(extension, (*local_handler).u.bytestream.extension) == 0)
            {
                hr = IMFActivate_ActivateObject(
                    (*local_handler).activate,
                    ::core::ptr::addr_of!(IID_IMFByteStreamHandler),
                    handler as *mut *mut ::core::ffi::c_void,
                );
                if hr >= 0 {
                    break;
                }
            }
            local_handler = ((*local_handler).entry.next as *mut ::core::ffi::c_char)
                .offset(-(::core::mem::offset_of!(local_handler, entry) as isize)) as *mut local_handler;
        }

        LeaveCriticalSection(::core::ptr::addr_of_mut!(local_handlers_section));

        if !(*handler).is_null() {
            return hr;
        }
    }

    i = 0;
    hr = E_FAIL;
    while (i as usize) < (ARRAY_SIZE!(hkey_roots) as usize) {
        let mut namesW: [*const WCHAR; 2] = [mime, extension];
        let mut hkey: HKEY = ::core::ptr::null_mut();
        let mut hkey_handler: HKEY = ::core::ptr::null_mut();

        if RegOpenKeyW(hkey_roots[i as usize], RUSTCONV_WSTR_F31527B5B4F5.as_ptr(), &mut hkey) == 0 {
            j = 0;
            while (j as usize) < (ARRAY_SIZE!(namesW) as usize) {
                if !namesW[j as usize].is_null() {
                    if RegOpenKeyW(hkey, namesW[j as usize], &mut hkey_handler) == 0 {
                        hr = resolver_create_registered_handler(
                            hkey_handler,
                            ::core::ptr::addr_of!(IID_IMFByteStreamHandler),
                            handler as *mut *mut ::core::ffi::c_void,
                        );
                        RegCloseKey(hkey_handler);
                    }

                    if hr >= 0 {
                        break;
                    }
                }
                j = j.wrapping_add(1);
            }

            RegCloseKey(hkey);

            if hr >= 0 {
                break;
            }
        }
        i = i.wrapping_add(1);
    }

    return hr;
}

unsafe extern "C" fn resolver_get_bytestream_url_hint(stream: *mut IMFByteStream, url: *mut *const WCHAR) -> HRESULT {
    static asfmagic: [u8; 16] = [0x30, 0x26, 0xb2, 0x75, 0x8e, 0x66, 0xcf, 0x11, 0xa6, 0xd9, 0x00, 0xaa, 0x00, 0x62, 0xce, 0x6c];
    static wavmagic: [u8; 16] = [b'R', b'I', b'F', b'F', 0x00, 0x00, 0x00, 0x00, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' '];
    static wavmask: [u8; 16] = [0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    static isommagic: [u8; 16] = [0x00, 0x00, 0x00, 0x00, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm', 0x00, 0x00, 0x00, 0x00];
    static mp4_magic: [u8; 16] = [0x00, 0x00, 0x00, 0x00, b'f', b't', b'y', b'p', b'M', b'S', b'N', b'V', 0x00, 0x00, 0x00, 0x00];
    static mp42magic: [u8; 16] = [0x00, 0x00, 0x00, 0x00, b'f', b't', b'y', b'p', b'm', b'p', b'4', b'2', 0x00, 0x00, 0x00, 0x00];
    static mp4vmagic: [u8; 16] = [0x00, 0x00, 0x00, 0x00, b'f', b't', b'y', b'p', b'M', b'4', b'V', b' ', 0x00, 0x00, 0x00, 0x00];
    static mp4mask: [u8; 16] = [0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00];
    static mp3magic: [u8; 16] = [0xff, 0xf2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    static mp3mask: [u8; 16] = [0xff, 0xf6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    static idv2_3_magic: [u8; 16] = [b'I', b'D', b'3', 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    static idv2_3_mask: [u8; 16] = [0xff, 0xff, 0xff, 0xff, 0x00, 0x1f, 0x80, 0x80, 0x80, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[repr(C)]
    struct stream_content_url_hint {
        magic: *const u8,
        url: *const WCHAR,
        mask: *const u8,
    }

    let url_hints: [stream_content_url_hint; 8] = [
        stream_content_url_hint { magic: core::ptr::addr_of!(asfmagic) as *const u8, url: RUSTCONV_WSTR_659752A281E4.as_ptr(), mask: core::ptr::null() },
        stream_content_url_hint { magic: core::ptr::addr_of!(wavmagic) as *const u8, url: RUSTCONV_WSTR_2872FAAF7237.as_ptr(), mask: core::ptr::addr_of!(wavmask) as *const u8 },
        stream_content_url_hint { magic: core::ptr::addr_of!(isommagic) as *const u8, url: RUSTCONV_WSTR_51ABFF51CF66.as_ptr(), mask: core::ptr::addr_of!(mp4mask) as *const u8 },
        stream_content_url_hint { magic: core::ptr::addr_of!(mp42magic) as *const u8, url: RUSTCONV_WSTR_51ABFF51CF66.as_ptr(), mask: core::ptr::addr_of!(mp4mask) as *const u8 },
        stream_content_url_hint { magic: core::ptr::addr_of!(mp4_magic) as *const u8, url: RUSTCONV_WSTR_51ABFF51CF66.as_ptr(), mask: core::ptr::addr_of!(mp4mask) as *const u8 },
        stream_content_url_hint { magic: core::ptr::addr_of!(mp4vmagic) as *const u8, url: RUSTCONV_WSTR_BE99061A9E69.as_ptr(), mask: core::ptr::addr_of!(mp4mask) as *const u8 },
        stream_content_url_hint { magic: core::ptr::addr_of!(mp3magic) as *const u8, url: RUSTCONV_WSTR_DE1FC36E58D3.as_ptr(), mask: core::ptr::addr_of!(mp3mask) as *const u8 },
        stream_content_url_hint { magic: core::ptr::addr_of!(idv2_3_magic) as *const u8, url: RUSTCONV_WSTR_DE1FC36E58D3.as_ptr(), mask: core::ptr::addr_of!(idv2_3_mask) as *const u8 },
    ];

    let mut buffer: [u8; 4 * core::mem::size_of::<u32>()] = [0; 4 * core::mem::size_of::<u32>()];
    let mut pattern: [u8; 4 * core::mem::size_of::<u32>()] = [0; 4 * core::mem::size_of::<u32>()];
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut::<IMFAttributes>();
    let mut length: DWORD = 0;
    let mut caps: DWORD = 0;
    let mut i: u32;
    let mut j: u32;
    let mut position: QWORD = 0;
    let mut hr: HRESULT;

    *url = core::ptr::null();

    if IMFByteStream_QueryInterface(stream, &raw const IID_IMFAttributes, &raw mut attributes as *mut *mut core::ffi::c_void) >= 0 {
        let mut string_length: UINT32 = 0;
        IMFAttributes_GetStringLength(attributes, &raw const MF_BYTESTREAM_CONTENT_TYPE, &raw mut string_length);
        IMFAttributes_Release(attributes);

        if string_length != 0 {
            return S_OK;
        }
    }

    hr = IMFByteStream_GetCapabilities(stream, &raw mut caps);
    if hr < 0 {
        return hr;
    }

    if (caps & MFBYTESTREAM_IS_SEEKABLE) == 0 {
        return MF_E_UNSUPPORTED_BYTESTREAM_TYPE;
    }

    hr = IMFByteStream_GetCurrentPosition(stream, &raw mut position);
    if hr < 0 {
        return hr;
    }
    if position != 0 {
        hr = IMFByteStream_SetCurrentPosition(stream, 0);
        if hr < 0 {
            return hr;
        }
    }
    hr = IMFByteStream_Read(stream, &raw mut buffer as *mut u8, core::mem::size_of_val(&buffer) as u32, &raw mut length);
    if hr < 0 {
        return hr;
    }

    if (length as usize) < core::mem::size_of_val(&buffer) {
        return S_OK;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(url_hints) as usize {
        memcpy(&raw mut pattern as *mut core::ffi::c_void, &raw const buffer as *const core::ffi::c_void, core::mem::size_of_val(&buffer));
        if !url_hints[i as usize].mask.is_null() {
            let mut mask: *mut u32 = url_hints[i as usize].mask as *mut u32;
            let mut data: *mut u32 = &raw mut pattern as *mut u32;

            j = 0;
            while (j as usize) < core::mem::size_of_val(&buffer) / core::mem::size_of::<u32>() {
                *data.offset(j as isize) &= *mask.offset(j as isize);
                j = j.wrapping_add(1);
            }
        }
        if memcmp(&raw const pattern as *const core::ffi::c_void, url_hints[i as usize].magic as *const core::ffi::c_void, core::mem::size_of_val(&pattern)) == 0 {
            *url = url_hints[i as usize].url;
            break;
        }
        i = i.wrapping_add(1);
    }

    if !(*url).is_null() {
        TRACE!("Content type guessed as %s from %s.\n", debugstr_w(*url), debugstr_an(&raw mut buffer as *mut core::ffi::c_char, length as core::ffi::c_int));
    } else {
        WARN!("Unrecognized content type %s.\n", debugstr_an(&raw mut buffer as *mut core::ffi::c_char, length as core::ffi::c_int));
    }

    return S_OK;
}

unsafe extern "C" fn resolver_create_gstreamer_handler(handler: *mut *mut IMFByteStreamHandler) -> HRESULT {
    static CLSID_GStreamerByteStreamHandler: GUID = GUID {
        Data1: 0x317df618,
        Data2: 0x5e5a,
        Data3: 0x468a,
        Data4: [
            0x9f,
            0x15,
            0xd8,
            0x27,
            0xa9,
            0xa0,
            0x81,
            0x62,
        ],
    };
    return CoCreateInstance(
        core::ptr::addr_of!(CLSID_GStreamerByteStreamHandler),
        core::ptr::null_mut::<IUnknown>(),
        CLSCTX_INPROC_SERVER as DWORD,
        core::ptr::addr_of!(IID_IMFByteStreamHandler),
        handler as *mut LPVOID,
    );
}

unsafe extern "C" fn resolver_get_bytestream_handler(
    stream: *mut IMFByteStream,
    mut url: *const WCHAR,
    flags: DWORD,
    handler: *mut *mut IMFByteStreamHandler,
) -> HRESULT {
    let mut mimeW: *mut WCHAR = core::ptr::null_mut();
    let mut urlW: *mut WCHAR = core::ptr::null_mut();
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    let mut url_ext: *const WCHAR = core::ptr::null();
    let mut hr: HRESULT = E_FAIL;
    let mut length: UINT32 = 0;

    *handler = core::ptr::null_mut();

    /* MIME type */
    if IMFByteStream_QueryInterface(
        stream,
        core::ptr::addr_of!(IID_IMFAttributes),
        core::ptr::addr_of_mut!(attributes) as *mut *mut core::ffi::c_void,
    ) >= 0
    {
        IMFAttributes_GetAllocatedString(
            attributes,
            core::ptr::addr_of!(MF_BYTESTREAM_CONTENT_TYPE),
            core::ptr::addr_of_mut!(mimeW),
            core::ptr::addr_of_mut!(length),
        );
        if url.is_null() {
            IMFAttributes_GetAllocatedString(
                attributes,
                core::ptr::addr_of!(MF_BYTESTREAM_ORIGIN_NAME),
                core::ptr::addr_of_mut!(urlW),
                core::ptr::addr_of_mut!(length),
            );
            url = urlW;
        }
        IMFAttributes_Release(attributes);
    }

    /* Extension */
    url_ext = if !url.is_null() {
        wcsrchr(url, '.' as u16)
    } else {
        core::ptr::null_mut()
    };

    /* If content type was provided by the caller, it's tried first. Otherwise an attempt to deduce
       content type from the content itself is made.

       TODO: wine specific fallback to predefined handler could be replaced by normally registering
       this handler for all possible types.
     */

    if !url_ext.is_null() || !mimeW.is_null() {
        hr = resolver_create_bytestream_handler(stream, flags, mimeW, url_ext, handler);

        if hr < 0 {
            hr = resolver_create_gstreamer_handler(handler);
        }
    }

    CoTaskMemFree(mimeW as LPVOID);
    CoTaskMemFree(urlW as LPVOID);

    if hr >= 0 {
        return hr;
    }

    if !url_ext.is_null() && (flags & MF_RESOLUTION_CONTENT_DOES_NOT_HAVE_TO_MATCH_EXTENSION_OR_MIME_TYPE) == 0 {
        return MF_E_UNSUPPORTED_BYTESTREAM_TYPE;
    }

    hr = resolver_get_bytestream_url_hint(stream, core::ptr::addr_of_mut!(url_ext));
    if hr < 0 {
        return hr;
    }

    hr = resolver_create_bytestream_handler(stream, flags, core::ptr::null(), url_ext, handler);

    if hr < 0 {
        hr = resolver_create_gstreamer_handler(handler);
    }

    return hr;
}

unsafe extern "C" fn resolver_create_scheme_handler(
    scheme: *const WCHAR,
    flags: DWORD,
    handler: *mut *mut IMFSchemeHandler,
) -> HRESULT {
    static mut hkey_roots: [HKEY; 2] = [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE];
    let mut hr: HRESULT = MF_E_UNSUPPORTED_SCHEME;
    let mut i: u32;
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"resolver_create_scheme_handler\0".as_ptr() as *const i8,
            6365i32,
            b"%s, %#lx, %p.\n\0".as_ptr() as *const i8,
            debugstr_w(scheme),
            flags,
            handler,
        );
    }
    *handler = core::ptr::null_mut();
    if (flags & MF_RESOLUTION_DISABLE_LOCAL_PLUGINS) == 0 {
        let mut local_handler: *mut local_handler = core::ptr::null_mut();
        EnterCriticalSection(core::ptr::addr_of_mut!(local_handlers_section));
        local_handler = ((*core::ptr::addr_of_mut!(local_scheme_handlers)).next as *mut u8)
            .offset(-(core::mem::offset_of!(local_handler, entry) as isize)) as *mut local_handler;
        while core::ptr::addr_of_mut!((*local_handler).entry) != core::ptr::addr_of_mut!(local_scheme_handlers) {
            if lstrcmpiW(scheme, (*local_handler).u.scheme) == 0 {
                hr = IMFActivate_ActivateObject(
                    (*local_handler).activate,
                    core::ptr::addr_of!(IID_IMFSchemeHandler),
                    handler as *mut *mut core::ffi::c_void,
                );
                if hr >= 0 {
                    break;
                }
            }
            local_handler = ((*local_handler).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(local_handler, entry) as isize)) as *mut local_handler;
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!(local_handlers_section));
        if !(*handler).is_null() {
            return hr;
        }
    }
    i = 0;
    while (i as usize) < ARRAY_SIZE!(hkey_roots) as usize {
        let mut hkey: HKEY = core::ptr::null_mut();
        let mut hkey_handler: HKEY = core::ptr::null_mut();
        hr = MF_E_UNSUPPORTED_SCHEME;
        if RegOpenKeyW(
            hkey_roots[i as usize],
            RUSTCONV_WSTR_4BB168AF0C9D.as_ptr(),
            core::ptr::addr_of_mut!(hkey),
        ) == 0
        {
            if RegOpenKeyW(hkey, scheme, core::ptr::addr_of_mut!(hkey_handler)) == 0 {
                hr = resolver_create_registered_handler(
                    hkey_handler,
                    core::ptr::addr_of!(IID_IMFSchemeHandler),
                    handler as *mut *mut core::ffi::c_void,
                );
                RegCloseKey(hkey_handler);
            }
            RegCloseKey(hkey);
            if hr >= 0 {
                break;
            }
        }
        i = i.wrapping_add(1);
    }
    return hr;
}

unsafe extern "C" fn is_escaped_drive_spec(str: *const WCHAR) -> BOOL {
    return (isalpha(*str.offset(0 as isize) as core::ffi::c_int) != 0
        && (*str.offset(1 as isize) == ':' as WCHAR
            || *str.offset(1 as isize) == '|' as WCHAR)) as BOOL;
}

unsafe extern "C" fn resolver_find_scheme(url: *const WCHAR, len: *mut ::core::ffi::c_uint) -> BOOL {
    let mut ptr: *const WCHAR = url;

    *len = 0;

    if is_escaped_drive_spec(url) != 0 {
        return (FALSE) as BOOL;
    }

    /* RFC 3986: scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ) */
    while *ptr != 0 {
        let mut ch: WCHAR = towlower(*ptr) as WCHAR;

        if *ptr as ::core::ffi::c_int == '*' as i32 && ptr == url {
            ptr = ptr.offset(1);
            break;
        } else if !(*ptr as ::core::ffi::c_int >= '0' as i32
            && *ptr as ::core::ffi::c_int <= '9' as i32)
            && !(ch as ::core::ffi::c_int >= 'a' as i32
                && ch as ::core::ffi::c_int <= 'z' as i32)
            && *ptr as ::core::ffi::c_int != '+' as i32
            && *ptr as ::core::ffi::c_int != '-' as i32
            && *ptr as ::core::ffi::c_int != '.' as i32
        {
            break;
        }

        ptr = ptr.offset(1);
    }

    /* Schemes must end with a ':' */
    if ptr == url || *ptr as ::core::ffi::c_int != ':' as i32 {
        return (FALSE) as BOOL;
    }

    *len = ptr.offset_from(url) as ::core::ffi::c_uint + 1;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn resolver_get_scheme_handler(
    url: *const WCHAR,
    flags: DWORD,
    handler: *mut *mut IMFSchemeHandler,
) -> HRESULT {
    let mut len: ::core::ffi::c_uint = 0;
    let mut scheme: *mut WCHAR = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    if resolver_find_scheme(url, core::ptr::addr_of_mut!(len)) == 0 {
        return MF_E_UNSUPPORTED_SCHEME;
    }
    scheme = malloc(
        ((len.wrapping_add(1 as ::core::ffi::c_uint)) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if scheme.is_null() {
        return E_OUTOFMEMORY;
    }
    memcpy(
        scheme as *mut ::core::ffi::c_void,
        url as *const ::core::ffi::c_void,
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    *scheme.offset(len as isize) = 0 as WCHAR;
    hr = resolver_create_scheme_handler(scheme, flags, handler);
    free(scheme as *mut ::core::ffi::c_void);
    return hr;
}

unsafe extern "C" fn resolver_end_create_object(
    mut resolver: *mut source_resolver,
    mut origin: resolved_object_origin,
    mut result: *mut IRtwqAsyncResult,
    mut obj_type: *mut MF_OBJECT_TYPE,
    mut out: *mut *mut IUnknown,
) -> HRESULT {
    let mut queued_result: *mut resolver_queued_result =
        ::core::ptr::null_mut::<resolver_queued_result>();
    let mut iter: *mut resolver_queued_result = ::core::ptr::null_mut::<resolver_queued_result>();
    let mut object: *mut IUnknown = ::core::ptr::null_mut::<IUnknown>();
    let mut hr: HRESULT = 0;
    hr = IRtwqAsyncResult_GetObject(result, &raw mut object);
    if hr < 0 {
        return hr;
    }
    EnterCriticalSection(&raw mut (*resolver).cs);
    iter = ((*resolver).pending.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(resolver_queued_result, entry)
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut resolver_queued_result;
    while &raw mut (*iter).entry != &raw mut (*resolver).pending {
        if (*iter).inner_result == result
            || (*iter).object == object
                && (*iter).origin as ::core::ffi::c_uint == origin as ::core::ffi::c_uint
        {
            list_remove(&raw mut (*iter).entry);
            queued_result = iter;
            break;
        } else {
            iter = ((*iter).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(resolver_queued_result, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut resolver_queued_result;
        }
    }
    LeaveCriticalSection(&raw mut (*resolver).cs);
    IUnknown_Release(object);
    if !queued_result.is_null() {
        *out = (*queued_result).object;
        *obj_type = (*queued_result).obj_type;
        hr = (*queued_result).hr;
        if !(*queued_result).inner_result.is_null() {
            IRtwqAsyncResult_Release((*queued_result).inner_result);
        }
        free(queued_result as *mut ::core::ffi::c_void);
    } else {
        hr = E_UNEXPECTED;
    }
    return hr;
}

unsafe extern "system" fn source_resolver_QueryInterface(iface: *mut IMFSourceResolver, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSourceResolver)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*resolver).IMFSourceResolver_iface) as *mut core::ffi::c_void;
    } else {
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn source_resolver_AddRef(iface: *mut IMFSourceResolver) -> ULONG {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*resolver).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"source_resolver_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            6549 as ::core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }

    return refcount;
}

unsafe extern "system" fn source_resolver_Release(iface: *mut IMFSourceResolver) -> ULONG {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*resolver).refcount) as *mut core::ffi::c_long,
    ) as ULONG;
    let mut result: *mut resolver_queued_result = core::ptr::null_mut::<resolver_queued_result>();
    let mut result2: *mut resolver_queued_result = core::ptr::null_mut::<resolver_queued_result>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"source_resolver_Release\0" as *const u8 as *const ::core::ffi::c_char,
            6560 as ::core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        result = ((*resolver).pending.next as *mut u8)
            .offset(-(core::mem::offset_of!(resolver_queued_result, entry) as isize))
            as *mut resolver_queued_result;
        result2 = ((*result).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(resolver_queued_result, entry) as isize))
            as *mut resolver_queued_result;
        while core::ptr::addr_of_mut!((*result).entry)
            != core::ptr::addr_of_mut!((*resolver).pending)
        {
            if !(*result).object.is_null() {
                IUnknown_Release((*result).object);
            }
            list_remove(core::ptr::addr_of_mut!((*result).entry));
            free(result as *mut core::ffi::c_void);
            result = result2;
            result2 = ((*result).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(resolver_queued_result, entry) as isize))
                as *mut resolver_queued_result;
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*resolver).cs));
        free(resolver as *mut core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "C" fn resolver_normalise_url(mut url: *const WCHAR) -> *mut WCHAR {
    let filescheme: [WCHAR; 6] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x0000];
    let filescheme_dblslash: *const WCHAR = RUSTCONV_WSTR_1B79C1FA9B08.as_ptr();
    let mut len: ::core::ffi::c_uint = 0;
    let mut slash_count: ::core::ffi::c_uint;
    let mut normalised_url: *mut WCHAR;
    let mut scheme: *const WCHAR;

    if resolver_find_scheme(url, core::ptr::addr_of_mut!(len)) != 0 {
        return wcsdup(url as *const WCHAR) as *mut WCHAR;
    }

    /* If not found try "file:"
     * Native allows leading slashes in file paths. If "file:" is missing, native prepends
     * it before calling BeginCreateObject(), and includes two or three slashes if any were
     * present. The rules don't seem to follow much of a pattern, and may have been designed
     * to get the desired result from PathCreateFromUrlW(). */
    while *url.offset(len as isize) == 0x002Fu16 {
        len = len.wrapping_add(1);
    }
    if len == 0 {
        slash_count = 0;
    } else if is_escaped_drive_spec(url.offset(len as isize) as *const WCHAR) != 0 {
        slash_count = 1
            - ((len == 5 || len == 6) as ::core::ffi::c_uint);
    } else if len < 5 {
        slash_count = len & 1;
    } else {
        slash_count = (len > 6) as ::core::ffi::c_uint;
    }
    url = url.offset(len.wrapping_sub(slash_count) as isize);

    scheme = if len != 0 {
        filescheme_dblslash
    } else {
        core::ptr::addr_of!(filescheme) as *const WCHAR
    };
    len = wcslen(url as *const WCHAR) as ::core::ffi::c_uint;
    normalised_url = malloc(
        (wcslen(scheme as *const WCHAR) as usize
            + len as usize
            + 1 as usize)
            * ::core::mem::size_of::<WCHAR>(),
    ) as *mut WCHAR;
    if normalised_url.is_null() {
        return core::ptr::null_mut();
    }

    wcscpy(normalised_url, scheme as *const WCHAR);
    wcscat(normalised_url, url as *const WCHAR);

    return normalised_url;
}

unsafe extern "system" fn source_resolver_CreateObjectFromURL(iface: *mut IMFSourceResolver, url: *const WCHAR, flags: DWORD, props: *mut IPropertyStore, obj_type: *mut MF_OBJECT_TYPE, object: *mut *mut IUnknown) -> HRESULT {
    let mut resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    let mut normalised_url: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut handler: *mut IMFSchemeHandler = core::ptr::null_mut::<IMFSchemeHandler>();
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut::<IRtwqAsyncResult>();
    let mut data: *mut RTWQASYNCRESULT = core::ptr::null_mut::<RTWQASYNCRESULT>();
    let mut hr: HRESULT = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"source_resolver_CreateObjectFromURL\0" as *const u8 as *const ::core::ffi::c_char,
            6626 as ::core::ffi::c_int,
            b"%p, %s, %#lx, %p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_w(url),
            flags,
            props,
            obj_type,
            object,
        );
    }

    if url.is_null() || obj_type.is_null() || object.is_null() {
        return E_POINTER;
    }

    normalised_url = resolver_normalise_url(url);
    if normalised_url.is_null() {
        return E_OUTOFMEMORY;
    }

    'done: {
        hr = resolver_get_scheme_handler(normalised_url, flags, &raw mut handler);
        if hr < 0 {
            break 'done;
        }

        hr = RtwqCreateAsyncResult(
            handler as *mut IUnknown,
            core::ptr::null_mut::<IRtwqAsyncCallback>(),
            core::ptr::null_mut::<IUnknown>(),
            &raw mut result,
        );
        IMFSchemeHandler_Release(handler);
        if hr < 0 {
            break 'done;
        }

        data = result as *mut RTWQASYNCRESULT;
        (*data).hEvent = CreateEventW(
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            (FALSE) as BOOL,
            (FALSE) as BOOL,
            core::ptr::null::<WCHAR>(),
        );

        hr = IMFSchemeHandler_BeginCreateObject(
            handler,
            normalised_url,
            flags,
            props,
            core::ptr::null_mut::<*mut IUnknown>(),
            &raw mut (*resolver).url_callback as *mut IMFAsyncCallback,
            result as *mut IUnknown,
        );
        if hr < 0 {
            IRtwqAsyncResult_Release(result);
            break 'done;
        }

        WaitForSingleObject((*data).hEvent, INFINITE as DWORD);

        hr = resolver_end_create_object(resolver, (OBJECT_FROM_URL) as resolved_object_origin, result, obj_type, object);
        IRtwqAsyncResult_Release(result);
    }

    free(normalised_url as *mut ::core::ffi::c_void);
    return hr;
}

unsafe extern "system" fn source_resolver_CreateObjectFromByteStream(
    iface: *mut IMFSourceResolver,
    stream: *mut IMFByteStream,
    url: *const WCHAR,
    flags: DWORD,
    props: *mut IPropertyStore,
    obj_type: *mut MF_OBJECT_TYPE,
    object: *mut *mut IUnknown,
) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    let mut handler: *mut IMFByteStreamHandler = core::ptr::null_mut();
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut data: *mut RTWQASYNCRESULT = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"source_resolver_CreateObjectFromByteStream\0" as *const u8
                as *const i8,
            6672i32,
            b"%p, %p, %s, %#lx, %p, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            stream,
            debugstr_w(url),
            flags,
            props,
            obj_type,
            object,
        );
    }

    if stream.is_null() || obj_type.is_null() || object.is_null() {
        return E_POINTER;
    }

    hr = resolver_get_bytestream_handler(stream, url, flags, &mut handler);
    if hr < 0 {
        return MF_E_UNSUPPORTED_BYTESTREAM_TYPE;
    }

    hr = RtwqCreateAsyncResult(
        handler as *mut IUnknown,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut result,
    );
    IMFByteStreamHandler_Release(handler);
    if hr < 0 {
        return hr;
    }

    data = result as *mut RTWQASYNCRESULT;
    (*data).hEvent = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null(),
    );

    hr = IMFByteStreamHandler_BeginCreateObject(
        handler,
        stream,
        url,
        flags,
        props,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*resolver).stream_callback) as *mut IMFAsyncCallback,
        result as *mut IUnknown,
    );
    if hr < 0 {
        IRtwqAsyncResult_Release(result);
        return hr;
    }

    WaitForSingleObject((*data).hEvent, INFINITE as DWORD);

    hr = resolver_end_create_object(
        resolver,
        (OBJECT_FROM_BYTESTREAM) as resolved_object_origin,
        result,
        obj_type,
        object,
    );
    IRtwqAsyncResult_Release(result);

    hr
}

unsafe extern "system" fn source_resolver_BeginCreateObjectFromURL(
    iface: *mut IMFSourceResolver,
    url: *const WCHAR,
    flags: DWORD,
    props: *mut IPropertyStore,
    cancel_cookie: *mut *mut IUnknown,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    let mut handler: *mut IMFSchemeHandler = core::ptr::null_mut::<IMFSchemeHandler>();
    let mut inner_cookie: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut::<IRtwqAsyncResult>();
    let mut normalised_url: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut hr: HRESULT = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"source_resolver_BeginCreateObjectFromURL\0" as *const u8
                as *const ::core::ffi::c_char,
            6714 as ::core::ffi::c_int,
            b"%p, %s, %#lx, %p, %p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_w(url),
            flags,
            props,
            cancel_cookie,
            callback,
            state,
        );
    }

    normalised_url = resolver_normalise_url(url);
    if normalised_url.is_null() {
        return E_OUTOFMEMORY;
    }

    'done: {
        hr = resolver_get_scheme_handler(normalised_url, flags, core::ptr::addr_of_mut!(handler));
        if hr < 0 {
            break 'done;
        }

        if !cancel_cookie.is_null() {
            *cancel_cookie = core::ptr::null_mut::<IUnknown>();
        }

        hr = RtwqCreateAsyncResult(
            handler as *mut IUnknown,
            callback as *mut IRtwqAsyncCallback,
            state,
            core::ptr::addr_of_mut!(result),
        );
        IMFSchemeHandler_Release(handler);
        if hr < 0 {
            break 'done;
        }

        hr = IMFSchemeHandler_BeginCreateObject(
            handler,
            normalised_url,
            flags,
            props,
            if !cancel_cookie.is_null() {
                core::ptr::addr_of_mut!(inner_cookie)
            } else {
                core::ptr::null_mut::<*mut IUnknown>()
            },
            core::ptr::addr_of_mut!((*resolver).url_callback) as *mut IMFAsyncCallback,
            result as *mut IUnknown,
        );

        if hr >= 0 && !inner_cookie.is_null() {
            resolver_create_cancel_object(
                handler as *mut IUnknown,
                (OBJECT_FROM_URL) as resolved_object_origin,
                inner_cookie,
                cancel_cookie,
            );
            IUnknown_Release(inner_cookie);
        }

        IRtwqAsyncResult_Release(result);
    }

    free(normalised_url as *mut ::core::ffi::c_void);
    return hr;
}

unsafe extern "system" fn source_resolver_EndCreateObjectFromURL(
    iface: *mut IMFSourceResolver,
    result: *mut IMFAsyncResult,
    obj_type: *mut MF_OBJECT_TYPE,
    object: *mut *mut IUnknown,
) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    TRACE!("%p, %p, %p, %p.\n", iface, result, obj_type, object);
    return resolver_end_create_object(
        resolver,
        (OBJECT_FROM_URL) as resolved_object_origin,
        result as *mut IRtwqAsyncResult,
        obj_type,
        object,
    );
}

unsafe extern "system" fn source_resolver_BeginCreateObjectFromByteStream(
    iface: *mut IMFSourceResolver,
    stream: *mut IMFByteStream,
    url: *const WCHAR,
    flags: DWORD,
    props: *mut IPropertyStore,
    cancel_cookie: *mut *mut IUnknown,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    let mut handler: *mut IMFByteStreamHandler = core::ptr::null_mut::<IMFByteStreamHandler>();
    let mut inner_cookie: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut::<IRtwqAsyncResult>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"source_resolver_BeginCreateObjectFromByteStream\0" as *const u8
                as *const ::core::ffi::c_char,
            6767 as ::core::ffi::c_int,
            b"%p, %p, %s, %#lx, %p, %p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            stream,
            debugstr_w(url),
            flags,
            props,
            cancel_cookie,
            callback,
            state,
        );
    }

    hr = resolver_get_bytestream_handler(stream, url, flags, &raw mut handler);
    if hr < 0 {
        return hr;
    }

    if !cancel_cookie.is_null() {
        *cancel_cookie = core::ptr::null_mut::<IUnknown>();
    }

    hr = RtwqCreateAsyncResult(
        handler as *mut IUnknown,
        callback as *mut IRtwqAsyncCallback,
        state,
        &raw mut result,
    );
    IMFByteStreamHandler_Release(handler);
    if hr < 0 {
        return hr;
    }

    hr = IMFByteStreamHandler_BeginCreateObject(
        handler,
        stream,
        url,
        flags,
        props,
        if !cancel_cookie.is_null() {
            &raw mut inner_cookie
        } else {
            core::ptr::null_mut::<*mut IUnknown>()
        },
        &raw mut (*resolver).stream_callback as *mut IMFAsyncCallback,
        result as *mut IUnknown,
    );

    /* Cancel object wraps underlying handler cancel cookie with context necessary to call CancelObjectCreate(). */
    if hr >= 0 && !inner_cookie.is_null() {
        resolver_create_cancel_object(
            handler as *mut IUnknown,
            (OBJECT_FROM_BYTESTREAM) as resolved_object_origin,
            inner_cookie,
            cancel_cookie,
        );
        IUnknown_Release(inner_cookie);
    }

    IRtwqAsyncResult_Release(result);

    return hr;
}

unsafe extern "system" fn source_resolver_EndCreateObjectFromByteStream(iface: *mut IMFSourceResolver, result: *mut IMFAsyncResult, obj_type: *mut MF_OBJECT_TYPE, object: *mut *mut IUnknown) -> HRESULT {
    let resolver: *mut source_resolver = impl_from_IMFSourceResolver(iface);
    TRACE!("%p, %p, %p, %p.\n", iface, result, obj_type, object);
    return resolver_end_create_object(resolver, (OBJECT_FROM_BYTESTREAM) as resolved_object_origin, result as *mut IRtwqAsyncResult, obj_type, object);
}
unsafe extern "system" fn source_resolver_CancelObjectCreation(iface: *mut IMFSourceResolver, cancel_cookie: *mut IUnknown) -> HRESULT {
    let mut object: *mut resolver_cancel_object;
    let mut hr: HRESULT;

    object = unsafe_impl_cancel_obj_from_IUnknown(cancel_cookie);
    if object.is_null() {
        return E_UNEXPECTED;
    }
    match ((*object).origin as ::core::ffi::c_uint) as i32 {
        OBJECT_FROM_BYTESTREAM => {
            hr = IMFByteStreamHandler_CancelObjectCreation((*object).u.stream_handler, (*object).cancel_cookie);
        }
        OBJECT_FROM_URL => {
            hr = IMFSchemeHandler_CancelObjectCreation((*object).u.scheme_handler, (*object).cancel_cookie);
        }
        _ => {
            hr = E_UNEXPECTED;
        }
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSourceResolver(resolver: *mut *mut IMFSourceResolver) -> HRESULT {
    let mut object: *mut source_resolver = core::ptr::null_mut();

    if resolver.is_null() {
        return E_POINTER;
    }

    object = calloc(1, (core::mem::size_of::<source_resolver>() as usize) as _) as *mut source_resolver;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFSourceResolver_iface.lpVtbl = core::ptr::addr_of!(mfsourceresolvervtbl);
    (*object).stream_callback.lpVtbl = core::ptr::addr_of!(source_resolver_callback_stream_vtbl);
    (*object).url_callback.lpVtbl = core::ptr::addr_of!(source_resolver_callback_url_vtbl);
    (*object).refcount = 1;
    list_init(core::ptr::addr_of_mut!((*object).pending));
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    *resolver = core::ptr::addr_of_mut!((*object).IMFSourceResolver_iface);

    return S_OK;
}

unsafe extern "C" fn impl_from_IMFMediaEvent(iface: *mut IMFMediaEvent) -> *mut media_event {
    return CONTAINING_RECORD!(iface, media_event, IMFMediaEvent_iface);
}

unsafe extern "system" fn mfmediaevent_QueryInterface(iface: *mut IMFMediaEvent, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false ||
       IsEqualGUID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != false ||
       IsEqualGUID!(riid, core::ptr::addr_of!(IID_IMFMediaEvent)) != false
    {
        *out = core::ptr::addr_of_mut!((*event).IMFMediaEvent_iface) as *mut core::ffi::c_void;
    }
    else
    {
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn mfmediaevent_AddRef(iface: *mut IMFMediaEvent) -> ULONG {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*event).attributes.ref_) as *mut i32,
    ) as ULONG;

    if (*(*core::ptr::addr_of!(__wine_dbch___default))).flags as i32
        & ((1i32) << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const i8,
            b"mfmediaevent_AddRef\0".as_ptr() as *const i8,
            6915i32,
            b"%p, refcount %lu.\n\0".as_ptr() as *const i8,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn mfmediaevent_Release(iface: *mut IMFMediaEvent) -> ULONG {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*event).attributes.ref_) as *mut i32,
    ) as ULONG;
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"mfmediaevent_Release\0" as *const u8 as *const i8,
            6925i32,
            b"%p, refcount %lu.\n\0" as *const u8 as *const i8,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        clear_attributes_object(core::ptr::addr_of_mut!((*event).attributes));
        PropVariantClear(core::ptr::addr_of_mut!((*event).value));
        free(event as *mut core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "system" fn mfmediaevent_GetItem(iface: *mut IMFMediaEvent, key: REFGUID, value: *mut PROPVARIANT) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetItem(core::ptr::addr_of_mut!((*event).attributes), key, value);
}

unsafe extern "system" fn mfmediaevent_GetItemType(iface: *mut IMFMediaEvent, key: *const GUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetItemType(core::ptr::addr_of_mut!((*event).attributes), key, type_);
}

unsafe extern "system" fn mfmediaevent_CompareItem(iface: *mut IMFMediaEvent, key: *const GUID, value: *const PROPVARIANT, result: *mut BOOL) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_CompareItem\0" as *const u8 as *const ::core::ffi::c_char,
            6959 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return attributes_CompareItem(core::ptr::addr_of_mut!((*event).attributes), key, value, result);
}

unsafe extern "system" fn mfmediaevent_Compare(iface: *mut IMFMediaEvent, attrs: *mut IMFAttributes, type_: MF_ATTRIBUTES_MATCH_TYPE, result: *mut BOOL) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_Compare(core::ptr::addr_of_mut!((*event).attributes), attrs, type_, result);
}

unsafe extern "system" fn mfmediaevent_GetUINT32(
    mut iface: *mut IMFMediaEvent,
    key: *const GUID,
    mut value: *mut UINT32,
) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_GetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            6978 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT32(core::ptr::addr_of_mut!((*event).attributes), key, value);
}

unsafe extern "system" fn mfmediaevent_GetUINT64(
    mut iface: *mut IMFMediaEvent,
    key: *const GUID,
    mut value: *mut UINT64,
) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"mfmediaevent_GetUINT64\0" as *const u8 as *const i8,
            6987 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT64(&raw mut (*event).attributes, key, value);
}

unsafe extern "system" fn mfmediaevent_GetDouble(iface: *mut IMFMediaEvent, key: *const GUID, value: *mut f64) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"mfmediaevent_GetDouble\0" as *const u8 as *const i8,
            6996 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetDouble(&raw mut (*event).attributes, key, value);
}

unsafe extern "system" fn mfmediaevent_GetGUID(
    iface: *mut IMFMediaEvent,
    key: *const GUID,
    value: *mut GUID,
) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_GetGUID\0" as *const u8 as *const ::core::ffi::c_char,
            7005 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetGUID(&raw mut (*event).attributes, key, value);
}

unsafe extern "system" fn mfmediaevent_GetStringLength(iface: *mut IMFMediaEvent, key: REFGUID, length: *mut UINT32) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetStringLength(core::ptr::addr_of_mut!((*event).attributes), key, length);
}

unsafe extern "system" fn mfmediaevent_GetString(
    iface: *mut IMFMediaEvent,
    key: REFGUID,
    value: *mut WCHAR,
    size: UINT32,
    length: *mut UINT32,
) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfmediaevent_GetString\0".as_ptr() as *const i8,
            7024 as i32,
            b"%p, %s, %p, %u, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
            size,
            length,
        );
    }
    return attributes_GetString(&raw mut (*event).attributes, key, value, size, length);
}

unsafe extern "system" fn mfmediaevent_GetAllocatedString(iface: *mut IMFMediaEvent, key: *const GUID, value: *mut *mut u16, length: *mut u32) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    TRACE!("%p, %s, %p, %p.\n", iface, debugstr_attr(key), value, length);
    return attributes_GetAllocatedString(&raw mut (*event).attributes, key, value, length);
}

unsafe extern "system" fn mfmediaevent_GetBlobSize(iface: *mut IMFMediaEvent, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetBlobSize(core::ptr::addr_of_mut!((*event).attributes), key, size);
}

unsafe extern "system" fn mfmediaevent_GetBlob(iface: *mut IMFMediaEvent, key: REFGUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetBlob(&raw mut (*event).attributes, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn mfmediaevent_GetAllocatedBlob(iface: *mut IMFMediaEvent, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetAllocatedBlob(core::ptr::addr_of_mut!((*event).attributes), key, buf, size);
}

unsafe extern "system" fn mfmediaevent_GetUnknown(iface: *mut IMFMediaEvent, key: REFGUID, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetUnknown(core::ptr::addr_of_mut!((*event).attributes), key, riid, out);
}

unsafe extern "system" fn mfmediaevent_SetItem(
    iface: *mut IMFMediaEvent,
    key: *const GUID,
    value: *const PROPVARIANT,
) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
            7080 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
        );
    }
    return attributes_SetItem(core::ptr::addr_of_mut!((*event).attributes), key, value);
}

unsafe extern "system" fn mfmediaevent_DeleteItem(iface: *mut IMFMediaEvent, key: REFGUID) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"mfmediaevent_DeleteItem\0" as *const u8 as *const i8,
            7089 as i32,
            b"%p, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
        );
    }
    return attributes_DeleteItem(core::ptr::addr_of_mut!((*event).attributes), key);
}
unsafe extern "system" fn mfmediaevent_DeleteAllItems(mut iface: *mut IMFMediaEvent) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"mfmediaevent_DeleteAllItems\0".as_ptr() as *const core::ffi::c_char,
            7098 as core::ffi::c_int,
            b"%p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
        );
    }
    return attributes_DeleteAllItems(&raw mut (*event).attributes);
}

unsafe extern "system" fn mfmediaevent_SetUINT32(
    iface: *mut IMFMediaEvent,
    key: *const GUID,
    value: UINT32,
) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_SetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            7107 as ::core::ffi::c_int,
            b"%p, %s, %u.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_SetUINT32(core::ptr::addr_of_mut!((*event).attributes), key, value);
}

unsafe extern "system" fn mfmediaevent_SetUINT64(iface: *mut IMFMediaEvent, key: REFGUID, value: UINT64) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_SetUINT64(core::ptr::addr_of_mut!((*event).attributes), key, value);
}

unsafe extern "system" fn mfmediaevent_SetDouble(iface: *mut IMFMediaEvent, key: *const GUID, value: f64) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_SetDouble(core::ptr::addr_of_mut!((*event).attributes), key, value);
}

unsafe extern "system" fn mfmediaevent_SetGUID(iface: *mut IMFMediaEvent, key: REFGUID, value: REFGUID) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_SetGUID(core::ptr::addr_of_mut!((*event).attributes), key, value);
}

unsafe extern "system" fn mfmediaevent_SetString(
    mut iface: *mut IMFMediaEvent,
    key: *const GUID,
    mut value: *const WCHAR,
) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"mfmediaevent_SetString\0" as *const u8 as *const i8,
            7143 as i32,
            b"%p, %s, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_w(value),
        );
    }
    return attributes_SetString(
        core::ptr::addr_of_mut!((*event).attributes),
        key,
        value,
    );
}

unsafe extern "system" fn mfmediaevent_SetBlob(
    iface: *mut IMFMediaEvent,
    key: REFGUID,
    buf: *const UINT8,
    size: UINT32,
) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"mfmediaevent_SetBlob\0".as_ptr() as *const core::ffi::c_char,
            7152 as core::ffi::c_int,
            b"%p, %s, %p, %u.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            buf,
            size,
        );
    }
    return attributes_SetBlob(core::ptr::addr_of_mut!((*event).attributes), key, buf, size);
}

unsafe extern "system" fn mfmediaevent_SetUnknown(iface: *mut IMFMediaEvent, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_SetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            7161 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            unknown,
        );
    }
    return attributes_SetUnknown(core::ptr::addr_of_mut!((*event).attributes), key, unknown);
}

unsafe extern "system" fn mfmediaevent_LockStore(iface: *mut IMFMediaEvent) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_LockStore(core::ptr::addr_of_mut!((*event).attributes));
}

unsafe extern "system" fn mfmediaevent_UnlockStore(iface: *mut IMFMediaEvent) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const i8,
            b"mfmediaevent_UnlockStore\0" as *const u8 as *const i8,
            7179 as i32,
            b"%p.\n\0" as *const u8 as *const i8,
            iface,
        );
    }
    return attributes_UnlockStore(core::ptr::addr_of_mut!((*event).attributes));
}
unsafe extern "system" fn mfmediaevent_GetCount(
    mut iface: *mut IMFMediaEvent,
    mut count: *mut UINT32,
) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_GetCount\0" as *const u8 as *const ::core::ffi::c_char,
            7188 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            count,
        );
    }
    return attributes_GetCount(&raw mut (*event).attributes, count);
}

unsafe extern "system" fn mfmediaevent_GetItemByIndex(iface: *mut IMFMediaEvent, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    return attributes_GetItemByIndex(core::ptr::addr_of_mut!((*event).attributes), index, key, value);
}

unsafe extern "system" fn mfmediaevent_CopyAllItems(iface: *mut IMFMediaEvent, dest: *mut IMFAttributes) -> HRESULT {
    let mut event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"mfmediaevent_CopyAllItems\0".as_ptr() as *const i8,
            7206 as i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface,
            dest,
        );
    }
    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*event).attributes), dest);
}

unsafe extern "system" fn mfmediaevent_GetType(iface: *mut IMFMediaEvent, type_: *mut MediaEventType) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    *type_ = (*event).r#type;

    return S_OK;
}

unsafe extern "system" fn mfmediaevent_GetExtendedType(iface: *mut IMFMediaEvent, extended_type: *mut GUID) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    *extended_type = (*event).extended_type;

    return S_OK;
}

unsafe extern "system" fn mfmediaevent_GetStatus(iface: *mut IMFMediaEvent, status: *mut HRESULT) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);

    *status = (*event).status;

    return S_OK;
}

unsafe extern "system" fn mfmediaevent_GetValue(iface: *mut IMFMediaEvent, value: *mut PROPVARIANT) -> HRESULT {
    let event: *mut media_event = impl_from_IMFMediaEvent(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfmediaevent_GetValue\0" as *const u8 as *const ::core::ffi::c_char,
            7248 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            value,
        );
    }
    PropVariantCopy(value, core::ptr::addr_of_mut!((*event).value));
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMediaEvent(
    type_0: MediaEventType,
    extended_type: *const GUID,
    status: HRESULT,
    value: *const PROPVARIANT,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let mut object: *mut media_event = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = malloc(core::mem::size_of::<media_event>() as usize) as *mut media_event;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(core::ptr::addr_of_mut!((*object).attributes), 0);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }
    (*object).IMFMediaEvent_iface.lpVtbl = core::ptr::addr_of!(mfmediaevent_vtbl);

    (*object).r#type = type_0;
    (*object).extended_type = *extended_type;
    (*object).status = status;

    PropVariantInit!(core::ptr::addr_of_mut!((*object).value));
    if !value.is_null() {
        PropVariantCopy(core::ptr::addr_of_mut!((*object).value), value);
    }

    *event = core::ptr::addr_of_mut!((*object).IMFMediaEvent_iface);

    return S_OK;
}

unsafe extern "C" fn impl_from_IMFMediaEventQueue(iface: *mut IMFMediaEventQueue) -> *mut event_queue {
    return (iface as ULONG_PTR).wrapping_sub(core::mem::offset_of!(event_queue, IMFMediaEventQueue_iface) as ULONG_PTR) as *mut event_queue;
}

unsafe extern "C" fn queue_pop_event(queue: *mut event_queue) -> *mut IMFMediaEvent {
    let mut head: *mut list = list_head(core::ptr::addr_of_mut!((*queue).events));
    let mut queued_event: *mut queued_event = core::ptr::null_mut::<queued_event>();
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut::<IMFMediaEvent>();
    if head.is_null() {
        return core::ptr::null_mut::<IMFMediaEvent>();
    }
    queued_event = (head as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(queued_event, entry) as core::ffi::c_ulonglong
            as isize),
    ) as *mut queued_event;
    event = (*queued_event).event;
    list_remove(core::ptr::addr_of_mut!((*queued_event).entry));
    free(queued_event as *mut core::ffi::c_void);
    return event;
}

unsafe extern "C" fn event_queue_clear_subscriber(queue: *mut event_queue) {
    if !(*queue).subscriber.is_null() {
        IRtwqAsyncResult_Release((*queue).subscriber);
    }
    (*queue).subscriber = core::ptr::null_mut();
}

unsafe extern "C" fn event_queue_cleanup(queue: *mut event_queue) {
    let mut event: *mut IMFMediaEvent;
    loop {
        event = queue_pop_event(queue);
        if event.is_null() {
            break;
        }
        IMFMediaEvent_Release(event);
    }
    event_queue_clear_subscriber(queue);
}

unsafe extern "system" fn eventqueue_QueryInterface(iface: *mut IMFMediaEventQueue, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    TRACE!("%p, %s, %p.\n", iface, riid, out);
    if IsEqualIID!(riid, &raw const IID_IMFMediaEventQueue) != 0
        || IsEqualIID!(riid, &raw const IID_IUnknown) != 0
    {
        *out = &raw mut (*queue).IMFMediaEventQueue_iface as *mut core::ffi::c_void;
        IMFMediaEventQueue_AddRef(iface);
        return S_OK;
    }
    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn eventqueue_AddRef(iface: *mut IMFMediaEventQueue) -> ULONG {
    let queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    let refcount: ULONG = InterlockedIncrement(&raw mut (*queue).refcount as *mut ::core::ffi::c_long) as ULONG;

    return refcount;
}

unsafe extern "system" fn eventqueue_Release(iface: *mut IMFMediaEventQueue) -> ULONG {
    let queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*queue).refcount) as *mut i32) as ULONG;

    TRACE!("%p, refcount %lu.\n", queue, refcount);

    if refcount == 0 {
        event_queue_cleanup(queue);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
        free(queue as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn eventqueue_GetEvent(iface: *mut IMFMediaEventQueue, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let mut queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    let mut hr: HRESULT = S_OK;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"eventqueue_GetEvent\0" as *const u8 as *const ::core::ffi::c_char,
            7441 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            event,
        );
    }
    EnterCriticalSection(&raw mut (*queue).cs);

    if (*queue).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else if !(*queue).subscriber.is_null() {
        hr = MF_E_MULTIPLE_SUBSCRIBERS;
    } else {
        if (flags & MF_EVENT_FLAG_NO_WAIT as DWORD) != 0 {
            *event = queue_pop_event(queue);
            if (*event).is_null() {
                hr = MF_E_NO_EVENTS_AVAILABLE;
            }
        } else {
            while (list_empty(&raw mut (*queue).events) != 0) && (*queue).is_shut_down == 0 {
                SleepConditionVariableCS(&raw mut (*queue).update_event, &raw mut (*queue).cs, INFINITE as DWORD);
            }
            *event = queue_pop_event(queue);
            if (*queue).is_shut_down != 0 {
                hr = MF_E_SHUTDOWN;
            }
        }
    }

    LeaveCriticalSection(&raw mut (*queue).cs);

    return hr;
}

unsafe extern "C" fn queue_notify_subscriber(queue: *mut event_queue) {
    if list_empty(core::ptr::addr_of_mut!((*queue).events)) != 0
        || (*queue).subscriber.is_null()
        || (*queue).notified != 0
    {
        return;
    }

    (*queue).notified = (TRUE as i32);
    RtwqPutWorkItem(MFASYNC_CALLBACK_QUEUE_STANDARD, 0, (*queue).subscriber);
}

unsafe extern "system" fn eventqueue_BeginGetEvent(iface: *mut IMFMediaEventQueue, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT {
    let mut queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    let mut result_data: *mut RTWQASYNCRESULT = core::ptr::null_mut::<RTWQASYNCRESULT>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"eventqueue_BeginGetEvent\0" as *const u8 as *const core::ffi::c_char,
            7488 as core::ffi::c_int,
            b"%p, %p, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            callback,
            state,
        );
    }

    if callback.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*queue).cs));

    if (*queue).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        result_data = (*queue).subscriber as *mut RTWQASYNCRESULT;
        if !result_data.is_null() {
            if (*result_data).pCallback == callback as *mut IRtwqAsyncCallback {
                hr = if IRtwqAsyncResult_GetStateNoAddRef((*queue).subscriber) == state {
                    MF_S_MULTIPLE_BEGIN
                } else {
                    MF_E_MULTIPLE_BEGIN
                };
            } else {
                hr = MF_E_MULTIPLE_SUBSCRIBERS;
            }
        } else {
            hr = RtwqCreateAsyncResult(
                core::ptr::null_mut::<IUnknown>(),
                callback as *mut IRtwqAsyncCallback,
                state,
                core::ptr::addr_of_mut!((*queue).subscriber),
            );
            if hr >= 0 {
                queue_notify_subscriber(queue);
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*queue).cs));

    return hr;
}

unsafe extern "system" fn eventqueue_EndGetEvent(iface: *mut IMFMediaEventQueue, result: *mut IMFAsyncResult, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    let mut hr: HRESULT = E_FAIL;
    EnterCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    if (*queue).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else if (*queue).subscriber == result as *mut IRtwqAsyncResult {
        *event = queue_pop_event(queue);
        event_queue_clear_subscriber(queue);
        (*queue).notified = (FALSE as i32);
        hr = if !(*event).is_null() { S_OK } else { E_FAIL };
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    return hr;
}

unsafe extern "C" fn eventqueue_queue_event(
    mut queue: *mut event_queue,
    mut event: *mut IMFMediaEvent,
) -> HRESULT {
    let mut queued_event: *mut queued_event = core::ptr::null_mut::<queued_event>();
    let mut hr: HRESULT = S_OK;
    queued_event = malloc(core::mem::size_of::<queued_event>() as size_t) as *mut queued_event;
    if queued_event.is_null() {
        return E_OUTOFMEMORY;
    }
    (*queued_event).event = event;
    EnterCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    if (*queue).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        IMFMediaEvent_AddRef((*queued_event).event);
        list_add_tail(core::ptr::addr_of_mut!((*queue).events), core::ptr::addr_of_mut!((*queued_event).entry));
        queue_notify_subscriber(queue);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    if (hr) < 0 {
        free(queued_event as *mut core::ffi::c_void);
    }
    WakeAllConditionVariable(core::ptr::addr_of_mut!((*queue).update_event));
    return hr;
}

unsafe extern "system" fn eventqueue_QueueEvent(iface: *mut IMFMediaEventQueue, event: *mut IMFMediaEvent) -> HRESULT {
    let queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    TRACE!("%p, %p.\n", iface, event);
    return eventqueue_queue_event(queue, event);
}

unsafe extern "system" fn eventqueue_QueueEventParamVar(iface: *mut IMFMediaEventQueue, event_type: MediaEventType, extended_type: REFGUID, status: HRESULT, value: *const PROPVARIANT) -> HRESULT {
    let queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    let mut event: *mut IMFMediaEvent = core::mem::zeroed();
    let mut hr: HRESULT;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"eventqueue_QueueEventParamVar\0" as *const u8 as *const ::core::ffi::c_char,
            7590 as ::core::ffi::c_int,
            b"%p, %s, %s, %#lx, %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_eventid(event_type as DWORD),
            debugstr_guid(extended_type as *const _GUID),
            status,
            debugstr_propvar(value),
        );
    }

    hr = MFCreateMediaEvent(event_type, extended_type, status, value, core::ptr::addr_of_mut!(event));
    if hr < 0 {
        return hr;
    }

    hr = eventqueue_queue_event(queue, event);
    IMFMediaEvent_Release(event);
    return hr;
}

unsafe extern "system" fn eventqueue_QueueEventParamUnk(
    mut iface: *mut IMFMediaEventQueue,
    mut event_type: MediaEventType,
    extended_type: *const GUID,
    mut status: HRESULT,
    mut unk: *mut IUnknown,
) -> HRESULT {
    let mut queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);
    let mut event: *mut IMFMediaEvent = ::core::ptr::null_mut::<IMFMediaEvent>();
    let mut value: PROPVARIANT = tagPROPVARIANT {
        vt: 0,
        wReserved1: 0,
        wReserved2: 0,
        wReserved3: 0,
        __bindgen_anon_1: tagPROPVARIANT__bindgen_ty_1 { cVal: 0 },
    };
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"eventqueue_QueueEventParamUnk\0" as *const u8 as *const ::core::ffi::c_char,
            7608 as ::core::ffi::c_int,
            b"%p, %s, %s, %#lx, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_eventid(event_type as DWORD),
            debugstr_guid(extended_type as *const _GUID),
            status,
            unk,
        );
    }
    value.vt = VT_UNKNOWN;
    value.__bindgen_anon_1.punkVal = unk;

    hr = MFCreateMediaEvent(
        event_type,
        extended_type,
        status,
        &raw mut value,
        &raw mut event,
    );
    if hr < 0
    {
        return hr;
    }

    hr = eventqueue_queue_event(queue, event);
    IMFMediaEvent_Release(event);
    return hr;
}

unsafe extern "system" fn eventqueue_Shutdown(iface: *mut IMFMediaEventQueue) -> HRESULT {
    let queue: *mut event_queue = impl_from_IMFMediaEventQueue(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*queue).cs));

    if (*queue).is_shut_down == 0 {
        event_queue_cleanup(queue);
        (*queue).is_shut_down = (TRUE as i32);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*queue).cs));

    WakeAllConditionVariable(core::ptr::addr_of_mut!((*queue).update_event));

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateEventQueue(queue: *mut *mut IMFMediaEventQueue) -> HRESULT {
    let mut object: *mut event_queue = core::ptr::null_mut::<event_queue>();
    if (*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"MFCreateEventQueue\0".as_ptr() as *const i8,
            7663 as i32,
            b"%p\n\0".as_ptr() as *const i8,
            queue,
        );
    }
    object = calloc((1 as size_t) as _, (core::mem::size_of::<event_queue>() as size_t) as _) as *mut event_queue;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).IMFMediaEventQueue_iface.lpVtbl = core::ptr::addr_of!(eventqueuevtbl);
    (*object).refcount = 1 as LONG;
    list_init(core::ptr::addr_of_mut!((*object).events));
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));
    InitializeConditionVariable(core::ptr::addr_of_mut!((*object).update_event));
    *queue = core::ptr::addr_of_mut!((*object).IMFMediaEventQueue_iface);
    return S_OK;
}

unsafe extern "C" fn impl_from_IMFCollection(iface: *mut IMFCollection) -> *mut collection {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(collection, IMFCollection_iface)) as *mut collection;
}

unsafe extern "C" fn collection_clear(collection: *mut collection) {
    let mut i: size_t = 0;
    while i < (*collection).count {
        if !(*(*collection).elements.add(i)).is_null() {
            IUnknown_Release(*(*collection).elements.add(i));
        }
        i += 1;
    }
    free((*collection).elements as *mut core::ffi::c_void);
    (*collection).elements = core::ptr::null_mut();
    (*collection).count = 0;
    (*collection).capacity = 0;
}

unsafe extern "system" fn collection_QueryInterface(iface: *mut IMFCollection, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFCollection)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = iface as *mut core::ffi::c_void;
        IMFCollection_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn collection_AddRef(iface: *mut IMFCollection) -> ULONG {
    let collection: *mut collection = impl_from_IMFCollection(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*collection).refcount)) as ULONG;

    return refcount;
}

unsafe extern "system" fn collection_Release(iface: *mut IMFCollection) -> ULONG {
    let collection: *mut collection = impl_from_IMFCollection(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*collection).refcount)) as ULONG;

    TRACE!("%p, %ld.\n", collection, refcount);

    if refcount == 0 {
        collection_clear(collection);
        free((*collection).elements as *mut core::ffi::c_void);
        free(collection as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn collection_GetElementCount(iface: *mut IMFCollection, count: *mut DWORD) -> HRESULT {
    let collection: *mut collection = impl_from_IMFCollection(iface);

    if count.is_null() {
        return E_POINTER;
    }

    *count = (*collection).count as DWORD;

    return S_OK;
}

unsafe extern "system" fn collection_GetElement(iface: *mut IMFCollection, idx: DWORD, element: *mut *mut IUnknown) -> HRESULT {
    let collection: *mut collection = impl_from_IMFCollection(iface);

    if element.is_null() {
        return E_POINTER;
    }

    if (idx as usize) >= (*collection).count {
        return E_INVALIDARG;
    }

    *element = *(*collection).elements.add(idx as usize);
    if !(*element).is_null() {
        IUnknown_AddRef(*element);
    }

    return if !(*element).is_null() {
        S_OK
    } else {
        E_UNEXPECTED
    };
}

unsafe extern "system" fn collection_AddElement(iface: *mut IMFCollection, element: *mut IUnknown) -> HRESULT {
    let collection: *mut collection = impl_from_IMFCollection(iface);

    if mf_array_reserve(
        &raw mut (*collection).elements as *mut *mut ::core::ffi::c_void,
        &raw mut (*collection).capacity,
        (*collection).count.wrapping_add(1 as usize),
        ::core::mem::size_of::<IUnknown>() as usize,
    ) == 0
    {
        return E_OUTOFMEMORY;
    }

    let fresh19 = (*collection).count;
    (*collection).count = (*collection).count.wrapping_add(1);
    let ref mut fresh20 = *(*collection).elements.offset(fresh19 as isize);
    *fresh20 = element;
    if !element.is_null() {
        IUnknown_AddRef(element);
    }

    return S_OK;
}

unsafe extern "system" fn collection_RemoveElement(iface: *mut IMFCollection, idx: DWORD, element: *mut *mut IUnknown) -> HRESULT {
    let collection: *mut collection = impl_from_IMFCollection(iface);
    let mut count: size_t = 0;

    if element.is_null() {
        return E_POINTER;
    }

    if idx as size_t >= (*collection).count {
        return E_INVALIDARG;
    }

    *element = *(*collection).elements.add(idx as usize);

    count = (*collection)
        .count
        .wrapping_sub(idx as size_t)
        .wrapping_sub(1 as size_t);
    if count != 0 {
        memmove(
            (*collection).elements.add(idx as usize) as *mut c_void,
            (*collection)
                .elements
                .add((idx as usize).wrapping_add(1 as usize)) as *const c_void,
            count.wrapping_mul(core::mem::size_of::<IUnknown>() as size_t),
        );
    }
    (*collection).count = (*collection).count.wrapping_sub(1);

    return S_OK;
}

unsafe extern "system" fn collection_InsertElementAt(mut iface: *mut IMFCollection, mut idx: DWORD, mut element: *mut IUnknown) -> HRESULT {
    let mut collection: *mut collection = impl_from_IMFCollection(iface);
    let mut i: size_t = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"collection_InsertElementAt\0" as *const u8 as *const ::core::ffi::c_char,
            7831 as ::core::ffi::c_int,
            b"%p, %lu, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            idx,
            element,
        );
    }
    if mf_array_reserve(
        &raw mut (*collection).elements as *mut *mut ::core::ffi::c_void,
        &raw mut (*collection).capacity,
        idx.wrapping_add(1 as DWORD) as size_t,
        ::core::mem::size_of::<IUnknown>() as size_t,
    ) == 0
    {
        return E_OUTOFMEMORY;
    }
    if (idx as size_t) < (*collection).count {
        memmove(
            (*collection)
                .elements
                .offset(idx.wrapping_add(1 as DWORD) as isize) as *mut *mut IUnknown
                as *mut ::core::ffi::c_void,
            (*collection).elements.offset(idx as isize) as *mut *mut IUnknown
                as *const ::core::ffi::c_void,
            (*collection)
                .count
                .wrapping_sub(idx as size_t)
                .wrapping_mul(::core::mem::size_of::<IUnknown>() as size_t),
        );
        (*collection).count = (*collection).count.wrapping_add(1);
    } else {
        i = (*collection).count;
        while i < idx as size_t {
            *(*collection).elements.offset(i as isize) = ::core::ptr::null_mut::<IUnknown>();
            i = i.wrapping_add(1);
        }
        (*collection).count = idx.wrapping_add(1 as DWORD) as size_t;
    }
    *(*collection).elements.offset(idx as isize) = element;
    if !(*(*collection).elements.offset(idx as isize)).is_null() {
        IUnknown_AddRef(*(*collection).elements.offset(idx as isize));
    }
    return S_OK;
}

unsafe extern "system" fn collection_RemoveAllElements(iface: *mut IMFCollection) -> HRESULT {
    let collection: *mut collection = impl_from_IMFCollection(iface);

    collection_clear(collection);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateCollection(collection: *mut *mut IMFCollection) -> HRESULT {
    let mut object: *mut collection = core::ptr::null_mut();

    if collection.is_null() {
        return E_POINTER;
    }

    object = calloc(1, (core::mem::size_of::<collection>()) as _) as *mut collection;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFCollection_iface.lpVtbl = core::ptr::addr_of!(mfcollectionvtbl);
    (*object).refcount = 1;

    *collection = core::ptr::addr_of_mut!((*object).IMFCollection_iface);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFHeapAlloc(size: SIZE_T, flags: ULONG, file: *mut c_char, line: c_int, type_: EAllocationType) -> *mut c_void {
    return HeapAlloc(GetProcessHeap(), flags as DWORD, size) as *mut c_void;
}

#[no_mangle]
pub unsafe extern "system" fn MFHeapFree(p: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, p);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMFByteStreamOnStreamEx(
    stream: *mut IUnknown,
    bytestream: *mut *mut IMFByteStream,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", stream, bytestream);
    E_NOTIMPL
}

unsafe extern "system" fn system_clock_QueryInterface(iface: *mut IMFClock, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClock))) != 0) || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0) {
        *obj = iface as *mut core::ffi::c_void;
        IMFClock_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn system_clock_AddRef(mut iface: *mut IMFClock) -> ULONG {
    let mut clock: *mut system_clock = impl_from_IMFClock(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*clock).refcount) as *mut i32,
    ) as ULONG;
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                .as_ptr() as *const i8,
            b"system_clock_AddRef\0".as_ptr() as *const i8,
            7954i32,
            b"%p, refcount %lu.\n\0".as_ptr() as *const i8,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn system_clock_Release(iface: *mut IMFClock) -> ULONG {
    let clock: *mut system_clock = impl_from_IMFClock(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*clock).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"system_clock_Release\0" as *const u8 as *const ::core::ffi::c_char,
            7964 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }

    if refcount == 0 {
        free(clock as *mut ::core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn system_clock_GetClockCharacteristics(
    mut iface: *mut IMFClock,
    mut flags: *mut DWORD,
) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"system_clock_GetClockCharacteristics\0" as *const u8 as *const ::core::ffi::c_char,
            7974 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            flags,
        );
    }
    *flags = (MFCLOCK_CHARACTERISTICS_FLAG_FREQUENCY_10MHZ as ::core::ffi::c_int
        | MFCLOCK_CHARACTERISTICS_FLAG_ALWAYS_RUNNING as ::core::ffi::c_int
        | MFCLOCK_CHARACTERISTICS_FLAG_IS_SYSTEM_CLOCK as ::core::ffi::c_int) as DWORD;
    return S_OK;
}
unsafe extern "system" fn system_clock_GetCorrelatedTime(iface: *mut IMFClock, reserved: DWORD, clock_time: *mut LONGLONG, system_time: *mut MFTIME) -> HRESULT {
    let time = MFGetSystemTime();
    *system_time = time;
    *clock_time = time;

    return S_OK;
}
unsafe extern "system" fn system_clock_GetContinuityKey(iface: *mut IMFClock, key: *mut DWORD) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"system_clock_GetContinuityKey\0" as *const u8 as *const ::core::ffi::c_char,
            7994 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            key,
        );
    }

    *key = 0 as DWORD;

    return S_OK;
}
unsafe extern "system" fn system_clock_GetState(iface: *mut IMFClock, reserved: DWORD, state: *mut MFCLOCK_STATE) -> HRESULT {
    *state = MFCLOCK_STATE_RUNNING;

    return S_OK;
}

unsafe extern "system" fn system_clock_GetProperties(iface: *mut IMFClock, props: *mut MFCLOCK_PROPERTIES) -> HRESULT {
    if props.is_null() {
        return E_POINTER;
    }

    memset(props as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<MFCLOCK_PROPERTIES>() as size_t);
    (*props).qwClockFrequency = MFCLOCK_FREQUENCY_HNS as UINT64;
    (*props).dwClockTolerance = MFCLOCK_TOLERANCE_UNKNOWN as DWORD;
    (*props).dwClockJitter = 1 as DWORD;

    return S_OK;
}

unsafe extern "C" fn create_system_clock(clock: *mut *mut IMFClock) -> HRESULT {
    let mut object: *mut system_clock = core::ptr::null_mut();

    object = malloc(core::mem::size_of::<system_clock>()) as *mut system_clock;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFClock_iface.lpVtbl = core::ptr::addr_of!(system_clock_vtbl);
    (*object).refcount = 1;

    *clock = core::ptr::addr_of_mut!((*object).IMFClock_iface);

    return S_OK;
}

unsafe extern "system" fn system_time_source_QueryInterface(iface: *mut IMFPresentationTimeSource, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFPresentationTimeSource(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFPresentationTimeSource) as *const IID) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown) as *const IID) != 0
    {
        *obj = core::ptr::addr_of_mut!((*source).IMFPresentationTimeSource_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClockStateSink) as *const IID) != 0
    {
        *obj = core::ptr::addr_of_mut!((*source).IMFClockStateSink_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn system_time_source_AddRef(iface: *mut IMFPresentationTimeSource) -> ULONG {
    let source: *mut system_time_source = impl_from_IMFPresentationTimeSource(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*source).refcount) as *mut i32) as ULONG;

    return refcount;
}

unsafe extern "system" fn system_time_source_Release(iface: *mut IMFPresentationTimeSource) -> ULONG {
    let source: *mut system_time_source = impl_from_IMFPresentationTimeSource(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*source).refcount) as *mut i32,
    ) as ULONG;

    if refcount == 0 {
        if !(*source).clock.is_null() {
            IMFClock_Release((*source).clock);
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*source).cs));
        free(source as *mut core::ffi::c_void);
    }

    return refcount;
}
unsafe extern "system" fn system_time_source_GetClockCharacteristics(iface: *mut IMFPresentationTimeSource, flags: *mut DWORD) -> HRESULT {
    *flags = MFCLOCK_CHARACTERISTICS_FLAG_FREQUENCY_10MHZ | MFCLOCK_CHARACTERISTICS_FLAG_IS_SYSTEM_CLOCK;

    return S_OK;
}

unsafe extern "system" fn system_time_source_GetCorrelatedTime(iface: *mut IMFPresentationTimeSource, reserved: DWORD, clock_time: *mut LONGLONG, system_time: *mut MFTIME) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFPresentationTimeSource(iface);
    let mut hr: HRESULT = 0;

    TRACE!("%p, %#lx, %p, %p.\n", iface, reserved, clock_time, system_time);

    EnterCriticalSection(core::ptr::addr_of_mut!((*source).cs));
    hr = IMFClock_GetCorrelatedTime((*source).clock, 0, clock_time, system_time);
    if hr >= 0 {
        if (*source).state == MFCLOCK_STATE_RUNNING {
            system_time_source_update_clock_time(source, *system_time);
        }
        *clock_time = (*source).start_offset + (*source).clock_time;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*source).cs));

    return hr;
}

unsafe extern "system" fn system_time_source_GetContinuityKey(iface: *mut IMFPresentationTimeSource, key: *mut DWORD) -> HRESULT {
    TRACE!("%p, %p.\n", iface, key);

    *key = 0;

    return S_OK;
}

unsafe extern "system" fn system_time_source_GetState(
    iface: *mut IMFPresentationTimeSource,
    reserved: DWORD,
    state: *mut MFCLOCK_STATE,
) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFPresentationTimeSource(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*source).cs));
    *state = (*source).state;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*source).cs));

    return S_OK;
}
unsafe extern "system" fn system_time_source_GetProperties(mut iface: *mut IMFPresentationTimeSource, mut props: *mut MFCLOCK_PROPERTIES) -> HRESULT {
    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const core::ffi::c_char,
            b"system_time_source_GetProperties\0" as *const u8 as *const core::ffi::c_char,
            8160 as core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            props,
        );
    }

    if props.is_null() {
        return E_POINTER;
    }

    memset(
        props as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<MFCLOCK_PROPERTIES>() as size_t,
    );
    (*props).qwClockFrequency = MFCLOCK_FREQUENCY_HNS as UINT64;
    (*props).dwClockTolerance = MFCLOCK_TOLERANCE_UNKNOWN as DWORD;
    (*props).dwClockJitter = 1 as DWORD;

    return S_OK;
}

unsafe extern "system" fn system_time_source_GetUnderlyingClock(iface: *mut IMFPresentationTimeSource, clock: *mut *mut IMFClock) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFPresentationTimeSource(iface);
    *clock = (*source).clock;
    IMFClock_AddRef(*clock);
    return S_OK;
}

unsafe extern "system" fn system_time_source_sink_QueryInterface(iface: *mut IMFClockStateSink, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    return IMFPresentationTimeSource_QueryInterface(core::ptr::addr_of_mut!((*source).IMFPresentationTimeSource_iface), riid, out);
}

unsafe extern "system" fn system_time_source_sink_AddRef(iface: *mut IMFClockStateSink) -> ULONG {
    let source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    return IMFPresentationTimeSource_AddRef(core::ptr::addr_of_mut!((*source).IMFPresentationTimeSource_iface));
}

unsafe extern "system" fn system_time_source_sink_Release(iface: *mut IMFClockStateSink) -> ULONG {
    let source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    return IMFPresentationTimeSource_Release(core::ptr::addr_of_mut!((*source).IMFPresentationTimeSource_iface));
}

unsafe extern "C" fn system_time_source_change_state(source: *mut system_time_source, command: clock_command) -> HRESULT {
    static state_change_is_allowed: [[BYTE; (CLOCK_CMD_MAX as _)]; (MFCLOCK_STATE_PAUSED+1) as _] = [
        [1, 0, 1, 0],
        [1, 1, 1, 0],
        [1, 1, 0, 0],
        [1, 1, 0, 1],
    ];
    static states: [MFCLOCK_STATE; (CLOCK_CMD_MAX as _)] = [
        MFCLOCK_STATE_RUNNING,
        MFCLOCK_STATE_STOPPED,
        MFCLOCK_STATE_PAUSED,
        MFCLOCK_STATE_RUNNING,
    ];

    /* Special case that go against usual state change vs return value behavior. */
    if (*source).state as u32 == MFCLOCK_STATE_INVALID as u32 && command as u32 == CLOCK_CMD_STOP as u32 {
        return S_OK;
    }

    if state_change_is_allowed[(*source).state as usize][command as usize] == 0 {
        return MF_E_INVALIDREQUEST;
    }

    (*source).state = states[command as usize];

    return S_OK;
}

unsafe extern "system" fn system_time_source_sink_OnClockStart(iface: *mut IMFClockStateSink, system_time: MFTIME, start_offset: LONGLONG) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    let mut state: MFCLOCK_STATE;
    let mut hr: HRESULT;

    TRACE!("%p, %s, %s.\n", iface, debugstr_time(system_time as LONGLONG), debugstr_time(start_offset));

    EnterCriticalSection(core::ptr::addr_of_mut!((*source).cs));
    state = (*source).state;
    hr = system_time_source_change_state(source, (CLOCK_CMD_START) as clock_command);
    if hr >= 0 {
        if start_offset == (PRESENTATION_CURRENT_POSITION as _) {
            if state != MFCLOCK_STATE_RUNNING {
                (*source).start_offset -= system_time as LONGLONG;
                (*source).system_time = 0;
            }
        } else {
            (*source).start_offset = start_offset;
            (*source).system_time = system_time as LONGLONG;
            (*source).clock_time = 0;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*source).cs));

    return hr;
}

unsafe extern "system" fn system_time_source_sink_OnClockStop(iface: *mut IMFClockStateSink, system_time: MFTIME) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT;
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"system_time_source_sink_OnClockStop\0" as *const u8 as *const i8,
            8292i32,
            b"%p, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_time(system_time as LONGLONG),
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*source).cs));
    hr = system_time_source_change_state(source, (CLOCK_CMD_STOP) as clock_command);
    if hr >= 0 {
        (*source).clock_time = 0;
        (*source).system_time = (*source).clock_time;
        (*source).start_offset = (*source).system_time;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*source).cs));

    return hr;
}

unsafe extern "system" fn system_time_source_sink_OnClockPause(iface: *mut IMFClockStateSink, system_time: MFTIME) -> HRESULT {
    let source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT;

    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"system_time_source_sink_OnClockPause\0" as *const u8 as *const i8,
            8309 as i32,
            b"%p, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_time(system_time as LONGLONG),
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*source).cs));
    hr = system_time_source_change_state(source, (CLOCK_CMD_PAUSE) as clock_command);
    if hr >= 0 {
        system_time_source_update_clock_time(source, system_time as LONGLONG);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*source).cs));

    return hr;
}

unsafe extern "system" fn system_time_source_sink_OnClockRestart(iface: *mut IMFClockStateSink, system_time: MFTIME) -> HRESULT {
    let mut source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT = 0;

    TRACE!("%p, %s.\n", iface, debugstr_time(system_time as LONGLONG));

    EnterCriticalSection(core::ptr::addr_of_mut!((*source).cs));
    hr = system_time_source_change_state(source, (CLOCK_CMD_RESTART) as clock_command);
    if hr >= 0 {
        (*source).system_time = system_time as LONGLONG;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*source).cs));

    return hr;
}

unsafe extern "system" fn system_time_source_sink_OnClockSetRate(iface: *mut IMFClockStateSink, system_time: MFTIME, rate: f32) -> HRESULT {
    let mut source: *mut system_time_source = impl_from_IMFClockStateSink(iface);
    let mut intpart: f64 = 0.0;

    TRACE!("%p, %s, %f.\n", iface, debugstr_time(system_time as LONGLONG), rate as f64);

    if rate == 0.0f32 {
        return MF_E_UNSUPPORTED_RATE;
    }

    modf(rate as f64, core::ptr::addr_of_mut!(intpart));

    EnterCriticalSection(core::ptr::addr_of_mut!((*source).cs));
    (*source).rate = rate;
    (*source).i_rate = (if (rate as f64) == intpart { rate } else { 0 as f32 }) as i32;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*source).cs));

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSystemTimeSource(
    time_source: *mut *mut IMFPresentationTimeSource,
) -> HRESULT {
    let mut object: *mut system_time_source = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!("%p.\n", time_source);

    object = calloc(1, (core::mem::size_of::<system_time_source>()) as _) as *mut system_time_source;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFPresentationTimeSource_iface.lpVtbl = core::ptr::addr_of!(systemtimesourcevtbl);
    (*object).IMFClockStateSink_iface.lpVtbl = core::ptr::addr_of!(systemtimesourcesinkvtbl);
    (*object).refcount = 1;
    (*object).rate = 1.0f32;
    (*object).i_rate = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    hr = create_system_clock(core::ptr::addr_of_mut!((*object).clock));
    if hr < 0 {
        IMFPresentationTimeSource_Release(core::ptr::addr_of_mut!((*object).IMFPresentationTimeSource_iface));
        return hr;
    }

    *time_source = core::ptr::addr_of_mut!((*object).IMFPresentationTimeSource_iface);

    return S_OK;
}

unsafe extern "C" fn impl_from_create_file_IRtwqAsyncCallback(
    iface: *mut IRtwqAsyncCallback,
) -> *mut async_create_file {
    return (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(async_create_file, IRtwqAsyncCallback_iface) as ULONG_PTR,
    ) as *mut async_create_file;
}

unsafe extern "system" fn async_create_file_callback_QueryInterface(iface: *mut IRtwqAsyncCallback, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IRtwqAsyncCallback)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IRtwqAsyncCallback_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn async_create_file_callback_AddRef(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let async_: *mut async_create_file = impl_from_create_file_IRtwqAsyncCallback(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*async_).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"async_create_file_callback_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            8446 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn async_create_file_callback_Release(iface: *mut IRtwqAsyncCallback) -> ULONG {
    let async_0: *mut async_create_file = impl_from_create_file_IRtwqAsyncCallback(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*async_0).refcount) as *mut core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"async_create_file_callback_Release\0" as *const u8 as *const core::ffi::c_char,
            8456 as core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        free((*async_0).path as *mut core::ffi::c_void);
        free(async_0 as *mut core::ffi::c_void);
    }
    return refcount;
}
unsafe extern "system" fn async_create_file_callback_GetParameters(
    iface: *mut IRtwqAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn async_create_file_callback_Invoke(iface: *mut IRtwqAsyncCallback, result: *mut IRtwqAsyncResult) -> HRESULT {
    let async_0: *mut async_create_file = impl_from_create_file_IRtwqAsyncCallback(iface);
    let mut caller: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut stream: *mut IMFByteStream = core::ptr::null_mut();
    let mut hr: HRESULT;

    caller = IRtwqAsyncResult_GetStateNoAddRef(result) as *mut IRtwqAsyncResult;

    hr = MFCreateFile(
        (*async_0).access_mode,
        (*async_0).open_mode,
        (*async_0).flags,
        (*async_0).path as *const WCHAR,
        core::ptr::addr_of_mut!(stream),
    );
    if hr >= 0 {
        let mut result_item: *mut async_create_file_result = core::ptr::null_mut();

        result_item = malloc(core::mem::size_of::<async_create_file_result>() as size_t)
            as *mut async_create_file_result;
        if !result_item.is_null() {
            (*result_item).result = caller;
            IRtwqAsyncResult_AddRef(caller);
            (*result_item).stream = stream;
            IMFByteStream_AddRef(stream);

            EnterCriticalSection(core::ptr::addr_of_mut!(async_create_file_cs));
            list_add_tail(
                core::ptr::addr_of_mut!(async_create_file_results),
                core::ptr::addr_of_mut!((*result_item).entry),
            );
            LeaveCriticalSection(core::ptr::addr_of_mut!(async_create_file_cs));
        }

        IMFByteStream_Release(stream);
    } else {
        IRtwqAsyncResult_SetStatus(caller, hr);
    }

    RtwqInvokeCallback(caller);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFBeginCreateFile(
    access_mode: MF_FILE_ACCESSMODE,
    open_mode: MF_FILE_OPENMODE,
    flags: MF_FILE_FLAGS,
    path: *const WCHAR,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
    cancel_cookie: *mut *mut IUnknown,
) -> HRESULT {
    let mut async_: *mut async_create_file = core::ptr::null_mut();
    let mut caller: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut item: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if !cancel_cookie.is_null() {
        *cancel_cookie = core::ptr::null_mut::<IUnknown>();
    }

    hr = RtwqCreateAsyncResult(
        core::ptr::null_mut::<IUnknown>(),
        callback as *mut IRtwqAsyncCallback,
        state,
        &raw mut caller,
    );
    if hr < 0 {
        return hr;
    }

    'failed: {
        async_ = malloc(core::mem::size_of::<async_create_file>() as size_t)
            as *mut async_create_file;
        if async_.is_null() {
            hr = E_OUTOFMEMORY;
            break 'failed;
        }

        (*async_).IRtwqAsyncCallback_iface.lpVtbl = &raw const async_create_file_callback_vtbl;
        (*async_).refcount = 1;
        (*async_).access_mode = access_mode;
        (*async_).open_mode = open_mode;
        (*async_).flags = flags;
        (*async_).path = wcsdup(path as *const wchar_t) as *mut WCHAR;
        if (*async_).path.is_null() {
            hr = E_OUTOFMEMORY;
            break 'failed;
        }

        hr = RtwqCreateAsyncResult(
            core::ptr::null_mut::<IUnknown>(),
            &raw mut (*async_).IRtwqAsyncCallback_iface,
            caller as *mut IUnknown,
            &raw mut item,
        );
        if hr < 0 {
            break 'failed;
        }

        if !cancel_cookie.is_null() {
            *cancel_cookie = caller as *mut IUnknown;
            IUnknown_AddRef(*cancel_cookie);
        }

        hr = RtwqInvokeCallback(item);
    }

    if !async_.is_null() {
        IRtwqAsyncCallback_Release(&raw mut (*async_).IRtwqAsyncCallback_iface);
    }
    if !item.is_null() {
        IRtwqAsyncResult_Release(item);
    }
    if !caller.is_null() {
        IRtwqAsyncResult_Release(caller);
    }

    return hr;
}

unsafe extern "C" fn async_create_file_pull_result(
    mut unk: *mut IUnknown,
    mut stream: *mut *mut IMFByteStream,
) -> HRESULT {
    let mut item: *mut async_create_file_result =
        ::core::ptr::null_mut::<async_create_file_result>();
    let mut hr: HRESULT = MF_E_UNEXPECTED;
    let mut result: *mut IRtwqAsyncResult = ::core::ptr::null_mut::<IRtwqAsyncResult>();
    *stream = ::core::ptr::null_mut::<IMFByteStream>();
    if IUnknown_QueryInterface(
        unk,
        &raw const IID_IRtwqAsyncResult,
        &raw mut result as *mut *mut ::core::ffi::c_void,
    ) < 0
    {
        return hr;
    }
    EnterCriticalSection(&raw mut async_create_file_cs);
    item = (async_create_file_results.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(async_create_file_result, entry)
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut async_create_file_result;
    while &raw mut (*item).entry != &raw mut async_create_file_results {
        if result == (*item).result {
            *stream = (*item).stream;
            IRtwqAsyncResult_Release((*item).result);
            list_remove(&raw mut (*item).entry);
            free(item as *mut ::core::ffi::c_void);
            break;
        } else {
            item = ((*item).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(async_create_file_result, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut async_create_file_result;
        }
    }
    LeaveCriticalSection(&raw mut async_create_file_cs);
    if !(*stream).is_null() {
        hr = IRtwqAsyncResult_GetStatus(result);
    }
    IRtwqAsyncResult_Release(result);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFEndCreateFile(result: *mut IMFAsyncResult, stream: *mut *mut IMFByteStream) -> HRESULT {
    return async_create_file_pull_result(result as *mut IUnknown, stream);
}

#[no_mangle]
pub unsafe extern "system" fn MFCancelCreateFile(cancel_cookie: *mut IUnknown) -> HRESULT {
    let mut stream: *mut IMFByteStream = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = async_create_file_pull_result(cancel_cookie, core::ptr::addr_of_mut!(stream));

    if !stream.is_null() {
        IMFByteStream_Release(stream);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFRegisterLocalSchemeHandler(scheme: *const WCHAR, activate: *mut IMFActivate) -> HRESULT {
    let mut handler: *mut local_handler = core::ptr::null_mut();

    if scheme.is_null() || activate.is_null() {
        return E_INVALIDARG;
    }

    handler = malloc(core::mem::size_of::<local_handler>() as size_t) as *mut local_handler;
    if handler.is_null() {
        return E_OUTOFMEMORY;
    }

    (*handler).u.scheme = wcsdup(scheme as *const wchar_t) as *mut WCHAR;
    if (*handler).u.scheme.is_null() {
        free(handler as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }
    (*handler).activate = activate;
    IMFActivate_AddRef((*handler).activate);

    EnterCriticalSection(core::ptr::addr_of_mut!(local_handlers_section));
    list_add_head(core::ptr::addr_of_mut!(local_scheme_handlers), core::ptr::addr_of_mut!((*handler).entry));
    LeaveCriticalSection(core::ptr::addr_of_mut!(local_handlers_section));

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFRegisterLocalByteStreamHandler(extension: *const WCHAR, mime: *const WCHAR, activate: *mut IMFActivate) -> HRESULT {
    let mut handler: *mut local_handler;

    if (extension.is_null() && mime.is_null()) || activate.is_null() {
        return E_INVALIDARG;
    }

    handler = calloc(1, (core::mem::size_of::<local_handler>()) as _) as *mut local_handler;
    if handler.is_null() {
        return E_OUTOFMEMORY;
    }

    'failed: {
        if !extension.is_null() {
            (*handler).u.bytestream.extension = wcsdup(extension as *const wchar_t) as *mut WCHAR;
            if (*handler).u.bytestream.extension.is_null() {
                break 'failed;
            }
        }
        if !mime.is_null() {
            (*handler).u.bytestream.mime = wcsdup(mime as *const wchar_t) as *mut WCHAR;
            if (*handler).u.bytestream.mime.is_null() {
                break 'failed;
            }
        }
        (*handler).activate = activate;
        IMFActivate_AddRef((*handler).activate);

        EnterCriticalSection(core::ptr::addr_of_mut!(local_handlers_section));
        list_add_head(core::ptr::addr_of_mut!(local_bytestream_handlers), core::ptr::addr_of_mut!((*handler).entry));
        LeaveCriticalSection(core::ptr::addr_of_mut!(local_handlers_section));

        return S_OK;
    }

    free((*handler).u.bytestream.extension as *mut core::ffi::c_void);
    free((*handler).u.bytestream.mime as *mut core::ffi::c_void);
    free(handler as *mut core::ffi::c_void);

    return E_OUTOFMEMORY;
}

unsafe extern "C" fn impl_from_IPropertyStore(iface: *mut IPropertyStore) -> *mut property_store {
    return CONTAINING_RECORD!(iface, property_store, IPropertyStore_iface);
}

unsafe extern "system" fn property_store_QueryInterface(iface: *mut IPropertyStore, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IPropertyStore)) != 0 || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IPropertyStore_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}

unsafe extern "system" fn property_store_AddRef(iface: *mut IPropertyStore) -> ULONG {
    let store: *mut property_store = impl_from_IPropertyStore(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*store).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"property_store_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            8745 as ::core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}
unsafe extern "system" fn property_store_Release(iface: *mut IPropertyStore) -> ULONG {
    let store: *mut property_store = impl_from_IPropertyStore(iface);
    let refcount: ULONG = InterlockedDecrement(
        &raw mut (*store).refcount as *mut core::ffi::c_long,
    ) as ULONG;

    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"property_store_Release\0" as *const u8 as *const core::ffi::c_char,
            8755 as core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }

    if refcount == 0 {
        DeleteCriticalSection(&raw mut (*store).cs);
        free((*store).values as *mut core::ffi::c_void);
        free(store as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn property_store_GetCount(iface: *mut IPropertyStore, count: *mut DWORD) -> HRESULT {
    let store: *mut property_store = impl_from_IPropertyStore(iface);
    TRACE!("%p, %p.\n", iface, count);
    if count.is_null() {
        return E_INVALIDARG;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    *count = (*store).count as DWORD;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    return S_OK;
}

unsafe extern "system" fn property_store_GetAt(iface: *mut IPropertyStore, index: DWORD, key: *mut PROPERTYKEY) -> HRESULT {
    let store: *mut property_store = impl_from_IPropertyStore(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));

    if index as usize >= (*store).count {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
        return E_INVALIDARG;
    }

    *key = (*(*store).values.offset(index as isize)).key;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    return S_OK;
}

unsafe extern "system" fn property_store_GetValue(iface: *mut IPropertyStore, key: *const PROPERTYKEY, value: *mut PROPVARIANT) -> HRESULT {
    let store: *mut property_store = impl_from_IPropertyStore(iface);
    let mut i: u32;
    TRACE!("%p, %p, %p.\n", iface, key, value);
    if value.is_null() {
        return E_INVALIDARG;
    }
    if key.is_null() {
        return S_FALSE;
    }
    EnterCriticalSection(&raw mut (*store).cs);
    i = 0 as u32;
    while (i as usize) < (*store).count {
        if memcmp(
            key as *const core::ffi::c_void,
            &raw mut (*(*store).values.add(i as usize)).key as *const core::ffi::c_void,
            core::mem::size_of::<PROPERTYKEY>() as usize,
        ) == 0
        {
            PropVariantCopy(value, &raw mut (*(*store).values.add(i as usize)).value);
            LeaveCriticalSection(&raw mut (*store).cs);
            return S_OK;
        }
        i = i.wrapping_add(1);
    }
    LeaveCriticalSection(&raw mut (*store).cs);
    return S_FALSE;
}

unsafe extern "system" fn property_store_SetValue(iface: *mut IPropertyStore, key: *const PROPERTYKEY, value: *const PROPVARIANT) -> HRESULT {
    let store: *mut property_store = impl_from_IPropertyStore(iface);
    let mut i: u32 = 0;
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0" as *const u8 as *const i8,
            b"property_store_SetValue\0" as *const u8 as *const i8,
            8836 as i32,
            b"%p, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            key,
            value,
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    i = 0;
    while (i as usize) < (*store).count {
        if memcmp(
            key as *const core::ffi::c_void,
            core::ptr::addr_of_mut!((*(*store).values.offset(i as isize)).key) as *const core::ffi::c_void,
            core::mem::size_of::<PROPERTYKEY>() as usize,
        ) == 0
        {
            PropVariantCopy(core::ptr::addr_of_mut!((*(*store).values.offset(i as isize)).value), value);
            LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
            return S_OK;
        }
        i = i.wrapping_add(1);
    }
    if mf_array_reserve(
        core::ptr::addr_of_mut!((*store).values) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*store).capacity),
        (*store).count.wrapping_add(1 as usize),
        core::mem::size_of_val(&*(*store).values) as usize,
    ) == 0
    {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
        return E_OUTOFMEMORY;
    }
    (*(*store).values.offset((*store).count as isize)).key = *key;
    PropVariantCopy(
        core::ptr::addr_of_mut!((*(*store).values.offset((*store).count as isize)).value),
        value,
    );
    (*store).count = (*store).count.wrapping_add(1);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    return S_OK;
}
unsafe extern "system" fn property_store_Commit(iface: *mut IPropertyStore) -> HRESULT {
    // TRACE("%p.\n", iface);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn CreatePropertyStore(store: *mut *mut IPropertyStore) -> HRESULT {
    let mut object: *mut property_store = core::ptr::null_mut::<property_store>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CreatePropertyStore\0" as *const u8 as *const ::core::ffi::c_char,
            8890 as ::core::ffi::c_int,
            b"%p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            store,
        );
    }
    if store.is_null() {
        return E_INVALIDARG;
    }
    object = calloc(
        1,
        (core::mem::size_of::<property_store>()) as _,
    ) as *mut property_store;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).IPropertyStore_iface.lpVtbl = core::ptr::addr_of!(property_store_vtbl);
    (*object).refcount = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl_0) as u32,
            __dbch_0,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CreatePropertyStore\0" as *const u8 as *const ::core::ffi::c_char,
            8902 as ::core::ffi::c_int,
            b"Created store %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            object,
        );
    }
    *store = core::ptr::addr_of_mut!((*object).IPropertyStore_iface);
    return S_OK;
}

unsafe extern "C" fn impl_from_IMFD3D12SynchronizationObject(
    iface: *mut IMFD3D12SynchronizationObject,
) -> *mut d3d12_sync_object {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(d3d12_sync_object, IMFD3D12SynchronizationObject_iface),
    ) as *mut d3d12_sync_object;
}

unsafe extern "C" fn impl_from_IMFD3D12SynchronizationObjectCommands(
    iface: *mut IMFD3D12SynchronizationObjectCommands,
) -> *mut d3d12_sync_object {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(d3d12_sync_object, IMFD3D12SynchronizationObjectCommands_iface),
    ) as *mut d3d12_sync_object;
}

unsafe extern "C" fn d3d12_sync_object_from_IRtwqAsyncCallback(
    iface: *mut IRtwqAsyncCallback,
) -> *mut d3d12_sync_object {
    return (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(d3d12_sync_object, async_release_iface) as ULONG_PTR,
    ) as *mut d3d12_sync_object;
}

unsafe extern "system" fn d3d12_sync_object_QueryInterface(iface: *mut IMFD3D12SynchronizationObject, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObject(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFD3D12SynchronizationObject)) != 0 || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = core::ptr::addr_of_mut!((*syncobj).IMFD3D12SynchronizationObject_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFD3D12SynchronizationObjectCommands)) != 0 {
        *obj = core::ptr::addr_of_mut!((*syncobj).IMFD3D12SynchronizationObjectCommands_iface) as *mut core::ffi::c_void;
    } else {
        *obj = core::ptr::null_mut();
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        return E_NOINTERFACE;
    }

    IMFD3D12SynchronizationObject_AddRef(core::ptr::addr_of_mut!((*syncobj).IMFD3D12SynchronizationObject_iface));
    return S_OK;
}

unsafe extern "system" fn d3d12_sync_object_AddRef(iface: *mut IMFD3D12SynchronizationObject) -> ULONG {
    let syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObject(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*syncobj).refcount) as *mut LONG) as ULONG;

    TRACE!("%p, refcount %ld.\n", iface, refcount);

    return refcount;
}
unsafe extern "system" fn d3d12_sync_object_Release(
    mut iface: *mut IMFD3D12SynchronizationObject,
) -> ULONG {
    let mut syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObject(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*syncobj).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"d3d12_sync_object_Release\0" as *const u8 as *const ::core::ffi::c_char,
            8982 as ::core::ffi::c_int,
            b"%p, refcount %ld.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        let mut cursor: *mut d3d12_sync_object_release =
            core::ptr::null_mut::<d3d12_sync_object_release>();
        let mut cursor2: *mut d3d12_sync_object_release =
            core::ptr::null_mut::<d3d12_sync_object_release>();
        cursor = ((*syncobj).release_freelist.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(d3d12_sync_object_release, entry) as isize),
        ) as *mut d3d12_sync_object_release;
        cursor2 = ((*cursor).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(d3d12_sync_object_release, entry) as isize),
        ) as *mut d3d12_sync_object_release;
        while core::ptr::addr_of_mut!((*cursor).entry)
            != core::ptr::addr_of_mut!((*syncobj).release_freelist)
        {
            if !(*cursor).fence.is_null() {
                ID3D12Fence_Release((*cursor).fence);
            }
            CloseHandle((*cursor).event);
            free(cursor as *mut ::core::ffi::c_void);
            cursor = cursor2;
            cursor2 = ((*cursor).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(d3d12_sync_object_release, entry) as isize),
            ) as *mut d3d12_sync_object_release;
        }
        ID3D12Device_Release((*syncobj).device);
        ID3D12Fence_Release((*syncobj).ready_fence);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
        free(syncobj as *mut ::core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "system" fn d3d12_sync_object_SignalEventOnFinalResourceRelease(iface: *mut IMFD3D12SynchronizationObject, event: HANDLE) -> HRESULT {
    let mut syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObject(iface);

    TRACE!("%p, %p.\n", iface, event);

    if event.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    if (*syncobj).release_wait != 0 {
        (*syncobj).final_release_event = event;
    } else {
        SetEvent(event);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));

    return S_OK;
}

unsafe extern "system" fn d3d12_sync_object_Reset(iface: *mut IMFD3D12SynchronizationObject) -> HRESULT {
    let syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObject(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p.\n", iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    if (*syncobj).release_wait != 0 {
        hr = MF_E_UNEXPECTED;
    } else {
        (*syncobj).generation = (*syncobj).generation.wrapping_add(1);
        (*syncobj).release_freelist_cursor = core::ptr::addr_of_mut!((*syncobj).release_freelist);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));

    return hr;
}

unsafe extern "system" fn d3d12_sync_object_commands_QueryInterface(iface: *mut IMFD3D12SynchronizationObjectCommands, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    return d3d12_sync_object_QueryInterface(core::ptr::addr_of_mut!((*impl_from_IMFD3D12SynchronizationObjectCommands(iface)).IMFD3D12SynchronizationObject_iface), riid, obj);
}
unsafe extern "system" fn d3d12_sync_object_commands_AddRef(iface: *mut IMFD3D12SynchronizationObjectCommands) -> ULONG {
    return d3d12_sync_object_AddRef(core::ptr::addr_of_mut!((*impl_from_IMFD3D12SynchronizationObjectCommands(iface)).IMFD3D12SynchronizationObject_iface));
}
unsafe extern "system" fn d3d12_sync_object_commands_Release(iface: *mut IMFD3D12SynchronizationObjectCommands) -> ULONG {
    return d3d12_sync_object_Release(core::ptr::addr_of_mut!((*impl_from_IMFD3D12SynchronizationObjectCommands(iface)).IMFD3D12SynchronizationObject_iface));
}

unsafe extern "system" fn d3d12_sync_object_commands_EnqueueResourceReady(iface: *mut IMFD3D12SynchronizationObjectCommands, producer_queue: *mut ID3D12CommandQueue) -> HRESULT {
    let syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObjectCommands(iface);
    let mut hr: HRESULT;

    TRACE!("%p, %p.\n", iface, producer_queue);

    if producer_queue.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    hr = ID3D12CommandQueue_Signal(producer_queue, (*syncobj).ready_fence, (*syncobj).generation);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));

    return hr;
}

unsafe extern "system" fn d3d12_sync_object_commands_EnqueueResourceReadyWait(iface: *mut IMFD3D12SynchronizationObjectCommands, consumer_queue: *mut ID3D12CommandQueue) -> HRESULT {
    let mut syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObjectCommands(iface);
    let mut hr: HRESULT;

    if consumer_queue.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(&raw mut (*syncobj).cs);
    hr = ID3D12CommandQueue_Wait(consumer_queue, (*syncobj).ready_fence, (*syncobj).generation);
    LeaveCriticalSection(&raw mut (*syncobj).cs);

    return hr;
}

unsafe extern "system" fn d3d12_sync_object_commands_SignalEventOnResourceReady(
    iface: *mut IMFD3D12SynchronizationObjectCommands,
    event: HANDLE,
) -> HRESULT {
    let syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObjectCommands(iface);
    let mut hr: HRESULT;
    let mut generation: UINT64;

    EnterCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    generation = (*syncobj).generation;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));

    hr = ID3D12Fence_SetEventOnCompletion((*syncobj).ready_fence, generation, event);
    return hr;
}

unsafe extern "C" fn d3d12_sync_object_create_release(
    mut syncobj: *mut d3d12_sync_object,
    mut out: *mut *mut d3d12_sync_object_release,
) -> HRESULT {
    let mut next: *mut list = ::core::ptr::null_mut::<list>();
    let mut relobj: *mut d3d12_sync_object_release =
        ::core::ptr::null_mut::<d3d12_sync_object_release>();
    let mut hr: HRESULT = 0;
    next = list_next(
        &raw mut (*syncobj).release_freelist,
        (*syncobj).release_freelist_cursor,
    );
    if !next.is_null() {
        relobj = (next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(d3d12_sync_object_release, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut d3d12_sync_object_release;
    } else {
        relobj = calloc(
            (1 as size_t) as _,
            (::core::mem::size_of::<d3d12_sync_object_release>() as size_t) as _,
        ) as *mut d3d12_sync_object_release;
        if relobj.is_null() {
            return E_OUTOFMEMORY;
        }
        (*relobj).event = CreateEventA(
            ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            (FALSE) as BOOL,
            (FALSE) as BOOL,
            ::core::ptr::null::<CHAR>(),
        );
        list_add_after((*syncobj).release_freelist_cursor, &raw mut (*relobj).entry);
    }
    if (*relobj).fence.is_null() {
        hr = ID3D12Device_CreateFence(
            (*syncobj).device,
            0 as UINT64,
            D3D12_FENCE_FLAG_SHARED,
            &raw const IID_ID3D12Fence,
            &raw mut (*relobj).fence as *mut *mut ::core::ffi::c_void,
        );
        if hr < 0 as HRESULT {
            return hr;
        }
    }
    *out = relobj;
    return S_OK;
}

unsafe extern "system" fn d3d12_sync_object_commands_EnqueueResourceRelease(iface: *mut IMFD3D12SynchronizationObjectCommands, consumer_queue: *mut ID3D12CommandQueue) -> HRESULT {
    let syncobj: *mut d3d12_sync_object = impl_from_IMFD3D12SynchronizationObjectCommands(iface);
    let mut relobj: *mut d3d12_sync_object_release = core::ptr::null_mut();
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut key: RTWQWORKITEM_KEY = 0;
    let mut hr: HRESULT = 0;

    // TRACE("%p, %p.\n", iface, consumer_queue);
    TRACE!("%p, %p.\n", iface, consumer_queue);

    if consumer_queue.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    hr = d3d12_sync_object_create_release(syncobj, core::ptr::addr_of_mut!(relobj));
    'end: {
        if hr < 0 {
            break 'end;
        }

        hr = RtwqCreateAsyncResult(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*syncobj).async_release_iface),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(result),
        );
        if hr < 0 {
            break 'end;
        }

        hr = RtwqPutWaitingWorkItem((*relobj).event, 0, result, core::ptr::addr_of_mut!(key));
        IRtwqAsyncResult_Release(result);
        if hr < 0 {
            break 'end;
        }

        hr = ID3D12CommandQueue_Signal(consumer_queue, (*relobj).fence, (*syncobj).generation);
        if hr < 0 {
            RtwqCancelWorkItem(key);
            break 'end;
        }

        hr = ID3D12Fence_SetEventOnCompletion((*relobj).fence, (*syncobj).generation, (*relobj).event);
        if hr < 0 {
            RtwqCancelWorkItem(key);
            /* can't re-use fence since queue might signal it */
            ID3D12Fence_Release((*relobj).fence);
            (*relobj).fence = core::ptr::null_mut();
            break 'end;
        }

        (*syncobj).release_wait = (*syncobj).release_wait.wrapping_add(1);
        (*syncobj).release_freelist_cursor = core::ptr::addr_of_mut!((*relobj).entry);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    return hr;
}

unsafe extern "system" fn d3d12_sync_object_async_release_QueryInterface(iface: *mut IRtwqAsyncCallback, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0) || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IRtwqAsyncCallback))) != 0) {
        *obj = iface as *mut core::ffi::c_void;
        IRtwqAsyncCallback_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}
unsafe extern "system" fn d3d12_sync_object_async_release_AddRef(iface: *mut IRtwqAsyncCallback) -> ULONG {
    return d3d12_sync_object_AddRef(&raw mut (*d3d12_sync_object_from_IRtwqAsyncCallback(iface)).IMFD3D12SynchronizationObject_iface);
}
unsafe extern "system" fn d3d12_sync_object_async_release_Release(iface: *mut IRtwqAsyncCallback) -> ULONG {
    return d3d12_sync_object_Release(&raw mut (*d3d12_sync_object_from_IRtwqAsyncCallback(iface)).IMFD3D12SynchronizationObject_iface);
}
unsafe extern "system" fn d3d12_sync_object_async_release_GetParameters(
    iface: *mut IRtwqAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn d3d12_sync_object_async_release_Invoke(iface: *mut IRtwqAsyncCallback, result: *mut IRtwqAsyncResult) -> HRESULT {
    let syncobj: *mut d3d12_sync_object = d3d12_sync_object_from_IRtwqAsyncCallback(iface);

    TRACE!("%p, %p.\n", iface, result);

    EnterCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    (*syncobj).release_wait = (*syncobj).release_wait.wrapping_sub(1);
    if (*syncobj).release_wait == 0 && !(*syncobj).final_release_event.is_null() {
        SetEvent((*syncobj).final_release_event);
        (*syncobj).final_release_event = core::ptr::null_mut();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateD3D12SynchronizationObject(
    device: *mut ID3D12Device,
    riid: *const IID,
    obj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut syncobj: *mut d3d12_sync_object;

    if obj.is_null() {
        return E_INVALIDARG;
    }

    syncobj = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<d3d12_sync_object>() as size_t) as _,
    ) as *mut d3d12_sync_object;
    if syncobj.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = ID3D12Device_CreateFence(
        device,
        0 as UINT64,
        D3D12_FENCE_FLAG_SHARED,
        core::ptr::addr_of!(IID_ID3D12Fence),
        core::ptr::addr_of_mut!((*syncobj).ready_fence) as *mut *mut ::core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        free(syncobj as *mut ::core::ffi::c_void);
        return hr;
    }

    (*syncobj).refcount = 1 as LONG;
    (*syncobj).IMFD3D12SynchronizationObject_iface.lpVtbl =
        core::ptr::addr_of!(d3d12_sync_object_vtbl);
    (*syncobj).IMFD3D12SynchronizationObjectCommands_iface.lpVtbl =
        core::ptr::addr_of!(d3d12_sync_object_commands_vtbl);
    (*syncobj).async_release_iface.lpVtbl =
        core::ptr::addr_of!(d3d12_sync_object_async_release_vtbl);
    InitializeCriticalSection(core::ptr::addr_of_mut!((*syncobj).cs));
    ID3D12Device_AddRef(device);
    (*syncobj).device = device;
    (*syncobj).generation = 1 as UINT64;
    list_init(core::ptr::addr_of_mut!((*syncobj).release_freelist));
    (*syncobj).release_freelist_cursor = core::ptr::addr_of_mut!((*syncobj).release_freelist);

    hr = IMFD3D12SynchronizationObject_QueryInterface(
        core::ptr::addr_of_mut!((*syncobj).IMFD3D12SynchronizationObject_iface),
        riid,
        obj,
    );
    IMFD3D12SynchronizationObject_Release(core::ptr::addr_of_mut!((*syncobj).IMFD3D12SynchronizationObject_iface));

    return hr;
}

unsafe extern "C" fn impl_from_IMFDXGIDeviceManager(
    iface: *mut IMFDXGIDeviceManager,
) -> *mut dxgi_device_manager {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(dxgi_device_manager, IMFDXGIDeviceManager_iface),
    ) as *mut dxgi_device_manager;
}

unsafe extern "C" fn dxgi_device_manager_get_handle_index(manager: *mut dxgi_device_manager, hdevice: HANDLE, idx: *mut size_t) -> HRESULT {
    if hdevice.is_null() || hdevice > ULongToHandle!((*manager).count as ::core::ffi::c_long) {
        return E_HANDLE;
    }
    *idx = (hdevice as ULONG_PTR).wrapping_sub(1 as ULONG_PTR) as size_t;
    return S_OK;
}

unsafe extern "system" fn dxgi_device_manager_QueryInterface(iface: *mut IMFDXGIDeviceManager, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFDXGIDeviceManager)) != 0 ||
        IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFDXGIDeviceManager_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn dxgi_device_manager_AddRef(iface: *mut IMFDXGIDeviceManager) -> ULONG {
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*manager).refcount)) as ULONG;

    return refcount;
}

unsafe extern "system" fn dxgi_device_manager_Release(iface: *mut IMFDXGIDeviceManager) -> ULONG {
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*manager).refcount) as *mut LONG) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        if !(*manager).device.is_null() {
            IUnknown_Release((*manager).device);
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*manager).cs));
        free((*manager).handles as *mut core::ffi::c_void);
        free(manager as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn dxgi_device_manager_lock_handle(
    mut manager: *mut dxgi_device_manager,
    mut idx: size_t,
) {
    if *(*manager).handles.offset(idx as isize)
        & DXGI_DEVICE_HANDLE_FLAG_LOCKED as ::core::ffi::c_uint
        != 0
    {
        return;
    }
    *(*manager).handles.offset(idx as isize) |=
        DXGI_DEVICE_HANDLE_FLAG_LOCKED as ::core::ffi::c_uint;
    (*manager).locks = (*manager).locks.wrapping_add(1);
}

unsafe extern "C" fn dxgi_device_manager_unlock_handle(manager: *mut dxgi_device_manager, idx: size_t) {
    if (*(*manager).handles.add(idx as usize) & (DXGI_DEVICE_HANDLE_FLAG_LOCKED as ::core::ffi::c_uint)) == 0 {
        return;
    }
    *(*manager).handles.add(idx as usize) &= !(DXGI_DEVICE_HANDLE_FLAG_LOCKED as ::core::ffi::c_uint);
    (*manager).locks = (*manager).locks.wrapping_sub(1);
    if (*manager).locks == 0 {
        (*manager).locking_tid = 0 as ::core::ffi::c_uint;
    }
}

unsafe extern "system" fn dxgi_device_manager_CloseDeviceHandle(iface: *mut IMFDXGIDeviceManager, hdevice: HANDLE) -> HRESULT {
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let mut hr: HRESULT;
    let mut idx: usize = 0;
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0".as_ptr() as *const i8,
            b"dxgi_device_manager_CloseDeviceHandle\0".as_ptr() as *const i8,
            9426i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface,
            hdevice,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    hr = dxgi_device_manager_get_handle_index(manager, hdevice, core::ptr::addr_of_mut!(idx));
    if hr >= 0 {
        if *(*manager).handles.add(idx) & (DXGI_DEVICE_HANDLE_FLAG_OPEN as u32) != 0 {
            dxgi_device_manager_unlock_handle(manager, idx);
            *(*manager).handles.add(idx) = 0;
            if idx == (*manager).count.wrapping_sub(1) {
                (*manager).count = (*manager).count.wrapping_sub(1);
            }
        } else {
            hr = E_HANDLE;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    WakeAllConditionVariable(core::ptr::addr_of_mut!((*manager).lock));

    return hr;
}

unsafe extern "system" fn dxgi_device_manager_GetVideoService(
    iface: *mut IMFDXGIDeviceManager,
    hdevice: HANDLE,
    riid: *const IID,
    service: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"dxgi_device_manager_GetVideoService\0" as *const u8 as *const core::ffi::c_char,
            9457 as core::ffi::c_int,
            b"%p, %p, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            hdevice,
            debugstr_guid(riid as *const _GUID),
            service,
        );
    }
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let mut hr: HRESULT;
    let mut idx: size_t = 0;

    EnterCriticalSection(&raw mut (*manager).cs);

    if (*manager).device.is_null() {
        hr = MF_E_DXGI_DEVICE_NOT_INITIALIZED;
    } else {
        hr = dxgi_device_manager_get_handle_index(manager, hdevice, &raw mut idx);
        if hr >= 0 {
            if *(*manager).handles.offset(idx as isize) & (DXGI_DEVICE_HANDLE_FLAG_INVALID as u32) != 0 {
                hr = MF_E_DXGI_NEW_VIDEO_DEVICE;
            } else if *(*manager).handles.offset(idx as isize) & (DXGI_DEVICE_HANDLE_FLAG_OPEN as u32) != 0 {
                hr = IUnknown_QueryInterface((*manager).device, riid, service);
            } else {
                hr = E_HANDLE;
            }
        }
    }

    LeaveCriticalSection(&raw mut (*manager).cs);

    return hr;
}

unsafe extern "system" fn dxgi_device_manager_LockDevice(iface: *mut IMFDXGIDeviceManager, hdevice: HANDLE, riid: REFIID, obj: *mut *mut core::ffi::c_void, block: BOOL) -> HRESULT {
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let mut hr: HRESULT = 0;
    let mut idx: size_t = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    hr = dxgi_device_manager_get_handle_index(manager, hdevice, core::ptr::addr_of_mut!(idx));
    if hr >= 0 {
        if (*manager).device.is_null() {
            hr = MF_E_DXGI_DEVICE_NOT_INITIALIZED;
        } else if ((*manager).locking_tid as DWORD) == GetCurrentThreadId() {
            hr = IUnknown_QueryInterface((*manager).device, riid, obj);
            if hr >= 0 {
                dxgi_device_manager_lock_handle(manager, idx);
            }
        } else if (*manager).locking_tid != 0 && block == 0 {
            hr = MF_E_DXGI_VIDEO_DEVICE_LOCKED;
        } else {
            while (*manager).locking_tid != 0 {
                SleepConditionVariableCS(
                    core::ptr::addr_of_mut!((*manager).lock),
                    core::ptr::addr_of_mut!((*manager).cs),
                    INFINITE,
                );
            }

            hr = dxgi_device_manager_get_handle_index(manager, hdevice, core::ptr::addr_of_mut!(idx));
            if hr >= 0 {
                if (*(*manager).handles.add(idx) & (DXGI_DEVICE_HANDLE_FLAG_INVALID as u32)) != 0 {
                    hr = MF_E_DXGI_NEW_VIDEO_DEVICE;
                } else {
                    hr = IUnknown_QueryInterface((*manager).device, riid, obj);
                    if hr >= 0 {
                        (*manager).locking_tid = GetCurrentThreadId() as DWORD;
                        dxgi_device_manager_lock_handle(manager, idx);
                    }
                }
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    return hr;
}
unsafe extern "system" fn dxgi_device_manager_OpenDeviceHandle(iface: *mut IMFDXGIDeviceManager, hdevice: *mut HANDLE) -> HRESULT {
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: size_t;

    TRACE!("%p, %p.\n", iface, hdevice);

    *hdevice = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    if (*manager).device.is_null() {
        hr = MF_E_DXGI_DEVICE_NOT_INITIALIZED;
    } else {
        i = 0 as size_t;
        while i < (*manager).count {
            if (*(*manager).handles.offset(i as isize)
                & (DXGI_DEVICE_HANDLE_FLAG_OPEN as ::core::ffi::c_uint))
                == 0
            {
                *(*manager).handles.offset(i as isize) |=
                    DXGI_DEVICE_HANDLE_FLAG_OPEN as ::core::ffi::c_uint;
                *hdevice = ULongToHandle!(
                    i.wrapping_add(1 as size_t) as ::core::ffi::c_long,
                ) as HANDLE;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }

        if mf_array_reserve(
            core::ptr::addr_of_mut!((*manager).handles) as *mut *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!((*manager).capacity),
            (*manager).count.wrapping_add(1 as size_t),
            ::core::mem::size_of::<::core::ffi::c_uint>() as size_t,
        ) != 0
        {
            *hdevice = ULongToHandle!(
                (*manager).count.wrapping_add(1 as size_t)
                    as ::core::ffi::c_long,
            ) as HANDLE;
            let fresh = (*manager).count;
            (*manager).count = (*manager).count.wrapping_add(1);
            *(*manager).handles.offset(fresh as isize) =
                DXGI_DEVICE_HANDLE_FLAG_OPEN as ::core::ffi::c_uint;
        } else {
            hr = E_OUTOFMEMORY;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    return hr;
}
unsafe extern "system" fn dxgi_device_manager_ResetDevice(iface: *mut IMFDXGIDeviceManager, device: *mut IUnknown, token: UINT) -> HRESULT {
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let mut d3d_device: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
    let mut i: size_t = 0;

    if device.is_null() || token != (*manager).token {
        return E_INVALIDARG;
    }

    if (*(*device).lpVtbl)
        .QueryInterface
        .unwrap_unchecked()(
        device,
        &raw const IID_ID3D11Device,
        &raw mut d3d_device as *mut *mut core::ffi::c_void,
    ) < 0
    {
        if (*(*device).lpVtbl)
            .QueryInterface
            .unwrap_unchecked()(
            device,
            &raw const IID_ID3D12Device,
            &raw mut d3d_device as *mut *mut core::ffi::c_void,
        ) < 0
        {
            WARN!("Unsupported device interface.\n");
            return E_INVALIDARG;
        }
    }

    EnterCriticalSection(&raw mut (*manager).cs);

    if !(*manager).device.is_null() {
        i = 0 as size_t;
        while i < (*manager).count {
            *(*manager).handles.offset(i as isize) |= (DXGI_DEVICE_HANDLE_FLAG_INVALID as u32);
            *(*manager).handles.offset(i as isize) &= (!DXGI_DEVICE_HANDLE_FLAG_LOCKED) as u32;
            i = i.wrapping_add(1);
        }
        (*manager).locking_tid = 0;
        (*manager).locks = 0;
        (*(*(*manager).device).lpVtbl)
            .Release
            .unwrap_unchecked()((*manager).device);
    }
    (*manager).device = d3d_device;

    LeaveCriticalSection(&raw mut (*manager).cs);

    WakeAllConditionVariable(&raw mut (*manager).lock);

    return S_OK;
}

unsafe extern "system" fn dxgi_device_manager_TestDevice(iface: *mut IMFDXGIDeviceManager, hdevice: HANDLE) -> HRESULT {
    let manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let mut hr: HRESULT;
    let mut idx: size_t = 0;

    EnterCriticalSection(&raw mut (*manager).cs);

    hr = dxgi_device_manager_get_handle_index(manager, hdevice, &raw mut idx);
    if hr >= 0 {
        if *(*manager).handles.add(idx) & (DXGI_DEVICE_HANDLE_FLAG_INVALID as u32) != 0 {
            hr = MF_E_DXGI_NEW_VIDEO_DEVICE;
        } else if *(*manager).handles.add(idx) & (DXGI_DEVICE_HANDLE_FLAG_OPEN as u32) == 0 {
            hr = E_HANDLE;
        }
    }

    LeaveCriticalSection(&raw mut (*manager).cs);

    return hr;
}
unsafe extern "system" fn dxgi_device_manager_UnlockDevice(
    mut iface: *mut IMFDXGIDeviceManager,
    mut hdevice: HANDLE,
    mut savestate: BOOL,
) -> HRESULT {
    let mut manager: *mut dxgi_device_manager = impl_from_IMFDXGIDeviceManager(iface);
    let mut hr: HRESULT = E_FAIL;
    let mut idx: size_t = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"dxgi_device_manager_UnlockDevice\0" as *const u8 as *const ::core::ffi::c_char,
            9642 as ::core::ffi::c_int,
            b"%p, %p, %d.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            hdevice,
            savestate,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    if dxgi_device_manager_get_handle_index(manager, hdevice, core::ptr::addr_of_mut!(idx)) >= 0 {
        hr = if (*(*manager).handles.offset(idx as isize) & (DXGI_DEVICE_HANDLE_FLAG_LOCKED as u32)) != 0 {
            S_OK
        } else {
            E_INVALIDARG
        };
        if hr >= 0 {
            dxgi_device_manager_unlock_handle(manager, idx);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    WakeAllConditionVariable(core::ptr::addr_of_mut!((*manager).lock));

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateDXGIDeviceManager(token: *mut UINT, manager: *mut *mut IMFDXGIDeviceManager) -> HRESULT {
    let mut object: *mut dxgi_device_manager = core::ptr::null_mut();

    if token.is_null() || manager.is_null() {
        return E_POINTER;
    }

    object = calloc((1 as usize) as _, (core::mem::size_of::<dxgi_device_manager>() as usize) as _) as *mut dxgi_device_manager;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFDXGIDeviceManager_iface.lpVtbl = core::ptr::addr_of!(dxgi_device_manager_vtbl);
    (*object).refcount = 1 as LONG;
    (*object).token = GetTickCount() as UINT;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));
    InitializeConditionVariable(core::ptr::addr_of_mut!((*object).lock));

    *token = (*object).token;
    *manager = core::ptr::addr_of_mut!((*object).IMFDXGIDeviceManager_iface);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFLockDXGIDeviceManager(token: *mut UINT, manager: *mut *mut IMFDXGIDeviceManager) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFLockDXGIDeviceManager\0" as *const u8 as *const ::core::ffi::c_char,
            9710 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            token,
            manager,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(shared_dm_cs));

    if shared_dm.manager.is_null() {
        hr = MFCreateDXGIDeviceManager(
            core::ptr::addr_of_mut!(shared_dm.token),
            core::ptr::addr_of_mut!(shared_dm.manager),
        );
    }

    if hr >= 0 {
        *manager = shared_dm.manager;
        IMFDXGIDeviceManager_AddRef(*manager);
        shared_dm.locks = shared_dm.locks.wrapping_add(1);

        if !token.is_null() {
            *token = shared_dm.token as UINT;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(shared_dm_cs));

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFUnlockDXGIDeviceManager() -> HRESULT {
    EnterCriticalSection(core::ptr::addr_of_mut!(shared_dm_cs));

    if !shared_dm.manager.is_null() {
        IMFDXGIDeviceManager_Release(shared_dm.manager);
        shared_dm.locks = shared_dm.locks.wrapping_sub(1);
        if shared_dm.locks == 0 {
            shared_dm.manager = core::ptr::null_mut();
            shared_dm.token = 0;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(shared_dm_cs));

    return S_OK;
}

unsafe extern "C" fn llmult128(
    mut c1: *mut ULARGE_INTEGER,
    mut c0: *mut ULARGE_INTEGER,
    mut val: LONGLONG,
    mut num: LONGLONG,
) {
    let mut a1: ULARGE_INTEGER = core::mem::zeroed();
    let mut b0: ULARGE_INTEGER = core::mem::zeroed();
    let mut v: ULARGE_INTEGER = core::mem::zeroed();
    let mut n: ULARGE_INTEGER = core::mem::zeroed();
    v.QuadPart = llabs(val as ::core::ffi::c_longlong) as ULONGLONG;
    n.QuadPart = llabs(num as ::core::ffi::c_longlong) as ULONGLONG;
    (*c0).QuadPart =
        (v.__bindgen_anon_1.LowPart as ULONGLONG).wrapping_mul(n.__bindgen_anon_1.LowPart as ULONGLONG);
    a1.QuadPart = (v.__bindgen_anon_1.LowPart as ULONGLONG)
        .wrapping_mul(n.__bindgen_anon_1.HighPart as ULONGLONG);
    b0.QuadPart = (v.__bindgen_anon_1.HighPart as ULONGLONG)
        .wrapping_mul(n.__bindgen_anon_1.LowPart as ULONGLONG);
    (*c1).QuadPart = ((*c0).__bindgen_anon_1.HighPart as ULONGLONG)
        .wrapping_add(a1.__bindgen_anon_1.LowPart as ULONGLONG)
        .wrapping_add(b0.__bindgen_anon_1.LowPart as ULONGLONG);
    (*c0).__bindgen_anon_1.HighPart = (*c1).__bindgen_anon_1.LowPart;
    (*c1).QuadPart = (v.__bindgen_anon_1.HighPart as ULONGLONG)
        .wrapping_mul(n.__bindgen_anon_1.HighPart as ULONGLONG)
        .wrapping_add((*c1).__bindgen_anon_1.HighPart as ULONGLONG)
        .wrapping_add(a1.__bindgen_anon_1.HighPart as ULONGLONG)
        .wrapping_add(b0.__bindgen_anon_1.HighPart as ULONGLONG);
}

unsafe extern "C" fn lldiv128(
    mut c1: ULARGE_INTEGER,
    mut c0: ULARGE_INTEGER,
    mut denom: LONGLONG,
) -> ULONGLONG {
    let mut q1: ULARGE_INTEGER = _ULARGE_INTEGER {
        __bindgen_anon_1: _ULARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    let mut q0: ULARGE_INTEGER = _ULARGE_INTEGER {
        __bindgen_anon_1: _ULARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    let mut rhat: ULARGE_INTEGER = _ULARGE_INTEGER {
        __bindgen_anon_1: _ULARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    let mut v: ULARGE_INTEGER = _ULARGE_INTEGER {
        __bindgen_anon_1: _ULARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    let mut cmp1: ULARGE_INTEGER = _ULARGE_INTEGER {
        __bindgen_anon_1: _ULARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    let mut cmp2: ULARGE_INTEGER = _ULARGE_INTEGER {
        __bindgen_anon_1: _ULARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    let mut s: DWORD = 0 as DWORD;
    v.QuadPart = llabs(denom as ::core::ffi::c_longlong) as ULONGLONG;
    if c1.QuadPart == 0 as ULONGLONG {
        return c0.QuadPart.wrapping_div(v.QuadPart);
    }
    if v.__bindgen_anon_1.HighPart == 0 as DWORD
        && c1.__bindgen_anon_1.HighPart == 0 as DWORD
    {
        let mut low: ULONGLONG = c0.__bindgen_anon_1.LowPart as ULONGLONG;
        let mut high: ULONGLONG = (c0.__bindgen_anon_1.HighPart as ULONGLONG)
            .wrapping_add((c1.__bindgen_anon_1.LowPart as ULONGLONG) << 32);
        low = low.wrapping_add(
            high.wrapping_rem(v.__bindgen_anon_1.LowPart as ULONGLONG) << 32,
        );
        return (high.wrapping_div(v.__bindgen_anon_1.LowPart as ULONGLONG)
            << 32)
            .wrapping_add(low.wrapping_div(v.__bindgen_anon_1.LowPart as ULONGLONG));
    }
    if v.__bindgen_anon_1.HighPart == 0 as DWORD {
        return UI64_MAX;
    }
    BitScanReverse(
        core::ptr::addr_of_mut!(s),
        v.__bindgen_anon_1.HighPart as ::core::ffi::c_ulong,
    );
    s = (31 as DWORD).wrapping_sub(s);
    if s != 0 {
        v.QuadPart <<= s;
        c1.QuadPart = (c1.QuadPart << s)
            | ((c0.__bindgen_anon_1.HighPart >> (32 as DWORD).wrapping_sub(s))
                as ULONGLONG);
        c0.QuadPart <<= s;
    }
    q1.QuadPart = c1
        .QuadPart
        .wrapping_div(v.__bindgen_anon_1.HighPart as ULONGLONG);
    rhat.QuadPart = c1.QuadPart.wrapping_sub(
        q1.QuadPart
            .wrapping_mul(v.__bindgen_anon_1.HighPart as ULONGLONG),
    );
    cmp1.__bindgen_anon_1.HighPart = rhat.__bindgen_anon_1.LowPart;
    cmp1.__bindgen_anon_1.LowPart = c0.__bindgen_anon_1.HighPart;
    cmp2.QuadPart = q1
        .QuadPart
        .wrapping_mul(v.__bindgen_anon_1.LowPart as ULONGLONG);
    while q1.__bindgen_anon_1.HighPart != 0 || cmp2.QuadPart > cmp1.QuadPart {
        q1.QuadPart = q1.QuadPart.wrapping_sub(1);
        rhat.QuadPart = rhat
            .QuadPart
            .wrapping_add(v.__bindgen_anon_1.HighPart as ULONGLONG);
        if rhat.__bindgen_anon_1.HighPart != 0 {
            break;
        }
        cmp1.__bindgen_anon_1.HighPart = rhat.__bindgen_anon_1.LowPart;
        cmp2.QuadPart = cmp2
            .QuadPart
            .wrapping_sub(v.__bindgen_anon_1.LowPart as ULONGLONG);
    }
    c1.__bindgen_anon_1.HighPart = c1.__bindgen_anon_1.LowPart;
    c1.__bindgen_anon_1.LowPart = c0.__bindgen_anon_1.HighPart;
    c1.QuadPart = c1
        .QuadPart
        .wrapping_sub(q1.QuadPart.wrapping_mul(v.QuadPart));
    q0.QuadPart = c1
        .QuadPart
        .wrapping_div(v.__bindgen_anon_1.HighPart as ULONGLONG);
    rhat.QuadPart = c1.QuadPart.wrapping_sub(
        q0.QuadPart
            .wrapping_mul(v.__bindgen_anon_1.HighPart as ULONGLONG),
    );
    cmp1.__bindgen_anon_1.HighPart = rhat.__bindgen_anon_1.LowPart;
    cmp1.__bindgen_anon_1.LowPart = c0.__bindgen_anon_1.LowPart;
    cmp2.QuadPart = q0
        .QuadPart
        .wrapping_mul(v.__bindgen_anon_1.LowPart as ULONGLONG);
    while q0.__bindgen_anon_1.HighPart != 0 || cmp2.QuadPart > cmp1.QuadPart {
        q0.QuadPart = q0.QuadPart.wrapping_sub(1);
        rhat.QuadPart = rhat
            .QuadPart
            .wrapping_add(v.__bindgen_anon_1.HighPart as ULONGLONG);
        if rhat.__bindgen_anon_1.HighPart != 0 {
            break;
        }
        cmp1.__bindgen_anon_1.HighPart = rhat.__bindgen_anon_1.LowPart;
        cmp2.QuadPart = cmp2
            .QuadPart
            .wrapping_sub(v.__bindgen_anon_1.LowPart as ULONGLONG);
    }
    q0.__bindgen_anon_1.HighPart = q0

        .__bindgen_anon_1
        .HighPart
        .wrapping_add(q1.__bindgen_anon_1.LowPart);
    return q0.QuadPart;
}

#[no_mangle]
pub unsafe extern "system" fn MFllMulDiv(
    mut val: LONGLONG,
    mut num: LONGLONG,
    mut denom: LONGLONG,
    mut factor: LONGLONG,
) -> LONGLONG {
    let mut sign: u32 = 0;
    let mut factor_sign: u32 = 0;
    let mut denom_sign: u32 = 0;
    let mut c1: ULARGE_INTEGER = core::mem::zeroed();
    let mut c0: ULARGE_INTEGER = core::mem::zeroed();
    let mut ret: ULONGLONG = 0;

    /* compute 128-bit numerator product */
    llmult128(&raw mut c1, &raw mut c0, val, num);

    sign = ((val < 0) ^ (num < 0)) as u32;
    factor_sign = (factor < 0) as u32;
    denom_sign = (denom < 0) as u32;

    factor = llabs(factor);
    if sign == factor_sign {
        if UI64_MAX - c0.QuadPart < factor as ULONGLONG {
            if c1.QuadPart == UI64_MAX {
                return if sign != 0 { i64::MIN } else { i64::MAX }; // LLOVERFLOW: reads `sign` at each use
            }
            c1.QuadPart = c1.QuadPart.wrapping_add(1);
        }
        c0.QuadPart = c0.QuadPart.wrapping_add(factor as ULONGLONG);
    } else {
        if c0.QuadPart >= factor as ULONGLONG {
            c0.QuadPart = c0.QuadPart.wrapping_sub(factor as ULONGLONG);
        } else {
            if c1.QuadPart != 0 {
                c1.QuadPart = c1.QuadPart.wrapping_sub(1);
            } else {
                sign = (sign == 0) as u32;
            }

            c0.QuadPart = (factor as ULONGLONG).wrapping_sub(c0.QuadPart);
        }
    }

    if c1.QuadPart >= denom as ULONGLONG {
        return if sign != 0 { i64::MIN } else { i64::MAX }; // LLOVERFLOW: reads `sign` at each use
    }

    /* compute quotient, fits in 64 bits */
    ret = lldiv128(c1, c0, denom);
    sign ^= denom_sign;
    if ret >= (I64_MAX as _) {
        return if sign != 0 { i64::MIN } else { i64::MAX }; // LLOVERFLOW: reads `sign` at each use
    }
    if sign != 0 {
        (ret as LONGLONG).wrapping_neg()
    } else {
        ret as LONGLONG
    }
}

#[no_mangle]
pub unsafe extern "system" fn MFCreatePathFromURL(url: *const WCHAR, ret_path: *mut *mut WCHAR) -> HRESULT {
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut length: DWORD;
    let mut hr: HRESULT;

    TRACE!("%s, %p.\n", debugstr_w(url), ret_path);

    if url.is_null() || ret_path.is_null() {
        return E_POINTER;
    }

    length = ARRAY_SIZE!(path) as DWORD;
    hr = PathCreateFromUrlW(
        url as LPCWSTR,
        core::ptr::addr_of_mut!(path) as LPWSTR,
        core::ptr::addr_of_mut!(length),
        0 as DWORD,
    );
    if hr < 0 as HRESULT {
        return hr;
    }

    *ret_path = CoTaskMemAlloc(
        (length.wrapping_add(1 as DWORD) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if (*ret_path).is_null() {
        return E_OUTOFMEMORY;
    }

    memcpy(
        *ret_path as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(path) as *mut WCHAR as *const core::ffi::c_void,
        (length.wrapping_add(1 as DWORD) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFSerializeAttributesToStream(
    attr: *mut IMFAttributes,
    options: DWORD,
    stream: *mut IStream,
) -> HRESULT {
    // FIXME("%p %lx %p: stub!\n", attr, options, stream);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn MFDeserializeAttributesFromStream(
    attr: *mut IMFAttributes,
    options: DWORD,
    stream: *mut IStream,
) -> HRESULT {
    // FIXME("%p %lx %p: stub!\n", attr, options, stream);
    E_NOTIMPL
}

// COM vtable tables — generated from main.c by vtbl_gen.py
/// main.c:552 `static const IMFActivateVtbl transform_activate_vtbl`  (slot order: rust:types_amd64.rs)
pub static transform_activate_vtbl: IMFActivateVtbl = IMFActivateVtbl {
    QueryInterface: Some(transform_activate_QueryInterface),
    AddRef: Some(transform_activate_AddRef),
    Release: Some(transform_activate_Release),
    GetItem: Some(transform_activate_GetItem),
    GetItemType: Some(transform_activate_GetItemType),
    CompareItem: Some(transform_activate_CompareItem),
    Compare: Some(transform_activate_Compare),
    GetUINT32: Some(transform_activate_GetUINT32),
    GetUINT64: Some(transform_activate_GetUINT64),
    GetDouble: Some(transform_activate_GetDouble),
    GetGUID: Some(transform_activate_GetGUID),
    GetStringLength: Some(transform_activate_GetStringLength),
    GetString: Some(transform_activate_GetString),
    GetAllocatedString: Some(transform_activate_GetAllocatedString),
    GetBlobSize: Some(transform_activate_GetBlobSize),
    GetBlob: Some(transform_activate_GetBlob),
    GetAllocatedBlob: Some(transform_activate_GetAllocatedBlob),
    GetUnknown: Some(transform_activate_GetUnknown),
    SetItem: Some(transform_activate_SetItem),
    DeleteItem: Some(transform_activate_DeleteItem),
    DeleteAllItems: Some(transform_activate_DeleteAllItems),
    SetUINT32: Some(transform_activate_SetUINT32),
    SetUINT64: Some(transform_activate_SetUINT64),
    SetDouble: Some(transform_activate_SetDouble),
    SetGUID: Some(transform_activate_SetGUID),
    SetString: Some(transform_activate_SetString),
    SetBlob: Some(transform_activate_SetBlob),
    SetUnknown: Some(transform_activate_SetUnknown),
    LockStore: Some(transform_activate_LockStore),
    UnlockStore: Some(transform_activate_UnlockStore),
    GetCount: Some(transform_activate_GetCount),
    GetItemByIndex: Some(transform_activate_GetItemByIndex),
    CopyAllItems: Some(transform_activate_CopyAllItems),
    ActivateObject: Some(transform_activate_ActivateObject),
    ShutdownObject: Some(transform_activate_ShutdownObject),
    DetachObject: Some(transform_activate_DetachObject),
};

/// main.c:3283 `static const IMFAttributesVtbl mfattributes_vtbl`  (slot order: rust:types_amd64.rs)
pub static mfattributes_vtbl: IMFAttributesVtbl = IMFAttributesVtbl {
    QueryInterface: Some(mfattributes_QueryInterface),
    AddRef: Some(mfattributes_AddRef),
    Release: Some(mfattributes_Release),
    GetItem: Some(mfattributes_GetItem),
    GetItemType: Some(mfattributes_GetItemType),
    CompareItem: Some(mfattributes_CompareItem),
    Compare: Some(mfattributes_Compare),
    GetUINT32: Some(mfattributes_GetUINT32),
    GetUINT64: Some(mfattributes_GetUINT64),
    GetDouble: Some(mfattributes_GetDouble),
    GetGUID: Some(mfattributes_GetGUID),
    GetStringLength: Some(mfattributes_GetStringLength),
    GetString: Some(mfattributes_GetString),
    GetAllocatedString: Some(mfattributes_GetAllocatedString),
    GetBlobSize: Some(mfattributes_GetBlobSize),
    GetBlob: Some(mfattributes_GetBlob),
    GetAllocatedBlob: Some(mfattributes_GetAllocatedBlob),
    GetUnknown: Some(mfattributes_GetUnknown),
    SetItem: Some(mfattributes_SetItem),
    DeleteItem: Some(mfattributes_DeleteItem),
    DeleteAllItems: Some(mfattributes_DeleteAllItems),
    SetUINT32: Some(mfattributes_SetUINT32),
    SetUINT64: Some(mfattributes_SetUINT64),
    SetDouble: Some(mfattributes_SetDouble),
    SetGUID: Some(mfattributes_SetGUID),
    SetString: Some(mfattributes_SetString),
    SetBlob: Some(mfattributes_SetBlob),
    SetUnknown: Some(mfattributes_SetUnknown),
    LockStore: Some(mfattributes_LockStore),
    UnlockStore: Some(mfattributes_UnlockStore),
    GetCount: Some(mfattributes_GetCount),
    GetItemByIndex: Some(mfattributes_GetItemByIndex),
    CopyAllItems: Some(mfattributes_CopyAllItems),
};

/// main.c:3778 `static const IUnknownVtbl async_stream_op_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static async_stream_op_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(async_stream_op_QueryInterface),
    AddRef: Some(async_stream_op_AddRef),
    Release: Some(async_stream_op_Release),
};

/// main.c:4196 `static const IMFByteStreamVtbl bytestream_file_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_file_vtbl: IMFByteStreamVtbl = IMFByteStreamVtbl {
    QueryInterface: Some(bytestream_QueryInterface),
    AddRef: Some(bytestream_AddRef),
    Release: Some(bytestream_Release),
    GetCapabilities: Some(bytestream_file_GetCapabilities),
    GetLength: Some(bytestream_file_GetLength),
    SetLength: Some(bytestream_file_SetLength),
    GetCurrentPosition: Some(bytestream_GetCurrentPosition),
    SetCurrentPosition: Some(bytestream_SetCurrentPosition),
    IsEndOfStream: Some(bytestream_file_IsEndOfStream),
    Read: Some(bytestream_file_Read),
    BeginRead: Some(bytestream_BeginRead),
    EndRead: Some(bytestream_EndRead),
    Write: Some(bytestream_file_Write),
    BeginWrite: Some(bytestream_BeginWrite),
    EndWrite: Some(bytestream_EndWrite),
    Seek: Some(bytestream_Seek),
    Flush: Some(bytestream_file_Flush),
    Close: Some(bytestream_file_Close),
};

/// main.c:4330 `static const IMFByteStreamVtbl bytestream_stream_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_stream_vtbl: IMFByteStreamVtbl = IMFByteStreamVtbl {
    QueryInterface: Some(bytestream_QueryInterface),
    AddRef: Some(bytestream_AddRef),
    Release: Some(bytestream_Release),
    GetCapabilities: Some(bytestream_stream_GetCapabilities),
    GetLength: Some(bytestream_stream_GetLength),
    SetLength: Some(bytestream_stream_SetLength),
    GetCurrentPosition: Some(bytestream_GetCurrentPosition),
    SetCurrentPosition: Some(bytestream_SetCurrentPosition),
    IsEndOfStream: Some(bytestream_stream_IsEndOfStream),
    Read: Some(bytestream_stream_Read),
    BeginRead: Some(bytestream_BeginRead),
    EndRead: Some(bytestream_EndRead),
    Write: Some(bytestream_stream_Write),
    BeginWrite: Some(bytestream_BeginWrite),
    EndWrite: Some(bytestream_EndWrite),
    Seek: Some(bytestream_Seek),
    Flush: Some(bytestream_stream_Flush),
    Close: Some(bytestream_stream_Close),
};

/// main.c:4375 `static const IMFAttributesVtbl bytestream_attributes_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_attributes_vtbl: IMFAttributesVtbl = IMFAttributesVtbl {
    QueryInterface: Some(bytestream_attributes_QueryInterface),
    AddRef: Some(bytestream_attributes_AddRef),
    Release: Some(bytestream_attributes_Release),
    GetItem: Some(mfattributes_GetItem),
    GetItemType: Some(mfattributes_GetItemType),
    CompareItem: Some(mfattributes_CompareItem),
    Compare: Some(mfattributes_Compare),
    GetUINT32: Some(mfattributes_GetUINT32),
    GetUINT64: Some(mfattributes_GetUINT64),
    GetDouble: Some(mfattributes_GetDouble),
    GetGUID: Some(mfattributes_GetGUID),
    GetStringLength: Some(mfattributes_GetStringLength),
    GetString: Some(mfattributes_GetString),
    GetAllocatedString: Some(mfattributes_GetAllocatedString),
    GetBlobSize: Some(mfattributes_GetBlobSize),
    GetBlob: Some(mfattributes_GetBlob),
    GetAllocatedBlob: Some(mfattributes_GetAllocatedBlob),
    GetUnknown: Some(mfattributes_GetUnknown),
    SetItem: Some(mfattributes_SetItem),
    DeleteItem: Some(mfattributes_DeleteItem),
    DeleteAllItems: Some(mfattributes_DeleteAllItems),
    SetUINT32: Some(mfattributes_SetUINT32),
    SetUINT64: Some(mfattributes_SetUINT64),
    SetDouble: Some(mfattributes_SetDouble),
    SetGUID: Some(mfattributes_SetGUID),
    SetString: Some(mfattributes_SetString),
    SetBlob: Some(mfattributes_SetBlob),
    SetUnknown: Some(mfattributes_SetUnknown),
    LockStore: Some(mfattributes_LockStore),
    UnlockStore: Some(mfattributes_UnlockStore),
    GetCount: Some(mfattributes_GetCount),
    GetItemByIndex: Some(mfattributes_GetItemByIndex),
    CopyAllItems: Some(mfattributes_CopyAllItems),
};

/// main.c:4464 `static const IRtwqAsyncCallbackVtbl bytestream_read_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_read_callback_vtbl: IRtwqAsyncCallbackVtbl = IRtwqAsyncCallbackVtbl {
    QueryInterface: Some(bytestream_callback_QueryInterface),
    AddRef: Some(bytestream_read_callback_AddRef),
    Release: Some(bytestream_read_callback_Release),
    GetParameters: Some(bytestream_callback_GetParameters),
    Invoke: Some(bytestream_read_callback_Invoke),
};

/// main.c:4473 `static const IRtwqAsyncCallbackVtbl bytestream_write_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_write_callback_vtbl: IRtwqAsyncCallbackVtbl = IRtwqAsyncCallbackVtbl {
    QueryInterface: Some(bytestream_callback_QueryInterface),
    AddRef: Some(bytestream_write_callback_AddRef),
    Release: Some(bytestream_write_callback_Release),
    GetParameters: Some(bytestream_callback_GetParameters),
    Invoke: Some(bytestream_write_callback_Invoke),
};

/// main.c:4556 `static const IMFGetServiceVtbl bytestream_file_getservice_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_file_getservice_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(bytestream_file_getservice_QueryInterface),
    AddRef: Some(bytestream_file_getservice_AddRef),
    Release: Some(bytestream_file_getservice_Release),
    GetService: Some(bytestream_file_getservice_GetService),
};

/// main.c:5004 `static const IMFByteStreamVtbl bytestream_wrapper_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_vtbl: IMFByteStreamVtbl = IMFByteStreamVtbl {
    QueryInterface: Some(bytestream_wrapper_QueryInterface),
    AddRef: Some(bytestream_wrapper_AddRef),
    Release: Some(bytestream_wrapper_Release),
    GetCapabilities: Some(bytestream_wrapper_GetCapabilities),
    GetLength: Some(bytestream_wrapper_GetLength),
    SetLength: Some(bytestream_wrapper_SetLength),
    GetCurrentPosition: Some(bytestream_wrapper_GetCurrentPosition),
    SetCurrentPosition: Some(bytestream_wrapper_SetCurrentPosition),
    IsEndOfStream: Some(bytestream_wrapper_IsEndOfStream),
    Read: Some(bytestream_wrapper_Read),
    BeginRead: Some(bytestream_wrapper_BeginRead),
    EndRead: Some(bytestream_wrapper_EndRead),
    Write: Some(bytestream_wrapper_Write),
    BeginWrite: Some(bytestream_wrapper_BeginWrite),
    EndWrite: Some(bytestream_wrapper_EndWrite),
    Seek: Some(bytestream_wrapper_Seek),
    Flush: Some(bytestream_wrapper_Flush),
    Close: Some(bytestream_wrapper_Close),
};

/// main.c:5054 `static const IMFByteStreamCacheControlVtbl bytestream_wrapper_cache_control_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_cache_control_vtbl: IMFByteStreamCacheControlVtbl = IMFByteStreamCacheControlVtbl {
    QueryInterface: Some(bytestream_wrapper_cache_control_QueryInterface),
    AddRef: Some(bytestream_wrapper_cache_control_AddRef),
    Release: Some(bytestream_wrapper_cache_control_Release),
    StopBackgroundTransfer: Some(bytestream_wrapper_cache_control_StopBackgroundTransfer),
};

/// main.c:5110 `static const IMFByteStreamBufferingVtbl bytestream_wrapper_buffering_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_buffering_vtbl: IMFByteStreamBufferingVtbl = IMFByteStreamBufferingVtbl {
    QueryInterface: Some(bytestream_wrapper_buffering_QueryInterface),
    AddRef: Some(bytestream_wrapper_buffering_AddRef),
    Release: Some(bytestream_wrapper_buffering_Release),
    SetBufferingParams: Some(bytestream_wrapper_buffering_SetBufferingParams),
    EnableBuffering: Some(bytestream_wrapper_buffering_EnableBuffering),
    StopBuffering: Some(bytestream_wrapper_buffering_StopBuffering),
};

/// main.c:5167 `static const IMFByteStreamTimeSeekVtbl bytestream_wrapper_timeseek_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_timeseek_vtbl: IMFByteStreamTimeSeekVtbl = IMFByteStreamTimeSeekVtbl {
    QueryInterface: Some(bytestream_wrapper_timeseek_QueryInterface),
    AddRef: Some(bytestream_wrapper_timeseek_AddRef),
    Release: Some(bytestream_wrapper_timeseek_Release),
    IsTimeSeekSupported: Some(bytestream_wrapper_timeseek_IsTimeSeekSupported),
    TimeSeek: Some(bytestream_wrapper_timeseek_TimeSeek),
    GetTimeSeekResult: Some(bytestream_wrapper_timeseek_GetTimeSeekResult),
};

/// main.c:5232 `static const IMFMediaEventGeneratorVtbl bytestream_wrapper_events_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_events_vtbl: IMFMediaEventGeneratorVtbl = IMFMediaEventGeneratorVtbl {
    QueryInterface: Some(bytestream_wrapper_events_QueryInterface),
    AddRef: Some(bytestream_wrapper_events_AddRef),
    Release: Some(bytestream_wrapper_events_Release),
    GetEvent: Some(bytestream_wrapper_events_GetEvent),
    BeginGetEvent: Some(bytestream_wrapper_events_BeginGetEvent),
    EndGetEvent: Some(bytestream_wrapper_events_EndGetEvent),
    QueueEvent: Some(bytestream_wrapper_events_QueueEvent),
};

/// main.c:5289 `static const IMFSampleOutputStreamVtbl bytestream_wrapper_sample_output_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_sample_output_vtbl: IMFSampleOutputStreamVtbl = IMFSampleOutputStreamVtbl {
    QueryInterface: Some(bytestream_wrapper_sample_output_QueryInterface),
    AddRef: Some(bytestream_wrapper_sample_output_AddRef),
    Release: Some(bytestream_wrapper_sample_output_Release),
    BeginWriteSample: Some(bytestream_wrapper_sample_output_BeginWriteSample),
    EndWriteSample: Some(bytestream_wrapper_sample_output_EndWriteSample),
    Close: Some(bytestream_wrapper_sample_output_Close),
};

/// main.c:5363 `static const IPropertyStoreVtbl bytestream_wrapper_propstore_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_propstore_vtbl: IPropertyStoreVtbl = IPropertyStoreVtbl {
    QueryInterface: Some(bytestream_wrapper_propstore_QueryInterface),
    AddRef: Some(bytestream_wrapper_propstore_AddRef),
    Release: Some(bytestream_wrapper_propstore_Release),
    GetCount: Some(bytestream_wrapper_propstore_GetCount),
    GetAt: Some(bytestream_wrapper_propstore_GetAt),
    GetValue: Some(bytestream_wrapper_propstore_GetValue),
    SetValue: Some(bytestream_wrapper_propstore_SetValue),
    Commit: Some(bytestream_wrapper_propstore_Commit),
};

/// main.c:5667 `static const IMFAttributesVtbl bytestream_wrapper_attributes_vtbl`  (slot order: rust:types_amd64.rs)
pub static bytestream_wrapper_attributes_vtbl: IMFAttributesVtbl = IMFAttributesVtbl {
    QueryInterface: Some(bytestream_wrapper_attributes_QueryInterface),
    AddRef: Some(bytestream_wrapper_attributes_AddRef),
    Release: Some(bytestream_wrapper_attributes_Release),
    GetItem: Some(bytestream_wrapper_attributes_GetItem),
    GetItemType: Some(bytestream_wrapper_attributes_GetItemType),
    CompareItem: Some(bytestream_wrapper_attributes_CompareItem),
    Compare: Some(bytestream_wrapper_attributes_Compare),
    GetUINT32: Some(bytestream_wrapper_attributes_GetUINT32),
    GetUINT64: Some(bytestream_wrapper_attributes_GetUINT64),
    GetDouble: Some(bytestream_wrapper_attributes_GetDouble),
    GetGUID: Some(bytestream_wrapper_attributes_GetGUID),
    GetStringLength: Some(bytestream_wrapper_attributes_GetStringLength),
    GetString: Some(bytestream_wrapper_attributes_GetString),
    GetAllocatedString: Some(bytestream_wrapper_attributes_GetAllocatedString),
    GetBlobSize: Some(bytestream_wrapper_attributes_GetBlobSize),
    GetBlob: Some(bytestream_wrapper_attributes_GetBlob),
    GetAllocatedBlob: Some(bytestream_wrapper_attributes_GetAllocatedBlob),
    GetUnknown: Some(bytestream_wrapper_attributes_GetUnknown),
    SetItem: Some(bytestream_wrapper_attributes_SetItem),
    DeleteItem: Some(bytestream_wrapper_attributes_DeleteItem),
    DeleteAllItems: Some(bytestream_wrapper_attributes_DeleteAllItems),
    SetUINT32: Some(bytestream_wrapper_attributes_SetUINT32),
    SetUINT64: Some(bytestream_wrapper_attributes_SetUINT64),
    SetDouble: Some(bytestream_wrapper_attributes_SetDouble),
    SetGUID: Some(bytestream_wrapper_attributes_SetGUID),
    SetString: Some(bytestream_wrapper_attributes_SetString),
    SetBlob: Some(bytestream_wrapper_attributes_SetBlob),
    SetUnknown: Some(bytestream_wrapper_attributes_SetUnknown),
    LockStore: Some(bytestream_wrapper_attributes_LockStore),
    UnlockStore: Some(bytestream_wrapper_attributes_UnlockStore),
    GetCount: Some(bytestream_wrapper_attributes_GetCount),
    GetItemByIndex: Some(bytestream_wrapper_attributes_GetItemByIndex),
    CopyAllItems: Some(bytestream_wrapper_attributes_CopyAllItems),
};

/// main.c:5812 `static const IMFPluginControlVtbl MFPluginControlVtbl`  (slot order: rust:types_amd64.rs)
pub static MFPluginControlVtbl: IMFPluginControlVtbl = IMFPluginControlVtbl {
    QueryInterface: Some(MFPluginControl_QueryInterface),
    AddRef: Some(MFPluginControl_AddRef),
    Release: Some(MFPluginControl_Release),
    GetPreferredClsid: Some(MFPluginControl_GetPreferredClsid),
    GetPreferredClsidByIndex: Some(MFPluginControl_GetPreferredClsidByIndex),
    SetPreferredClsid: Some(MFPluginControl_SetPreferredClsid),
    IsDisabled: Some(MFPluginControl_IsDisabled),
    GetDisabledByIndex: Some(MFPluginControl_GetDisabledByIndex),
    SetDisabled: Some(MFPluginControl_SetDisabled),
};

/// main.c:5996 `static const IUnknownVtbl resolver_cancel_object_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static resolver_cancel_object_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(resolver_cancel_object_QueryInterface),
    AddRef: Some(resolver_cancel_object_AddRef),
    Release: Some(resolver_cancel_object_Release),
};

/// main.c:6071 `static const IRtwqAsyncCallbackVtbl source_resolver_callback_stream_vtbl`  (slot order: rust:types_amd64.rs)
pub static source_resolver_callback_stream_vtbl: IRtwqAsyncCallbackVtbl = IRtwqAsyncCallbackVtbl {
    QueryInterface: Some(source_resolver_callback_QueryInterface),
    AddRef: Some(source_resolver_callback_stream_AddRef),
    Release: Some(source_resolver_callback_stream_Release),
    GetParameters: Some(source_resolver_callback_GetParameters),
    Invoke: Some(source_resolver_callback_stream_Invoke),
};

/// main.c:6099 `static const IRtwqAsyncCallbackVtbl source_resolver_callback_url_vtbl`  (slot order: rust:types_amd64.rs)
pub static source_resolver_callback_url_vtbl: IRtwqAsyncCallbackVtbl = IRtwqAsyncCallbackVtbl {
    QueryInterface: Some(source_resolver_callback_QueryInterface),
    AddRef: Some(source_resolver_callback_url_AddRef),
    Release: Some(source_resolver_callback_url_Release),
    GetParameters: Some(source_resolver_callback_GetParameters),
    Invoke: Some(source_resolver_callback_url_Invoke),
};

/// main.c:6830 `static const IMFSourceResolverVtbl mfsourceresolvervtbl`  (slot order: rust:types_amd64.rs)
pub static mfsourceresolvervtbl: IMFSourceResolverVtbl = IMFSourceResolverVtbl {
    QueryInterface: Some(source_resolver_QueryInterface),
    AddRef: Some(source_resolver_AddRef),
    Release: Some(source_resolver_Release),
    CreateObjectFromURL: Some(source_resolver_CreateObjectFromURL),
    CreateObjectFromByteStream: Some(source_resolver_CreateObjectFromByteStream),
    BeginCreateObjectFromURL: Some(source_resolver_BeginCreateObjectFromURL),
    EndCreateObjectFromURL: Some(source_resolver_EndCreateObjectFromURL),
    BeginCreateObjectFromByteStream: Some(source_resolver_BeginCreateObjectFromByteStream),
    EndCreateObjectFromByteStream: Some(source_resolver_EndCreateObjectFromByteStream),
    CancelObjectCreation: Some(source_resolver_CancelObjectCreation),
};

/// main.c:7255 `static const IMFMediaEventVtbl mfmediaevent_vtbl`  (slot order: rust:types_amd64.rs)
pub static mfmediaevent_vtbl: IMFMediaEventVtbl = IMFMediaEventVtbl {
    QueryInterface: Some(mfmediaevent_QueryInterface),
    AddRef: Some(mfmediaevent_AddRef),
    Release: Some(mfmediaevent_Release),
    GetItem: Some(mfmediaevent_GetItem),
    GetItemType: Some(mfmediaevent_GetItemType),
    CompareItem: Some(mfmediaevent_CompareItem),
    Compare: Some(mfmediaevent_Compare),
    GetUINT32: Some(mfmediaevent_GetUINT32),
    GetUINT64: Some(mfmediaevent_GetUINT64),
    GetDouble: Some(mfmediaevent_GetDouble),
    GetGUID: Some(mfmediaevent_GetGUID),
    GetStringLength: Some(mfmediaevent_GetStringLength),
    GetString: Some(mfmediaevent_GetString),
    GetAllocatedString: Some(mfmediaevent_GetAllocatedString),
    GetBlobSize: Some(mfmediaevent_GetBlobSize),
    GetBlob: Some(mfmediaevent_GetBlob),
    GetAllocatedBlob: Some(mfmediaevent_GetAllocatedBlob),
    GetUnknown: Some(mfmediaevent_GetUnknown),
    SetItem: Some(mfmediaevent_SetItem),
    DeleteItem: Some(mfmediaevent_DeleteItem),
    DeleteAllItems: Some(mfmediaevent_DeleteAllItems),
    SetUINT32: Some(mfmediaevent_SetUINT32),
    SetUINT64: Some(mfmediaevent_SetUINT64),
    SetDouble: Some(mfmediaevent_SetDouble),
    SetGUID: Some(mfmediaevent_SetGUID),
    SetString: Some(mfmediaevent_SetString),
    SetBlob: Some(mfmediaevent_SetBlob),
    SetUnknown: Some(mfmediaevent_SetUnknown),
    LockStore: Some(mfmediaevent_LockStore),
    UnlockStore: Some(mfmediaevent_UnlockStore),
    GetCount: Some(mfmediaevent_GetCount),
    GetItemByIndex: Some(mfmediaevent_GetItemByIndex),
    CopyAllItems: Some(mfmediaevent_CopyAllItems),
    GetType: Some(mfmediaevent_GetType),
    GetExtendedType: Some(mfmediaevent_GetExtendedType),
    GetStatus: Some(mfmediaevent_GetStatus),
    GetValue: Some(mfmediaevent_GetValue),
};

/// main.c:7642 `static const IMFMediaEventQueueVtbl eventqueuevtbl`  (slot order: rust:types_amd64.rs)
pub static eventqueuevtbl: IMFMediaEventQueueVtbl = IMFMediaEventQueueVtbl {
    QueryInterface: Some(eventqueue_QueryInterface),
    AddRef: Some(eventqueue_AddRef),
    Release: Some(eventqueue_Release),
    GetEvent: Some(eventqueue_GetEvent),
    BeginGetEvent: Some(eventqueue_BeginGetEvent),
    EndGetEvent: Some(eventqueue_EndGetEvent),
    QueueEvent: Some(eventqueue_QueueEvent),
    QueueEventParamVar: Some(eventqueue_QueueEventParamVar),
    QueueEventParamUnk: Some(eventqueue_QueueEventParamUnk),
    Shutdown: Some(eventqueue_Shutdown),
};

/// main.c:7868 `static const IMFCollectionVtbl mfcollectionvtbl`  (slot order: rust:types_amd64.rs)
pub static mfcollectionvtbl: IMFCollectionVtbl = IMFCollectionVtbl {
    QueryInterface: Some(collection_QueryInterface),
    AddRef: Some(collection_AddRef),
    Release: Some(collection_Release),
    GetElementCount: Some(collection_GetElementCount),
    GetElement: Some(collection_GetElement),
    AddElement: Some(collection_AddElement),
    RemoveElement: Some(collection_RemoveElement),
    InsertElementAt: Some(collection_InsertElementAt),
    RemoveAllElements: Some(collection_RemoveAllElements),
};

/// main.c:8025 `static const IMFClockVtbl system_clock_vtbl`  (slot order: rust:types_amd64.rs)
pub static system_clock_vtbl: IMFClockVtbl = IMFClockVtbl {
    QueryInterface: Some(system_clock_QueryInterface),
    AddRef: Some(system_clock_AddRef),
    Release: Some(system_clock_Release),
    GetClockCharacteristics: Some(system_clock_GetClockCharacteristics),
    GetCorrelatedTime: Some(system_clock_GetCorrelatedTime),
    GetContinuityKey: Some(system_clock_GetContinuityKey),
    GetState: Some(system_clock_GetState),
    GetProperties: Some(system_clock_GetProperties),
};

/// main.c:8185 `static const IMFPresentationTimeSourceVtbl systemtimesourcevtbl`  (slot order: rust:types_amd64.rs)
pub static systemtimesourcevtbl: IMFPresentationTimeSourceVtbl = IMFPresentationTimeSourceVtbl {
    QueryInterface: Some(system_time_source_QueryInterface),
    AddRef: Some(system_time_source_AddRef),
    Release: Some(system_time_source_Release),
    GetClockCharacteristics: Some(system_time_source_GetClockCharacteristics),
    GetCorrelatedTime: Some(system_time_source_GetCorrelatedTime),
    GetContinuityKey: Some(system_time_source_GetContinuityKey),
    GetState: Some(system_time_source_GetState),
    GetProperties: Some(system_time_source_GetProperties),
    GetUnderlyingClock: Some(system_time_source_GetUnderlyingClock),
};

/// main.c:8358 `static const IMFClockStateSinkVtbl systemtimesourcesinkvtbl`  (slot order: rust:types_amd64.rs)
pub static systemtimesourcesinkvtbl: IMFClockStateSinkVtbl = IMFClockStateSinkVtbl {
    QueryInterface: Some(system_time_source_sink_QueryInterface),
    AddRef: Some(system_time_source_sink_AddRef),
    Release: Some(system_time_source_sink_Release),
    OnClockStart: Some(system_time_source_sink_OnClockStart),
    OnClockStop: Some(system_time_source_sink_OnClockStop),
    OnClockPause: Some(system_time_source_sink_OnClockPause),
    OnClockRestart: Some(system_time_source_sink_OnClockRestart),
    OnClockSetRate: Some(system_time_source_sink_OnClockSetRate),
};

/// main.c:8509 `static const IRtwqAsyncCallbackVtbl async_create_file_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static async_create_file_callback_vtbl: IRtwqAsyncCallbackVtbl = IRtwqAsyncCallbackVtbl {
    QueryInterface: Some(async_create_file_callback_QueryInterface),
    AddRef: Some(async_create_file_callback_AddRef),
    Release: Some(async_create_file_callback_Release),
    GetParameters: Some(async_create_file_callback_GetParameters),
    Invoke: Some(async_create_file_callback_Invoke),
};

/// main.c:8871 `static const IPropertyStoreVtbl property_store_vtbl`  (slot order: rust:types_amd64.rs)
pub static property_store_vtbl: IPropertyStoreVtbl = IPropertyStoreVtbl {
    QueryInterface: Some(property_store_QueryInterface),
    AddRef: Some(property_store_AddRef),
    Release: Some(property_store_Release),
    GetCount: Some(property_store_GetCount),
    GetAt: Some(property_store_GetAt),
    GetValue: Some(property_store_GetValue),
    SetValue: Some(property_store_SetValue),
    Commit: Some(property_store_Commit),
};

/// main.c:9238 `static const IMFD3D12SynchronizationObjectVtbl d3d12_sync_object_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d12_sync_object_vtbl: IMFD3D12SynchronizationObjectVtbl = IMFD3D12SynchronizationObjectVtbl {
    QueryInterface: Some(d3d12_sync_object_QueryInterface),
    AddRef: Some(d3d12_sync_object_AddRef),
    Release: Some(d3d12_sync_object_Release),
    SignalEventOnFinalResourceRelease: Some(d3d12_sync_object_SignalEventOnFinalResourceRelease),
    Reset: Some(d3d12_sync_object_Reset),
};

/// main.c:9247 `static const IMFD3D12SynchronizationObjectCommandsVtbl d3d12_sync_object_commands_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d12_sync_object_commands_vtbl: IMFD3D12SynchronizationObjectCommandsVtbl = IMFD3D12SynchronizationObjectCommandsVtbl {
    QueryInterface: Some(d3d12_sync_object_commands_QueryInterface),
    AddRef: Some(d3d12_sync_object_commands_AddRef),
    Release: Some(d3d12_sync_object_commands_Release),
    EnqueueResourceReady: Some(d3d12_sync_object_commands_EnqueueResourceReady),
    EnqueueResourceReadyWait: Some(d3d12_sync_object_commands_EnqueueResourceReadyWait),
    SignalEventOnResourceReady: Some(d3d12_sync_object_commands_SignalEventOnResourceReady),
    EnqueueResourceRelease: Some(d3d12_sync_object_commands_EnqueueResourceRelease),
};

/// main.c:9258 `static const IRtwqAsyncCallbackVtbl d3d12_sync_object_async_release_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d12_sync_object_async_release_vtbl: IRtwqAsyncCallbackVtbl = IRtwqAsyncCallbackVtbl {
    QueryInterface: Some(d3d12_sync_object_async_release_QueryInterface),
    AddRef: Some(d3d12_sync_object_async_release_AddRef),
    Release: Some(d3d12_sync_object_async_release_Release),
    GetParameters: Some(d3d12_sync_object_async_release_GetParameters),
    Invoke: Some(d3d12_sync_object_async_release_Invoke),
};

/// main.c:9660 `static const IMFDXGIDeviceManagerVtbl dxgi_device_manager_vtbl`  (slot order: rust:types_amd64.rs)
pub static dxgi_device_manager_vtbl: IMFDXGIDeviceManagerVtbl = IMFDXGIDeviceManagerVtbl {
    QueryInterface: Some(dxgi_device_manager_QueryInterface),
    AddRef: Some(dxgi_device_manager_AddRef),
    Release: Some(dxgi_device_manager_Release),
    CloseDeviceHandle: Some(dxgi_device_manager_CloseDeviceHandle),
    GetVideoService: Some(dxgi_device_manager_GetVideoService),
    LockDevice: Some(dxgi_device_manager_LockDevice),
    OpenDeviceHandle: Some(dxgi_device_manager_OpenDeviceHandle),
    ResetDevice: Some(dxgi_device_manager_ResetDevice),
    TestDevice: Some(dxgi_device_manager_TestDevice),
    UnlockDevice: Some(dxgi_device_manager_UnlockDevice),
};
// enforce_import_abi: moved 1 import decl(s) into the extern block their
// exporting DLL's .spec requires: memcpy->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
