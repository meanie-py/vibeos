use crate::mfplat::buffer::buffer;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut MFVideoFormat_RGB1: GUID = _GUID {
    Data1: D3DFMT_A1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_RGB4: GUID = _GUID {
    Data1: ('4' as i32 as BYTE as DWORD
        | ('P' as i32 as BYTE as DWORD) << 8 as ::core::ffi::c_int
        | ('x' as i32 as BYTE as DWORD) << 16 as ::core::ffi::c_int
        | ('x' as i32 as BYTE as DWORD) << 24 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_ABGR32: GUID = _GUID {
    Data1: D3DFMT_A8B8G8R8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_ARGB1555: GUID = _GUID {
    Data1: D3DFMT_A1R5G5B5 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_ARGB4444: GUID = _GUID {
    Data1: D3DFMT_A4R4G4B4 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_A2B10G10R10: GUID = _GUID {
    Data1: D3DFMT_A2R10G10B10 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_IMC1: GUID = _GUID {
    Data1: ('I' as i32 as BYTE as DWORD
        | ('M' as i32 as BYTE as DWORD) << 8 as ::core::ffi::c_int
        | ('C' as i32 as BYTE as DWORD) << 16 as ::core::ffi::c_int
        | ('1' as i32 as BYTE as DWORD) << 24 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_IMC2: GUID = _GUID {
    Data1: ('I' as i32 as BYTE as DWORD
        | ('M' as i32 as BYTE as DWORD) << 8 as ::core::ffi::c_int
        | ('C' as i32 as BYTE as DWORD) << 16 as ::core::ffi::c_int
        | ('2' as i32 as BYTE as DWORD) << 24 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_IMC3: GUID = _GUID {
    Data1: ('I' as i32 as BYTE as DWORD
        | ('M' as i32 as BYTE as DWORD) << 8 as ::core::ffi::c_int
        | ('C' as i32 as BYTE as DWORD) << 16 as ::core::ffi::c_int
        | ('3' as i32 as BYTE as DWORD) << 24 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut MFVideoFormat_IMC4: GUID = _GUID {
    Data1: ('I' as i32 as BYTE as DWORD
        | ('M' as i32 as BYTE as DWORD) << 8 as ::core::ffi::c_int
        | ('C' as i32 as BYTE as DWORD) << 16 as ::core::ffi::c_int
        | ('4' as i32 as BYTE as DWORD) << 24 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut default_channel_mask: [UINT32; 7] = [
    0 as ::core::ffi::c_int as UINT32,
    SPEAKER_FRONT_LEFT as UINT32,
    (SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT) as UINT32,
    (SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER) as UINT32,
    KSAUDIO_SPEAKER_QUAD as UINT32,
    (KSAUDIO_SPEAKER_QUAD | SPEAKER_FRONT_CENTER) as UINT32,
    KSAUDIO_SPEAKER_5POINT1 as UINT32,
];

static mut video_formats: [uncompressed_video_format; 34] = unsafe {
    [
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_RGB1,
            bpp: 1 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_RGB4,
            bpp: 4 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_RGB24,
            bpp: 24 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_ARGB32,
            bpp: 32 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_RGB32,
            bpp: 32 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_RGB565,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_BITFIELDS as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_RGB555,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_ABGR32,
            bpp: 32 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_A2R10G10B10,
            bpp: 32 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_A2B10G10R10,
            bpp: 32 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_RGB8,
            bpp: 8 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_L8,
            bpp: 8 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_AYUV,
            bpp: 32 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_I420,
            bpp: 12 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_IMC1,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_IMC2,
            bpp: 12 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_IMC3,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_IMC4,
            bpp: 12 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_IYUV,
            bpp: 12 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_NV11,
            bpp: 12 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_NV12,
            bpp: 12 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_P010,
            bpp: 24 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_D16,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_L16,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_UYVY,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_YUY2,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_YV12,
            bpp: 12 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_YVYU,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 0 as ::core::ffi::c_uchar,
            bottom_up: 0 as ::core::ffi::c_uchar,
            yuv: 1 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MFVideoFormat_A16B16G16R16F,
            bpp: 64 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: -(1 as ::core::ffi::c_int),
        },
        uncompressed_video_format {
            subtype: &raw const MEDIASUBTYPE_RGB8,
            bpp: 8 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MEDIASUBTYPE_RGB565,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_BITFIELDS as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MEDIASUBTYPE_RGB555,
            bpp: 16 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MEDIASUBTYPE_RGB24,
            bpp: 24 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
        uncompressed_video_format {
            subtype: &raw const MEDIASUBTYPE_RGB32,
            bpp: 32 as ::core::ffi::c_uchar,
            alignment: 3 as ::core::ffi::c_uchar,
            bottom_up: 1 as ::core::ffi::c_uchar,
            yuv: 0 as ::core::ffi::c_uchar,
            compression: (BI_RGB as i32),
        },
    ]
};

static mut known_rates: [frame_rate; 8] = [
    frame_rate {
        time: 417188 as UINT64,
        rate: (24000 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1001 as UINT64,
    },
    frame_rate {
        time: 416667 as UINT64,
        rate: (24 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1 as UINT64,
    },
    frame_rate {
        time: 400000 as UINT64,
        rate: (25 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1 as UINT64,
    },
    frame_rate {
        time: 333667 as UINT64,
        rate: (30000 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1001 as UINT64,
    },
    frame_rate {
        time: 333333 as UINT64,
        rate: (30 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1 as UINT64,
    },
    frame_rate {
        time: 200000 as UINT64,
        rate: (50 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1 as UINT64,
    },
    frame_rate {
        time: 166833 as UINT64,
        rate: (60000 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1001 as UINT64,
    },
    frame_rate {
        time: 166667 as UINT64,
        rate: (60 as ::core::ffi::c_int as UINT64) << 32 as ::core::ffi::c_int | 1 as UINT64,
    },
];

// #define COBJMACROS: C preprocessor only, no Rust equivalent needed.
// WINE_DEFAULT_DEBUG_CHANNEL(mfplat): handled by the crate's DPRINT!/WARN!/ERR! macros.

// DEFINE_MEDIATYPE_GUID(name, fmt) == DEFINE_GUID(name, fmt, 0x0000, 0x0010, 0x80,0x00,0x00,0xaa,0x00,0x38,0x9b,0x71)

#[repr(C)]
pub struct media_type {
    pub attributes: attributes,
    pub IMFMediaType_iface: IMFMediaType,
    pub IMFVideoMediaType_iface: IMFVideoMediaType,
    pub IMFAudioMediaType_iface: IMFAudioMediaType,
    pub video_format: *mut MFVIDEOFORMAT,
    pub audio_format: *mut WAVEFORMATEX,
}

#[repr(C)]
pub struct stream_desc {
    pub attributes: attributes,
    pub IMFStreamDescriptor_iface: IMFStreamDescriptor,
    pub IMFMediaTypeHandler_iface: IMFMediaTypeHandler,
    pub identifier: DWORD,
    pub media_types: *mut *mut IMFMediaType,
    pub media_types_count: u32,
    pub current_type: *mut IMFMediaType,
}

#[repr(C)]
pub struct presentation_desc_entry {
    pub descriptor: *mut IMFStreamDescriptor,
    pub selected: BOOL,
}

#[repr(C)]
pub struct presentation_desc {
    pub attributes: attributes,
    pub IMFPresentationDescriptor_iface: IMFPresentationDescriptor,
    pub descriptors: *mut presentation_desc_entry,
    pub count: u32,
}

// static HRESULT presentation_descriptor_init(...);  -- forward fn declaration, omitted (no fn definitions here)
// static const IMFMediaTypeVtbl mediatypevtbl = {...};                 -- COM vtable, skipped (vtbl_gen.py)
// static const IMFVideoMediaTypeVtbl videomediatypevtbl = {...};       -- COM vtable, skipped (vtbl_gen.py)
// static const IMFAudioMediaTypeVtbl audiomediatypevtbl = {...};       -- COM vtable, skipped (vtbl_gen.py)
// static const IMFStreamDescriptorVtbl streamdescriptorvtbl = {...};   -- COM vtable, skipped (vtbl_gen.py)
// static const IMFMediaTypeHandlerVtbl mediatypehandlervtbl = {...};   -- COM vtable, skipped (vtbl_gen.py)
// static const IMFPresentationDescriptorVtbl presentationdescriptorvtbl = {...}; -- COM vtable, skipped (vtbl_gen.py)

#[repr(C)]
pub struct uncompressed_video_format {
    pub subtype: *const GUID,
    pub bpp: u8,
    pub alignment: u8,
    pub bottom_up: u8,
    pub yuv: u8,
    pub compression: i32,
}

#[repr(C)]
pub struct frame_rate {
    pub time: UINT64,
    pub rate: UINT64,
}

static audio_mediatype_SetUINT32_FILE: [u8; 0] = [];

unsafe extern "C" fn get_am_subtype_for_mf_subtype(mut subtype: GUID) -> GUID {
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB1)) != false {
        return MEDIASUBTYPE_RGB1;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB4)) != false {
        return MEDIASUBTYPE_RGB4;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB8)) != false {
        return MEDIASUBTYPE_RGB8;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB555)) != false {
        return MEDIASUBTYPE_RGB555;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB565)) != false {
        return MEDIASUBTYPE_RGB565;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB24)) != false {
        return MEDIASUBTYPE_RGB24;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB32)) != false {
        return MEDIASUBTYPE_RGB32;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_ARGB1555)) != false {
        return MEDIASUBTYPE_ARGB1555;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_ARGB4444)) != false {
        return MEDIASUBTYPE_ARGB4444;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_ARGB32)) != false {
        return MEDIASUBTYPE_ARGB32;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_A2B10G10R10)) != false {
        return MEDIASUBTYPE_A2R10G10B10;
    }
    if IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(MFVideoFormat_A2R10G10B10)) != false {
        return MEDIASUBTYPE_A2B10G10R10;
    }
    return subtype;
}

unsafe extern "C" fn impl_from_IMFMediaType(iface: *mut IMFMediaType) -> *mut media_type {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(media_type, IMFMediaType_iface)) as *mut media_type;
}

unsafe extern "C" fn impl_from_IMFVideoMediaType(iface: *mut IMFVideoMediaType) -> *mut media_type {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(media_type, IMFVideoMediaType_iface)) as *mut media_type;
}

unsafe extern "C" fn impl_from_IMFAudioMediaType(iface: *mut IMFAudioMediaType) -> *mut media_type {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(media_type, IMFAudioMediaType_iface)) as *mut media_type;
}

unsafe extern "C" fn impl_from_IMFStreamDescriptor(iface: *mut IMFStreamDescriptor) -> *mut stream_desc {
    return (iface as *mut u8).sub(core::mem::offset_of!(stream_desc, IMFStreamDescriptor_iface)) as *mut stream_desc;
}

unsafe extern "C" fn impl_from_IMFMediaTypeHandler(iface: *mut IMFMediaTypeHandler) -> *mut stream_desc {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(stream_desc, IMFMediaTypeHandler_iface)) as *mut stream_desc;
}

unsafe extern "C" fn impl_from_IMFPresentationDescriptor(iface: *mut IMFPresentationDescriptor) -> *mut presentation_desc {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(presentation_desc, IMFPresentationDescriptor_iface)) as *mut presentation_desc;
}

unsafe extern "system" fn mediatype_QueryInterface(iface: *mut IMFMediaType, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    let mut major: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };

    attributes_GetGUID(&raw mut (*media_type).attributes, &raw const MF_MT_MAJOR_TYPE, &raw mut major);

    if IsEqualGUID!(&raw const major, &raw const MFMediaType_Video) != false && IsEqualIID!(riid, &raw const IID_IMFVideoMediaType) != 0
    {
        *out = &raw mut (*media_type).IMFVideoMediaType_iface as *mut core::ffi::c_void;
    }
    else if IsEqualGUID!(&raw const major, &raw const MFMediaType_Audio) != false && IsEqualIID!(riid, &raw const IID_IMFAudioMediaType) != 0
    {
        *out = &raw mut (*media_type).IMFAudioMediaType_iface as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, &raw const IID_IMFMediaType) != 0 ||
            IsEqualIID!(riid, &raw const IID_IMFAttributes) != 0 ||
            IsEqualIID!(riid, &raw const IID_IUnknown) != 0
    {
        *out = &raw mut (*media_type).IMFMediaType_iface as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn mediatype_AddRef(iface: *mut IMFMediaType) -> ULONG {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*media_type).attributes.ref_) as *mut i32,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    return refcount;
}

unsafe extern "system" fn mediatype_Release(iface: *mut IMFMediaType) -> ULONG {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*media_type).attributes.ref_) as *mut i32,
    ) as ULONG;

    if refcount == 0 {
        clear_attributes_object(core::ptr::addr_of_mut!((*media_type).attributes));
        CoTaskMemFree((*media_type).video_format as LPVOID);
        CoTaskMemFree((*media_type).audio_format as LPVOID);
        free(media_type as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn mediatype_GetItem(
    iface: *mut IMFMediaType,
    key: *const GUID,
    value: *mut PROPVARIANT,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);
    return attributes_GetItem(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn mediatype_GetItemType(
    iface: *mut IMFMediaType,
    key: REFGUID,
    type_: *mut MF_ATTRIBUTE_TYPE,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"mediatype_GetItemType\0" as *const u8 as *const i8,
            235 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            type_,
        );
    }
    return attributes_GetItemType(&raw mut (*media_type).attributes, key, type_);
}

unsafe extern "system" fn mediatype_CompareItem(iface: *mut IMFMediaType, key: *const GUID, value: *const PROPVARIANT, result: *mut BOOL) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"mediatype_CompareItem\0".as_ptr() as *const i8,
            244i32,
            b"%p, %s, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return attributes_CompareItem(core::ptr::addr_of_mut!((*media_type).attributes), key, value, result);
}

unsafe extern "system" fn mediatype_Compare(
    iface: *mut IMFMediaType,
    attrs: *mut IMFAttributes,
    type_0: MF_ATTRIBUTES_MATCH_TYPE,
    result: *mut BOOL,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    TRACE!("%p, %p, %d, %p.\n", iface, attrs, type_0 as u32, result);

    return attributes_Compare(core::ptr::addr_of_mut!((*media_type).attributes), attrs, type_0, result);
}

unsafe extern "system" fn mediatype_GetUINT32(
    iface: *mut IMFMediaType,
    key: *const GUID,
    value: *mut UINT32,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mediatype_GetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            263 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT32(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn mediatype_GetUINT64(
    iface: *mut IMFMediaType,
    key: *const GUID,
    value: *mut UINT64,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);
    return attributes_GetUINT64(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn mediatype_GetDouble(iface: *mut IMFMediaType, key: *const GUID, value: *mut f64) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);

    return attributes_GetDouble(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn mediatype_GetGUID(iface: *mut IMFMediaType, key: *const GUID, value: *mut GUID) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"mediatype_GetGUID\0".as_ptr() as *const i8,
            290 as i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetGUID(&raw mut (*media_type).attributes, key, value);
}

unsafe extern "system" fn mediatype_GetStringLength(iface: *mut IMFMediaType, key: REFGUID, length: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_GetStringLength(core::ptr::addr_of_mut!((*media_type).attributes), key, length);
}

unsafe extern "system" fn mediatype_GetString(iface: *mut IMFMediaType, key: *const GUID, value: *mut WCHAR, size: UINT32, length: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_GetString(core::ptr::addr_of_mut!((*media_type).attributes), key, value, size, length);
}

unsafe extern "system" fn mediatype_GetAllocatedString(iface: *mut IMFMediaType, key: REFGUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_GetAllocatedString(core::ptr::addr_of_mut!((*media_type).attributes), key, value, length);
}

unsafe extern "system" fn mediatype_GetBlobSize(iface: *mut IMFMediaType, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_GetBlobSize(core::ptr::addr_of_mut!((*media_type).attributes), key, size);
}
unsafe extern "system" fn mediatype_GetBlob(
    mut iface: *mut IMFMediaType,
    key: *const GUID,
    mut buf: *mut UINT8,
    mut bufsize: UINT32,
    mut blobsize: *mut UINT32,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mediatype_GetBlob\0" as *const u8 as *const ::core::ffi::c_char,
            338 as ::core::ffi::c_int,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            buf,
            bufsize,
            blobsize,
        );
    }
    return attributes_GetBlob(
        &raw mut (*media_type).attributes,
        key,
        buf,
        bufsize,
        blobsize,
    );
}

unsafe extern "system" fn mediatype_GetAllocatedBlob(iface: *mut IMFMediaType, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_GetAllocatedBlob(core::ptr::addr_of_mut!((*media_type).attributes), key, buf, size);
}

unsafe extern "system" fn mediatype_GetUnknown(iface: *mut IMFMediaType, key: REFGUID, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const core::ffi::c_char,
            b"mediatype_GetUnknown\0".as_ptr() as *const core::ffi::c_char,
            356 as core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_guid(riid as *const _GUID),
            obj,
        );
    }
    return attributes_GetUnknown(core::ptr::addr_of_mut!((*media_type).attributes), key, riid, obj);
}
unsafe extern "system" fn mediatype_SetItem(iface: *mut IMFMediaType, key: REFGUID, value: REFPROPVARIANT) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_SetItem(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}
unsafe extern "system" fn mediatype_DeleteItem(
    mut iface: *mut IMFMediaType,
    key: *const GUID,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mediatype_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            374 as ::core::ffi::c_int,
            b"%p, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
        );
    }
    return attributes_DeleteItem(core::ptr::addr_of_mut!((*media_type).attributes), key);
}

unsafe extern "system" fn mediatype_DeleteAllItems(iface: *mut IMFMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    TRACE!("%p.\n", iface);

    return attributes_DeleteAllItems(core::ptr::addr_of_mut!((*media_type).attributes));
}

unsafe extern "system" fn mediatype_SetUINT32(
    iface: *mut IMFMediaType,
    key: *const GUID,
    value: UINT32,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mediatype_SetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            392 as ::core::ffi::c_int,
            b"%p, %s, %u.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_SetUINT32(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}
unsafe extern "system" fn mediatype_SetUINT64(
    mut iface: *mut IMFMediaType,
    key: *const GUID,
    mut value: UINT64,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"mediatype_SetUINT64\0" as *const u8 as *const i8,
            401 as i32,
            b"%p, %s, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            wine_dbgstr_longlong(value as u64),
        );
    }
    return attributes_SetUINT64(
        core::ptr::addr_of_mut!((*media_type).attributes),
        key,
        value,
    );
}

unsafe extern "system" fn mediatype_SetDouble(iface: *mut IMFMediaType, key: *const GUID, value: f64) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0" as *const u8 as *const i8,
            b"mediatype_SetDouble\0" as *const u8 as *const i8,
            410i32,
            b"%p, %s, %f.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_SetDouble(&raw mut (*media_type).attributes, key, value);
}
unsafe extern "system" fn mediatype_SetGUID(
    mut iface: *mut IMFMediaType,
    key: *const GUID,
    value: *const GUID,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"mediatype_SetGUID\0" as *const u8 as *const i8,
            419 as i32,
            b"%p, %s, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_mf_guid(value),
        );
    }
    return attributes_SetGUID(
        core::ptr::addr_of_mut!((*media_type).attributes),
        key,
        value,
    );
}
unsafe extern "system" fn mediatype_SetString(iface: *mut IMFMediaType, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    TRACE!("%p, %s, %s.\n", iface, debugstr_attr(key), debugstr_w(value));
    return attributes_SetString(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}
unsafe extern "system" fn mediatype_SetBlob(iface: *mut IMFMediaType, key: REFGUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_SetBlob(core::ptr::addr_of_mut!((*media_type).attributes), key, buf, size);
}

unsafe extern "system" fn mediatype_SetUnknown(iface: *mut IMFMediaType, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), unknown);
    return attributes_SetUnknown(core::ptr::addr_of_mut!((*media_type).attributes), key, unknown);
}

unsafe extern "system" fn mediatype_LockStore(iface: *mut IMFMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                .as_ptr() as *const i8,
            b"mediatype_LockStore\0".as_ptr() as *const i8,
            455i32,
            b"%p.\n\0".as_ptr() as *const i8,
            iface,
        );
    }
    return attributes_LockStore(core::ptr::addr_of_mut!((*media_type).attributes));
}

unsafe extern "system" fn mediatype_UnlockStore(mut iface: *mut IMFMediaType) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mediatype_UnlockStore\0" as *const u8 as *const ::core::ffi::c_char,
            464 as ::core::ffi::c_int,
            b"%p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
        );
    }
    return attributes_UnlockStore(core::ptr::addr_of_mut!((*media_type).attributes));
}

unsafe extern "system" fn mediatype_GetCount(iface: *mut IMFMediaType, count: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    return attributes_GetCount(core::ptr::addr_of_mut!((*media_type).attributes), count);
}

unsafe extern "system" fn mediatype_GetItemByIndex(iface: *mut IMFMediaType, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    TRACE!("%p, %u, %p, %p.\n", iface, index, key, value);

    return attributes_GetItemByIndex(core::ptr::addr_of_mut!((*media_type).attributes), index, key, value);
}

unsafe extern "system" fn mediatype_CopyAllItems(iface: *mut IMFMediaType, dest: *mut IMFAttributes) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);

    TRACE!("%p, %p.\n", iface, dest);

    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*media_type).attributes), dest);
}

unsafe extern "system" fn mediatype_GetMajorType(
    iface: *mut IMFMediaType,
    guid: *mut GUID,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"mediatype_GetMajorType\0" as *const u8 as *const i8,
            500 as i32,
            b"%p, %p.\n\0" as *const u8 as *const i8,
            iface,
            guid,
        );
    }
    return attributes_GetGUID(
        &raw mut (*media_type).attributes,
        &raw const MF_MT_MAJOR_TYPE,
        guid,
    );
}

unsafe extern "C" fn mediatype_is_compressed(media_type: *mut media_type, compressed: *mut BOOL) -> HRESULT {
    let mut value: UINT32 = 0;
    if (attributes_GetUINT32(
        core::ptr::addr_of_mut!((*media_type).attributes),
        core::ptr::addr_of!(MF_MT_ALL_SAMPLES_INDEPENDENT),
        core::ptr::addr_of_mut!(value),
    )) < 0
    {
        value = 0;
    }

    *compressed = (value == 0) as BOOL;

    return S_OK;
}

unsafe extern "system" fn mediatype_IsCompressedFormat(iface: *mut IMFMediaType, compressed: *mut BOOL) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"mediatype_IsCompressedFormat\0".as_ptr() as *const i8,
            523 as i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface,
            compressed,
        );
    }
    return mediatype_is_compressed(media_type, compressed);
}

unsafe extern "C" fn media_type_is_equal(media_type: *mut media_type, type_0: *mut IMFMediaType, flags: *mut DWORD) -> HRESULT {
    let full_equality_flags: DWORD = (MF_MEDIATYPE_EQUAL_MAJOR_TYPES as DWORD)
        | (MF_MEDIATYPE_EQUAL_FORMAT_TYPES as DWORD)
        | (MF_MEDIATYPE_EQUAL_FORMAT_DATA as DWORD)
        | (MF_MEDIATYPE_EQUAL_FORMAT_USER_DATA as DWORD);
    let mut left: comparand = core::mem::zeroed();
    let mut right: comparand = core::mem::zeroed();
    let mut swp: comparand = core::mem::zeroed();
    let mut i: ::core::ffi::c_uint;
    let mut result: BOOL;

    *flags = 0 as DWORD;

    left.type_ = &raw mut (*media_type).attributes.IMFAttributes_iface;
    right.type_ = type_0 as *mut IMFAttributes;

    if IMFAttributes_GetGUID(left.type_, &raw const MF_MT_MAJOR_TYPE, &raw mut left.guid) < 0 {
        return E_INVALIDARG;
    }

    if IMFAttributes_GetGUID(right.type_, &raw const MF_MT_MAJOR_TYPE, &raw mut right.guid) < 0 {
        return E_INVALIDARG;
    }

    if IsEqualGUID!(&raw mut left.guid, &raw mut right.guid) != false {
        *flags |= MF_MEDIATYPE_EQUAL_MAJOR_TYPES as DWORD;
    }

    /* Subtypes equal or both missing. */
    left.hr = IMFAttributes_GetGUID(left.type_, &raw const MF_MT_SUBTYPE, &raw mut left.guid);
    right.hr = IMFAttributes_GetGUID(right.type_, &raw const MF_MT_SUBTYPE, &raw mut right.guid);

    if ((left.hr >= 0)
        && (right.hr >= 0)
        && IsEqualGUID!(&raw mut left.guid, &raw mut right.guid) != false)
        || ((left.hr < 0) && (right.hr < 0))
    {
        *flags |= MF_MEDIATYPE_EQUAL_FORMAT_TYPES as DWORD;
    }

    /* Format data */
    IMFAttributes_GetCount(left.type_, &raw mut left.count);
    IMFAttributes_GetCount(right.type_, &raw mut right.count);

    if right.count < left.count {
        swp = left;
        left = right;
        right = swp;
    }

    *flags |= MF_MEDIATYPE_EQUAL_FORMAT_DATA as DWORD;

    i = 0 as ::core::ffi::c_uint;
    while i < left.count {
        let mut value: PROPVARIANT = core::mem::zeroed();
        let mut key: GUID = core::mem::zeroed();

        if IMFAttributes_GetItemByIndex(left.type_, i as UINT32, &raw mut key, &raw mut value) >= 0 {
            if IsEqualGUID!(&raw mut key, &raw const MF_MT_USER_DATA) != false
                || IsEqualGUID!(&raw mut key, &raw const MF_MT_FRAME_RATE_RANGE_MIN) != false
                || IsEqualGUID!(&raw mut key, &raw const MF_MT_FRAME_RATE_RANGE_MAX) != false
            {
                PropVariantClear(&raw mut value);
            } else {
                result = (FALSE as i32);
                IMFAttributes_CompareItem(right.type_, &raw mut key, &raw mut value, &raw mut result);
                PropVariantClear(&raw mut value);
                if result == 0 {
                    *flags &= !(MF_MEDIATYPE_EQUAL_FORMAT_DATA as DWORD);
                    break;
                }
            }
        }
        i = i.wrapping_add(1);
    }

    /* User data */
    PropVariantInit!(&raw mut left.value);
    left.hr = IMFAttributes_GetItem(left.type_, &raw const MF_MT_USER_DATA, &raw mut left.value);
    PropVariantInit!(&raw mut right.value);
    right.hr = IMFAttributes_GetItem(right.type_, &raw const MF_MT_USER_DATA, &raw mut right.value);

    /* Compare user data if both types have it, otherwise simply check if both don't. */
    if (left.hr >= 0) && (right.hr >= 0) {
        result = (FALSE as i32);
        IMFAttributes_CompareItem(left.type_, &raw const MF_MT_USER_DATA, &raw mut left.value, &raw mut result);
    } else {
        result = ((left.hr < 0) && (right.hr < 0)) as BOOL;
    }

    PropVariantClear(&raw mut left.value);
    PropVariantClear(&raw mut right.value);

    if result != 0 {
        *flags |= MF_MEDIATYPE_EQUAL_FORMAT_USER_DATA as DWORD;
    }

    return if *flags == full_equality_flags {
        S_OK
    } else {
        S_FALSE
    };
}
unsafe extern "system" fn mediatype_IsEqual(
    mut iface: *mut IMFMediaType,
    mut type_0: *mut IMFMediaType,
    mut flags: *mut DWORD,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mediatype_IsEqual\0" as *const u8 as *const ::core::ffi::c_char,
            634 as ::core::ffi::c_int,
            b"%p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            type_0,
            flags,
        );
    }
    return media_type_is_equal(media_type, type_0, flags);
}

unsafe extern "system" fn mediatype_GetRepresentation(iface: *mut IMFMediaType, mut guid: GUID, representation: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(&raw mut guid, &raw const AM_MEDIA_TYPE_REPRESENTATION) != false {
        return MFCreateAMMediaTypeFromMFMediaType(iface, GUID_NULL, representation as *mut *mut AM_MEDIA_TYPE);
    }

    if IsEqualGUID!(&raw mut guid, &raw const FORMAT_WaveFormatEx) != false
        || IsEqualGUID!(&raw mut guid, &raw const FORMAT_VideoInfo) != false
        || IsEqualGUID!(&raw mut guid, &raw const FORMAT_VideoInfo2) != false
        || IsEqualGUID!(&raw mut guid, &raw const FORMAT_MFVideoFormat) != false
    {
        return MFCreateAMMediaTypeFromMFMediaType(iface, guid, representation as *mut *mut AM_MEDIA_TYPE);
    }

    return MF_E_UNSUPPORTED_REPRESENTATION;
}

unsafe extern "system" fn mediatype_FreeRepresentation(
    mut iface: *mut IMFMediaType,
    mut guid: GUID,
    mut representation: *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut am_type: *mut AM_MEDIA_TYPE = representation as *mut AM_MEDIA_TYPE;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mediatype_FreeRepresentation\0" as *const u8 as *const ::core::ffi::c_char,
            660 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(core::ptr::addr_of_mut!(guid)),
            representation,
        );
    }
    CoTaskMemFree((*am_type).pbFormat as LPVOID);
    CoTaskMemFree(am_type as LPVOID);
    return S_OK;
}

unsafe extern "system" fn video_mediatype_QueryInterface(iface: *mut IMFVideoMediaType, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    return IMFMediaType_QueryInterface(core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface), riid, out);
}

unsafe extern "system" fn video_mediatype_AddRef(iface: *mut IMFVideoMediaType) -> ULONG {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    return IMFMediaType_AddRef(core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface));
}

unsafe extern "system" fn video_mediatype_Release(iface: *mut IMFVideoMediaType) -> ULONG {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    return IMFMediaType_Release(core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface));
}
unsafe extern "system" fn video_mediatype_GetItem(iface: *mut IMFVideoMediaType, key: REFGUID, value: *mut PROPVARIANT) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);

    return attributes_GetItem(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_GetItemType(
    mut iface: *mut IMFVideoMediaType,
    key: *const GUID,
    mut type_0: *mut MF_ATTRIBUTE_TYPE,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mediatype_GetItemType\0" as *const u8 as *const ::core::ffi::c_char,
            740 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            type_0,
        );
    }
    return attributes_GetItemType(&raw mut (*media_type).attributes, key, type_0);
}

unsafe extern "system" fn video_mediatype_CompareItem(
    iface: *mut IMFVideoMediaType,
    key: *const GUID,
    value: *const PROPVARIANT,
    result: *mut BOOL,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"video_mediatype_CompareItem\0" as *const u8 as *const i8,
            749i32,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return attributes_CompareItem(&raw mut (*media_type).attributes, key, value, result);
}

unsafe extern "system" fn video_mediatype_Compare(
    iface: *mut IMFVideoMediaType,
    attrs: *mut IMFAttributes,
    type_: MF_ATTRIBUTES_MATCH_TYPE,
    result: *mut BOOL,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_Compare(core::ptr::addr_of_mut!((*media_type).attributes), attrs, type_, result);
}

unsafe extern "system" fn video_mediatype_GetUINT32(
    iface: *mut IMFVideoMediaType,
    key: *const GUID,
    value: *mut UINT32,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"video_mediatype_GetUINT32\0" as *const u8 as *const i8,
            768 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT32(&raw mut (*media_type).attributes, key, value);
}

unsafe extern "system" fn video_mediatype_GetUINT64(iface: *mut IMFVideoMediaType, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);

    return attributes_GetUINT64(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_GetDouble(iface: *mut IMFVideoMediaType, key: *const GUID, value: *mut f64) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_GetDouble(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_GetGUID(iface: *mut IMFVideoMediaType, key: *const GUID, value: *mut GUID) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_GetGUID(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_GetStringLength(iface: *mut IMFVideoMediaType, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"video_mediatype_GetStringLength\0" as *const u8 as *const core::ffi::c_char,
            804i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            length,
        );
    }
    return attributes_GetStringLength(core::ptr::addr_of_mut!((*media_type).attributes), key, length);
}

unsafe extern "system" fn video_mediatype_GetString(iface: *mut IMFVideoMediaType, key: *const GUID, value: *mut WCHAR, size: UINT32, length: *mut UINT32) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_GetString(core::ptr::addr_of_mut!((*media_type).attributes), key, value, size, length);
}

unsafe extern "system" fn video_mediatype_GetAllocatedString(
    iface: *mut IMFVideoMediaType,
    key: *const GUID,
    value: *mut *mut WCHAR,
    length: *mut UINT32,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    TRACE!(
        "%p, %s, %p, %p.\n",
        iface,
        debugstr_attr(key),
        value,
        length
    );

    return attributes_GetAllocatedString(core::ptr::addr_of_mut!((*media_type).attributes), key, value, length);
}

unsafe extern "system" fn video_mediatype_GetBlobSize(
    iface: *mut IMFVideoMediaType,
    key: *const GUID,
    size: *mut UINT32,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"video_mediatype_GetBlobSize\0" as *const u8 as *const i8,
            833 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            size,
        );
    }
    return attributes_GetBlobSize(core::ptr::addr_of_mut!((*media_type).attributes), key, size);
}
unsafe extern "system" fn video_mediatype_GetBlob(
    mut iface: *mut IMFVideoMediaType,
    key: *const GUID,
    mut buf: *mut UINT8,
    mut bufsize: UINT32,
    mut blobsize: *mut UINT32,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mediatype_GetBlob\0" as *const u8 as *const ::core::ffi::c_char,
            843 as ::core::ffi::c_int,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            buf,
            bufsize,
            blobsize,
        );
    }
    return attributes_GetBlob(
        core::ptr::addr_of_mut!((*media_type).attributes),
        key,
        buf,
        bufsize,
        blobsize,
    );
}

unsafe extern "system" fn video_mediatype_GetAllocatedBlob(iface: *mut IMFVideoMediaType, key: *const GUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_GetAllocatedBlob(core::ptr::addr_of_mut!((*media_type).attributes), key, buf, size);
}

unsafe extern "system" fn video_mediatype_GetUnknown(iface: *mut IMFVideoMediaType, key: REFGUID, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_GetUnknown(core::ptr::addr_of_mut!((*media_type).attributes), key, riid, obj);
}

unsafe extern "system" fn video_mediatype_SetItem(iface: *mut IMFVideoMediaType, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_SetItem(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_DeleteItem(iface: *mut IMFVideoMediaType, key: *const GUID) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_DeleteItem(core::ptr::addr_of_mut!((*media_type).attributes), key);
}

unsafe extern "system" fn video_mediatype_DeleteAllItems(iface: *mut IMFVideoMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    TRACE!("%p.\n", iface);
    return attributes_DeleteAllItems(core::ptr::addr_of_mut!((*media_type).attributes));
}
unsafe extern "system" fn video_mediatype_SetUINT32(
    iface: *mut IMFVideoMediaType,
    key: *const GUID,
    value: UINT32,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"video_mediatype_SetUINT32\0" as *const u8 as *const core::ffi::c_char,
            897i32,
            b"%p, %s, %u.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_SetUINT32(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_SetUINT64(
    iface: *mut IMFVideoMediaType,
    key: *const GUID,
    value: UINT64,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"video_mediatype_SetUINT64\0".as_ptr() as *const ::core::ffi::c_char,
            906 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0".as_ptr() as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return attributes_SetUINT64(&raw mut (*media_type).attributes, key, value);
}

unsafe extern "system" fn video_mediatype_SetDouble(iface: *mut IMFVideoMediaType, key: *const GUID, value: f64) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    TRACE!("%p, %s, %f.\n", iface, debugstr_attr(key), value);
    return attributes_SetDouble(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}
unsafe extern "system" fn video_mediatype_SetGUID(iface: *mut IMFVideoMediaType, key: *const GUID, value: *const GUID) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"video_mediatype_SetGUID\0".as_ptr() as *const i8,
            924i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_mf_guid(value),
        );
    }
    return attributes_SetGUID(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_SetString(iface: *mut IMFVideoMediaType, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"video_mediatype_SetString\0".as_ptr() as *const i8,
            933i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_w(value),
        );
    }
    return attributes_SetString(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn video_mediatype_SetBlob(iface: *mut IMFVideoMediaType, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    TRACE!("%p, %s, %p, %u.\n", iface, debugstr_attr(key), buf, size);

    return attributes_SetBlob(&raw mut (*media_type).attributes, key, buf, size);
}

unsafe extern "system" fn video_mediatype_SetUnknown(iface: *mut IMFVideoMediaType, key: REFGUID, unknown: *mut IUnknown) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_SetUnknown(core::ptr::addr_of_mut!((*media_type).attributes), key, unknown);
}

unsafe extern "system" fn video_mediatype_LockStore(iface: *mut IMFVideoMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    TRACE!("%p.\n", iface);

    return attributes_LockStore(core::ptr::addr_of_mut!((*media_type).attributes));
}

unsafe extern "system" fn video_mediatype_UnlockStore(iface: *mut IMFVideoMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_UnlockStore(core::ptr::addr_of_mut!((*media_type).attributes));
}
unsafe extern "system" fn video_mediatype_GetCount(iface: *mut IMFVideoMediaType, count: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    TRACE!("%p, %p.\n", iface, count);

    return attributes_GetCount(core::ptr::addr_of_mut!((*media_type).attributes), count);
}
unsafe extern "system" fn video_mediatype_GetItemByIndex(
    mut iface: *mut IMFVideoMediaType,
    mut index: UINT32,
    mut key: *mut GUID,
    mut value: *mut PROPVARIANT,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mediatype_GetItemByIndex\0" as *const u8 as *const ::core::ffi::c_char,
            987 as ::core::ffi::c_int,
            b"%p, %u, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            key,
            value,
        );
    }
    return attributes_GetItemByIndex(&raw mut (*media_type).attributes, index, key, value);
}

unsafe extern "system" fn video_mediatype_CopyAllItems(iface: *mut IMFVideoMediaType, dest: *mut IMFAttributes) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);

    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*media_type).attributes), dest);
}

unsafe extern "system" fn video_mediatype_GetMajorType(iface: *mut IMFVideoMediaType, guid: *mut GUID) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    return attributes_GetGUID(core::ptr::addr_of_mut!((*media_type).attributes), core::ptr::addr_of!(MF_MT_MAJOR_TYPE), guid);
}

unsafe extern "system" fn video_mediatype_IsCompressedFormat(
    iface: *mut IMFVideoMediaType,
    compressed: *mut BOOL,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    TRACE!("%p, %p.\n", iface, compressed);
    return mediatype_is_compressed(media_type, compressed);
}

unsafe extern "system" fn video_mediatype_IsEqual(iface: *mut IMFVideoMediaType, type_0: *mut IMFMediaType, flags: *mut DWORD) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    TRACE!("%p, %p, %p.\n", iface, type_0, flags);
    return media_type_is_equal(media_type, type_0, flags);
}
unsafe extern "system" fn video_mediatype_GetRepresentation(iface: *mut IMFVideoMediaType, mut guid: GUID, representation: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(core::ptr::addr_of_mut!(guid) as *const GUID, core::ptr::addr_of!(FORMAT_WaveFormatEx)) != false {
        return MF_E_UNSUPPORTED_REPRESENTATION;
    }

    return mediatype_GetRepresentation(iface as *mut IMFMediaType, guid, representation);
}
unsafe extern "system" fn video_mediatype_FreeRepresentation(
    mut iface: *mut IMFVideoMediaType,
    mut guid: GUID,
    mut representation: *mut ::core::ffi::c_void,
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
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mediatype_FreeRepresentation\0" as *const u8 as *const ::core::ffi::c_char,
            1040 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(&raw mut guid),
            representation,
        );
    }
    return mediatype_FreeRepresentation(iface as *mut IMFMediaType, guid, representation);
}
unsafe extern "system" fn video_mediatype_GetVideoFormat(iface: *mut IMFVideoMediaType) -> *const MFVIDEOFORMAT {
    let mut media_type: *mut media_type = impl_from_IMFVideoMediaType(iface);
    let mut size: UINT32 = 0;
    let mut hr: HRESULT = 0;

    CoTaskMemFree((*media_type).video_format as LPVOID);
    (*media_type).video_format = core::ptr::null_mut();
    hr = MFCreateMFVideoFormatFromMFMediaType(
        core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface),
        core::ptr::addr_of_mut!((*media_type).video_format),
        core::ptr::addr_of_mut!(size),
    );
    if hr < 0 {
        WARN!("Failed to create format description, hr %#lx.\n", hr);
    }

    return (*media_type).video_format;
}
unsafe extern "system" fn video_mediatype_GetVideoRepresentation(
    iface: *mut IMFVideoMediaType,
    representation: GUID,
    data: *mut *mut core::ffi::c_void,
    stride: LONG,
) -> HRESULT {
    // FIXME("%p, %s, %p, %ld.\n", iface, debugstr_guid(&representation), data, stride);
    E_NOTIMPL
}

unsafe extern "system" fn audio_mediatype_QueryInterface(iface: *mut IMFAudioMediaType, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    return IMFMediaType_QueryInterface(core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface), riid, out);
}

unsafe extern "system" fn audio_mediatype_AddRef(iface: *mut IMFAudioMediaType) -> ULONG {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    return IMFMediaType_AddRef(core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface));
}

unsafe extern "system" fn audio_mediatype_Release(iface: *mut IMFAudioMediaType) -> ULONG {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    return IMFMediaType_Release(core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface));
}

unsafe extern "system" fn audio_mediatype_GetItem(
    iface: *mut IMFAudioMediaType,
    key: *const GUID,
    value: *mut PROPVARIANT,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"audio_mediatype_GetItem\0".as_ptr() as *const i8,
            1134i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetItem(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn audio_mediatype_GetItemType(iface: *mut IMFAudioMediaType, key: *const GUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"audio_mediatype_GetItemType\0" as *const u8 as *const ::core::ffi::c_char,
            1143 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            type_,
        );
    }
    return attributes_GetItemType(core::ptr::addr_of_mut!((*media_type).attributes), key, type_);
}

unsafe extern "system" fn audio_mediatype_CompareItem(iface: *mut IMFAudioMediaType, key: REFGUID, value: REFPROPVARIANT, result: *mut BOOL) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    TRACE!("%p, %s, %s, %p.\n", iface, debugstr_attr(key), debugstr_propvar(value), result);
    return attributes_CompareItem(core::ptr::addr_of_mut!((*media_type).attributes), key, value, result);
}
unsafe extern "system" fn audio_mediatype_Compare(
    iface: *mut IMFAudioMediaType,
    attrs: *mut IMFAttributes,
    type_0: MF_ATTRIBUTES_MATCH_TYPE,
    result: *mut BOOL,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"audio_mediatype_Compare\0" as *const u8 as *const i8,
            1162i32,
            b"%p, %p, %d, %p.\n\0" as *const u8 as *const i8,
            iface,
            attrs,
            type_0 as u32,
            result,
        );
    }
    return attributes_Compare(
        core::ptr::addr_of_mut!((*media_type).attributes),
        attrs,
        type_0,
        result,
    );
}

unsafe extern "system" fn audio_mediatype_GetUINT32(iface: *mut IMFAudioMediaType, key: *const GUID, value: *mut UINT32) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"audio_mediatype_GetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            1171 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT32(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn audio_mediatype_GetUINT64(iface: *mut IMFAudioMediaType, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_GetUINT64(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn audio_mediatype_GetDouble(iface: *mut IMFAudioMediaType, key: REFGUID, value: *mut f64) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_GetDouble(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn audio_mediatype_GetGUID(
    iface: *mut IMFAudioMediaType,
    key: *const GUID,
    value: *mut GUID,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"audio_mediatype_GetGUID\0" as *const u8 as *const i8,
            1198 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetGUID(
        core::ptr::addr_of_mut!((*media_type).attributes),
        key,
        value,
    );
}

unsafe extern "system" fn audio_mediatype_GetStringLength(iface: *mut IMFAudioMediaType, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), length);

    return attributes_GetStringLength(core::ptr::addr_of_mut!((*media_type).attributes), key, length);
}

unsafe extern "system" fn audio_mediatype_GetString(
    iface: *mut IMFAudioMediaType,
    key: *const GUID,
    value: *mut WCHAR,
    size: UINT32,
    length: *mut UINT32,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"audio_mediatype_GetString\0" as *const u8 as *const ::core::ffi::c_char,
            1217 as ::core::ffi::c_int,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
            size,
            length,
        );
    }
    return attributes_GetString(&raw mut (*media_type).attributes, key, value, size, length);
}

unsafe extern "system" fn audio_mediatype_GetAllocatedString(iface: *mut IMFAudioMediaType, key: REFGUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_GetAllocatedString(&raw mut (*media_type).attributes, key, value, length);
}
unsafe extern "system" fn audio_mediatype_GetBlobSize(iface: *mut IMFAudioMediaType, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), size);

    return attributes_GetBlobSize(core::ptr::addr_of_mut!((*media_type).attributes), key, size);
}

unsafe extern "system" fn audio_mediatype_GetBlob(iface: *mut IMFAudioMediaType, key: *const GUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"audio_mediatype_GetBlob\0" as *const u8 as *const i8,
            1246i32,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            buf,
            bufsize,
            blobsize,
        );
    }
    return attributes_GetBlob(
        core::ptr::addr_of_mut!((*media_type).attributes),
        key,
        buf,
        bufsize,
        blobsize,
    );
}

unsafe extern "system" fn audio_mediatype_GetAllocatedBlob(iface: *mut IMFAudioMediaType, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_GetAllocatedBlob(core::ptr::addr_of_mut!((*media_type).attributes), key, buf, size);
}

unsafe extern "system" fn audio_mediatype_GetUnknown(
    mut iface: *mut IMFAudioMediaType,
    key: *const GUID,
    riid: *const IID,
    mut obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"audio_mediatype_GetUnknown\0" as *const u8 as *const core::ffi::c_char,
            1264 as core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_guid(riid as *const _GUID),
            obj,
        );
    }
    return attributes_GetUnknown(
        core::ptr::addr_of_mut!((*media_type).attributes),
        key,
        riid,
        obj,
    );
}
unsafe extern "system" fn audio_mediatype_SetItem(
    iface: *mut IMFAudioMediaType,
    key: *const GUID,
    value: *const PROPVARIANT,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"audio_mediatype_SetItem\0" as *const u8 as *const i8,
            1273i32,
            b"%p, %s, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
        );
    }
    return attributes_SetItem(
        core::ptr::addr_of_mut!((*media_type).attributes),
        key,
        value,
    );
}

unsafe extern "system" fn audio_mediatype_DeleteItem(iface: *mut IMFAudioMediaType, key: *const GUID) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"audio_mediatype_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            1282 as ::core::ffi::c_int,
            b"%p, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
        );
    }
    return attributes_DeleteItem(core::ptr::addr_of_mut!((*media_type).attributes), key);
}

unsafe extern "system" fn audio_mediatype_DeleteAllItems(iface: *mut IMFAudioMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_DeleteAllItems(core::ptr::addr_of_mut!((*media_type).attributes));
}

unsafe extern "system" fn audio_mediatype_SetUINT32(
    mut iface: *mut IMFAudioMediaType,
    key: *const GUID,
    mut value: UINT32,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"audio_mediatype_SetUINT32\0" as *const u8 as *const i8,
            1300 as i32,
            b"%p, %s, %u.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_SetUINT32(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn audio_mediatype_SetUINT64(
    mut iface: *mut IMFAudioMediaType,
    key: *const GUID,
    mut value: UINT64,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"audio_mediatype_SetUINT64\0" as *const u8 as *const ::core::ffi::c_char,
            1309 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return attributes_SetUINT64(&raw mut (*media_type).attributes, key, value);
}

unsafe extern "system" fn audio_mediatype_SetDouble(iface: *mut IMFAudioMediaType, key: *const GUID, value: f64) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_SetDouble(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn audio_mediatype_SetGUID(
    mut iface: *mut IMFAudioMediaType,
    key: *const GUID,
    value: *const GUID,
) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    TRACE!(
        "%p, %s, %s.\n",
        iface,
        debugstr_attr(key),
        debugstr_mf_guid(value)
    );
    return attributes_SetGUID(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}
unsafe extern "system" fn audio_mediatype_SetString(iface: *mut IMFAudioMediaType, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let mut media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    TRACE!("%p, %s, %s.\n", iface, debugstr_attr(key), debugstr_w(value));
    return attributes_SetString(core::ptr::addr_of_mut!((*media_type).attributes), key, value);
}

unsafe extern "system" fn audio_mediatype_SetBlob(iface: *mut IMFAudioMediaType, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    TRACE!("%p, %s, %p, %u.\n", iface, debugstr_attr(key), buf, size);

    return attributes_SetBlob(core::ptr::addr_of_mut!((*media_type).attributes), key, buf, size);
}

unsafe extern "system" fn audio_mediatype_SetUnknown(iface: *mut IMFAudioMediaType, key: REFGUID, unknown: *mut IUnknown) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_SetUnknown(core::ptr::addr_of_mut!((*media_type).attributes), key, unknown);
}

unsafe extern "system" fn audio_mediatype_LockStore(iface: *mut IMFAudioMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_LockStore(core::ptr::addr_of_mut!((*media_type).attributes));
}

unsafe extern "system" fn audio_mediatype_UnlockStore(iface: *mut IMFAudioMediaType) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    TRACE!("%p.\n", iface);

    return attributes_UnlockStore(core::ptr::addr_of_mut!((*media_type).attributes));
}

unsafe extern "system" fn audio_mediatype_GetCount(
    iface: *mut IMFAudioMediaType,
    count: *mut UINT32,
) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"audio_mediatype_GetCount\0" as *const u8 as *const i8,
            1381i32,
            b"%p, %p.\n\0" as *const u8 as *const i8,
            iface,
            count,
        );
    }
    return attributes_GetCount(core::ptr::addr_of_mut!((*media_type).attributes), count);
}

unsafe extern "system" fn audio_mediatype_GetItemByIndex(iface: *mut IMFAudioMediaType, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    TRACE!("%p, %u, %p, %p.\n", iface, index, key, value);
    return attributes_GetItemByIndex(core::ptr::addr_of_mut!((*media_type).attributes), index, key, value);
}

unsafe extern "system" fn audio_mediatype_CopyAllItems(iface: *mut IMFAudioMediaType, dest: *mut IMFAttributes) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*media_type).attributes), dest);
}

unsafe extern "system" fn audio_mediatype_GetMajorType(iface: *mut IMFAudioMediaType, guid: *mut GUID) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    TRACE!("%p, %p.\n", iface, guid);

    return attributes_GetGUID(&raw mut (*media_type).attributes, &MF_MT_MAJOR_TYPE, guid);
}
unsafe extern "system" fn audio_mediatype_IsCompressedFormat(iface: *mut IMFAudioMediaType, compressed: *mut BOOL) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"audio_mediatype_IsCompressedFormat\0".as_ptr() as *const i8,
            1417 as i32,
            b"%p, %p.\n\0".as_ptr() as *const i8,
            iface,
            compressed,
        );
    }
    return mediatype_is_compressed(media_type, compressed);
}

unsafe extern "system" fn audio_mediatype_IsEqual(iface: *mut IMFAudioMediaType, type_: *mut IMFMediaType, flags: *mut DWORD) -> HRESULT {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);

    return media_type_is_equal(media_type, type_, flags);
}

unsafe extern "system" fn audio_mediatype_GetRepresentation(iface: *mut IMFAudioMediaType, mut guid: GUID, representation: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(&raw mut guid, &raw const FORMAT_VideoInfo)
        != false
        || IsEqualGUID!(&raw mut guid, &raw const FORMAT_VideoInfo2)
            != false
        || IsEqualGUID!(&raw mut guid, &raw const FORMAT_MFVideoFormat)
            != false
    {
        return E_INVALIDARG;
    }

    return mediatype_GetRepresentation(iface as *mut IMFMediaType, guid, representation);
}
unsafe extern "system" fn audio_mediatype_FreeRepresentation(
    iface: *mut IMFAudioMediaType,
    guid: GUID,
    representation: *mut core::ffi::c_void,
) -> HRESULT {
    let mut iface = iface;
    let mut guid = guid;
    let mut representation = representation;
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"audio_mediatype_FreeRepresentation\0" as *const u8 as *const core::ffi::c_char,
            1445i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_guid(core::ptr::addr_of_mut!(guid)),
            representation,
        );
    }
    return mediatype_FreeRepresentation(iface as *mut IMFMediaType, guid, representation);
}
unsafe extern "system" fn audio_mediatype_GetAudioFormat(
    iface: *mut IMFAudioMediaType,
) -> *const WAVEFORMATEX {
    let media_type: *mut media_type = impl_from_IMFAudioMediaType(iface);
    let mut size: u32 = 0;
    let mut hr: HRESULT = 0;

    TRACE!("%p.\n", iface);

    CoTaskMemFree((*media_type).audio_format as LPVOID);
    (*media_type).audio_format = core::ptr::null_mut::<WAVEFORMATEX>();
    hr = MFCreateWaveFormatExFromMFMediaType(
        core::ptr::addr_of_mut!((*media_type).IMFMediaType_iface),
        core::ptr::addr_of_mut!((*media_type).audio_format),
        core::ptr::addr_of_mut!(size),
        MFWaveFormatExConvertFlag_Normal as UINT32,
    );
    if hr < 0 {
        WARN!(
            "Failed to create wave format description, hr %#lx.\n",
            hr
        );
    }

    return (*media_type).audio_format as *const WAVEFORMATEX;
}

unsafe extern "C" fn create_media_type(ret: *mut *mut media_type) -> HRESULT {
    let mut object: *mut media_type;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<media_type>()) as _) as *mut media_type;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(core::ptr::addr_of_mut!((*object).attributes), 0);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }
    (*object).IMFMediaType_iface.lpVtbl = core::ptr::addr_of!(mediatypevtbl);
    (*object).IMFVideoMediaType_iface.lpVtbl = core::ptr::addr_of!(videomediatypevtbl);
    (*object).IMFAudioMediaType_iface.lpVtbl = core::ptr::addr_of!(audiomediatypevtbl);

    *ret = object;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMediaType(media_type: *mut *mut IMFMediaType) -> HRESULT {
    let mut object: *mut media_type = ::core::ptr::null_mut::<media_type>();
    let mut hr: HRESULT = 0;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFCreateMediaType\0" as *const u8 as *const ::core::ffi::c_char,
            1541 as ::core::ffi::c_int,
            b"%p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            media_type,
        );
    }
    if media_type.is_null() {
        return E_INVALIDARG;
    }
    hr = create_media_type(core::ptr::addr_of_mut!(object));
    if hr < 0 {
        return hr;
    }
    *media_type = core::ptr::addr_of_mut!((*object).IMFMediaType_iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl_0) as u32,
            __dbch_0,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFCreateMediaType\0" as *const u8 as *const ::core::ffi::c_char,
            1551 as ::core::ffi::c_int,
            b"Created media type %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            *media_type,
        );
    }
    return S_OK;
}

unsafe extern "system" fn stream_descriptor_QueryInterface(iface: *mut IMFStreamDescriptor, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFStreamDescriptor)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = iface as *mut core::ffi::c_void;
        IMFStreamDescriptor_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn stream_descriptor_AddRef(iface: *mut IMFStreamDescriptor) -> ULONG {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*stream_desc).attributes.ref_) as *mut i32) as ULONG;

    return refcount;
}

unsafe extern "system" fn stream_descriptor_Release(iface: *mut IMFStreamDescriptor) -> ULONG {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*stream_desc).attributes.ref_) as *mut ::core::ffi::c_long,
    ) as ULONG;
    let mut i: ::core::ffi::c_uint;

    if refcount == 0 {
        i = 0;
        while i < (*stream_desc).media_types_count {
            if !(*(*stream_desc).media_types.add(i as usize)).is_null() {
                IMFMediaType_Release(*(*stream_desc).media_types.add(i as usize));
            }
            i = i.wrapping_add(1);
        }
        free((*stream_desc).media_types as *mut ::core::ffi::c_void);
        if !(*stream_desc).current_type.is_null() {
            IMFMediaType_Release((*stream_desc).current_type);
        }
        clear_attributes_object(core::ptr::addr_of_mut!((*stream_desc).attributes));
        free(stream_desc as *mut ::core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn stream_descriptor_GetItem(iface: *mut IMFStreamDescriptor, key: *const GUID, value: *mut PROPVARIANT) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);
    return attributes_GetItem(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_GetItemType(iface: *mut IMFStreamDescriptor, key: *const GUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0" as *const u8 as *const i8,
            b"stream_descriptor_GetItemType\0" as *const u8 as *const i8,
            1622 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            type_,
        );
    }
    return attributes_GetItemType(&raw mut (*stream_desc).attributes, key, type_);
}
unsafe extern "system" fn stream_descriptor_CompareItem(
    mut iface: *mut IMFStreamDescriptor,
    key: *const GUID,
    value: *const PROPVARIANT,
    mut result: *mut BOOL,
) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"stream_descriptor_CompareItem\0" as *const u8 as *const ::core::ffi::c_char,
            1632 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return attributes_CompareItem(&raw mut (*stream_desc).attributes, key, value, result);
}

unsafe extern "system" fn stream_descriptor_Compare(
    iface: *mut IMFStreamDescriptor,
    theirs: *mut IMFAttributes,
    type_0: MF_ATTRIBUTES_MATCH_TYPE,
    result: *mut BOOL,
) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_Compare(core::ptr::addr_of_mut!((*stream_desc).attributes), theirs, type_0, result);
}

unsafe extern "system" fn stream_descriptor_GetUINT32(iface: *mut IMFStreamDescriptor, key: REFGUID, value: *mut UINT32) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_GetUINT32(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_GetUINT64(iface: *mut IMFStreamDescriptor, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"stream_descriptor_GetUINT64\0".as_ptr() as *const i8,
            1660 as i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT64(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_GetDouble(iface: *mut IMFStreamDescriptor, key: *const GUID, value: *mut f64) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_GetDouble(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}
unsafe extern "system" fn stream_descriptor_GetGUID(iface: *mut IMFStreamDescriptor, key: *const GUID, value: *mut GUID) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_GetGUID(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_GetStringLength(iface: *mut IMFStreamDescriptor, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), length);
    return attributes_GetStringLength(core::ptr::addr_of_mut!((*stream_desc).attributes), key, length);
}

unsafe extern "system" fn stream_descriptor_GetString(iface: *mut IMFStreamDescriptor, key: REFGUID, value: *mut WCHAR, size: UINT32, length: *mut UINT32) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_GetString(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value, size, length);
}

unsafe extern "system" fn stream_descriptor_GetAllocatedString(
    iface: *mut IMFStreamDescriptor,
    key: *const GUID,
    value: *mut *mut WCHAR,
    length: *mut UINT32,
) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"stream_descriptor_GetAllocatedString\0" as *const u8 as *const i8,
            1707 as i32,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
            length,
        );
    }
    return attributes_GetAllocatedString(
        core::ptr::addr_of_mut!((*stream_desc).attributes),
        key,
        value,
        length,
    );
}

unsafe extern "system" fn stream_descriptor_GetBlobSize(iface: *mut IMFStreamDescriptor, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"stream_descriptor_GetBlobSize\0" as *const u8 as *const ::core::ffi::c_char,
            1716 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            size,
        );
    }
    return attributes_GetBlobSize(core::ptr::addr_of_mut!((*stream_desc).attributes), key, size);
}

unsafe extern "system" fn stream_descriptor_GetBlob(
    iface: *mut IMFStreamDescriptor,
    key: *const GUID,
    buf: *mut UINT8,
    bufsize: UINT32,
    blobsize: *mut UINT32,
) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if (*__wine_dbch___default).flags as i32
        & ((1i32) << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"stream_descriptor_GetBlob\0".as_ptr() as *const i8,
            1726i32,
            b"%p, %s, %p, %u, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            buf,
            bufsize,
            blobsize,
        );
    }
    return attributes_GetBlob(
        core::ptr::addr_of_mut!((*stream_desc).attributes),
        key,
        buf,
        bufsize,
        blobsize,
    );
}

unsafe extern "system" fn stream_descriptor_GetAllocatedBlob(
    iface: *mut IMFStreamDescriptor,
    key: REFGUID,
    buf: *mut *mut UINT8,
    size: *mut UINT32,
) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0) {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"stream_descriptor_GetAllocatedBlob\0".as_ptr() as *const i8,
            1736 as i32,
            b"%p, %s, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            buf,
            size,
        );
    }
    return attributes_GetAllocatedBlob(
        core::ptr::addr_of_mut!((*stream_desc).attributes),
        key,
        buf,
        size,
    );
}
unsafe extern "system" fn stream_descriptor_GetUnknown(iface: *mut IMFStreamDescriptor, key: *const GUID, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"stream_descriptor_GetUnknown\0" as *const u8 as *const core::ffi::c_char,
            1745 as core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_guid(riid as *const GUID),
            out,
        );
    }
    return attributes_GetUnknown(core::ptr::addr_of_mut!((*stream_desc).attributes), key, riid, out);
}

unsafe extern "system" fn stream_descriptor_SetItem(iface: *mut IMFStreamDescriptor, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    TRACE!("%p, %s, %s.\n", iface, debugstr_attr(key), debugstr_propvar(value));

    return attributes_SetItem(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_DeleteItem(iface: *mut IMFStreamDescriptor, key: REFGUID) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if (*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0" as *const u8 as *const i8,
            b"stream_descriptor_DeleteItem\0" as *const u8 as *const i8,
            1763i32,
            b"%p, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
        );
    }
    return attributes_DeleteItem(core::ptr::addr_of_mut!((*stream_desc).attributes), key);
}

unsafe extern "system" fn stream_descriptor_DeleteAllItems(iface: *mut IMFStreamDescriptor) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    TRACE!("%p.\n", iface);
    return attributes_DeleteAllItems(core::ptr::addr_of_mut!((*stream_desc).attributes));
}

unsafe extern "system" fn stream_descriptor_SetUINT32(iface: *mut IMFStreamDescriptor, key: *const GUID, value: UINT32) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0" as *const u8 as *const i8,
            b"stream_descriptor_SetUINT32\0" as *const u8 as *const i8,
            1781 as i32,
            b"%p, %s, %u.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }

    return attributes_SetUINT32(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_SetUINT64(iface: *mut IMFStreamDescriptor, key: REFGUID, value: UINT64) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_SetUINT64(&raw mut (*stream_desc).attributes, key, value);
}

unsafe extern "system" fn stream_descriptor_SetDouble(iface: *mut IMFStreamDescriptor, key: *const GUID, value: f64) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0" as *const u8 as *const i8,
            b"stream_descriptor_SetDouble\0" as *const u8 as *const i8,
            1799 as i32,
            b"%p, %s, %f.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_SetDouble(&raw mut (*stream_desc).attributes, key, value);
}

unsafe extern "system" fn stream_descriptor_SetGUID(
    iface: *mut IMFStreamDescriptor,
    key: *const GUID,
    value: *const GUID,
) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    // TRACE channel check (expanded from TRACE macro in the anchor)
    if ((*(*core::ptr::addr_of!(__wine_dbch___default))).flags as i32
        & ((1i32) << 3i32)) != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = 3; // __WINE_DBCL_TRACE
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"stream_descriptor_SetGUID\0".as_ptr() as *const i8,
            1808,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_mf_guid(value),
        );
    }

    return attributes_SetGUID(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_SetString(iface: *mut IMFStreamDescriptor, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    if (*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"stream_descriptor_SetString\0".as_ptr() as *const i8,
            1817i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_w(value),
        );
    }
    return attributes_SetString(core::ptr::addr_of_mut!((*stream_desc).attributes), key, value);
}

unsafe extern "system" fn stream_descriptor_SetBlob(iface: *mut IMFStreamDescriptor, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_SetBlob(core::ptr::addr_of_mut!((*stream_desc).attributes), key, buf, size);
}

unsafe extern "system" fn stream_descriptor_SetUnknown(iface: *mut IMFStreamDescriptor, key: REFGUID, unknown: *mut IUnknown) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), unknown);

    return attributes_SetUnknown(core::ptr::addr_of_mut!((*stream_desc).attributes), key, unknown);
}

unsafe extern "system" fn stream_descriptor_LockStore(iface: *mut IMFStreamDescriptor) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    TRACE!("%p.\n", iface);

    return attributes_LockStore(core::ptr::addr_of_mut!((*stream_desc).attributes));
}

unsafe extern "system" fn stream_descriptor_UnlockStore(iface: *mut IMFStreamDescriptor) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_UnlockStore(core::ptr::addr_of_mut!((*stream_desc).attributes));
}

unsafe extern "system" fn stream_descriptor_GetCount(iface: *mut IMFStreamDescriptor, count: *mut UINT32) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    return attributes_GetCount(core::ptr::addr_of_mut!((*stream_desc).attributes), count);
}

unsafe extern "system" fn stream_descriptor_GetItemByIndex(
    mut iface: *mut IMFStreamDescriptor,
    mut index: UINT32,
    mut key: *mut GUID,
    mut value: *mut PROPVARIANT,
) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    TRACE!("%p, %u, %p, %p.\n", iface, index, key, value);
    return attributes_GetItemByIndex(
        core::ptr::addr_of_mut!((*stream_desc).attributes),
        index,
        key,
        value,
    );
}

unsafe extern "system" fn stream_descriptor_CopyAllItems(iface: *mut IMFStreamDescriptor, dest: *mut IMFAttributes) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    TRACE!("%p, %p.\n", iface, dest);

    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*stream_desc).attributes), dest);
}
unsafe extern "system" fn stream_descriptor_GetStreamIdentifier(iface: *mut IMFStreamDescriptor, identifier: *mut DWORD) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);
    TRACE!("%p, %p.\n", iface, identifier);
    *identifier = (*stream_desc).identifier;
    return S_OK;
}

unsafe extern "system" fn stream_descriptor_GetMediaTypeHandler(iface: *mut IMFStreamDescriptor, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFStreamDescriptor(iface);

    *handler = core::ptr::addr_of_mut!((*stream_desc).IMFMediaTypeHandler_iface) as *mut IMFMediaTypeHandler;
    IMFMediaTypeHandler_AddRef(*handler);

    return S_OK;
}

unsafe extern "system" fn mediatype_handler_QueryInterface(iface: *mut IMFMediaTypeHandler, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaTypeHandler)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFMediaTypeHandler_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid as *const GUID));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn mediatype_handler_AddRef(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamDescriptor_AddRef(core::ptr::addr_of_mut!((*stream_desc).IMFStreamDescriptor_iface));
}

unsafe extern "system" fn mediatype_handler_Release(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamDescriptor_Release(core::ptr::addr_of_mut!((*stream_desc).IMFStreamDescriptor_iface));
}

unsafe extern "C" fn stream_descriptor_is_mediatype_supported(media_type: *mut IMFMediaType, candidate: *mut IMFMediaType) -> BOOL {
    let mut flags: DWORD = 0;

    if (IMFMediaType_IsEqual(media_type, candidate, core::ptr::addr_of_mut!(flags))) < 0 {
        return (FALSE) as BOOL;
    }

    return ((flags & (MF_MEDIATYPE_EQUAL_MAJOR_TYPES | MF_MEDIATYPE_EQUAL_FORMAT_TYPES)) ==
            (MF_MEDIATYPE_EQUAL_MAJOR_TYPES | MF_MEDIATYPE_EQUAL_FORMAT_TYPES)) as BOOL;
}

unsafe extern "system" fn mediatype_handler_IsMediaTypeSupported(
    iface: *mut IMFMediaTypeHandler,
    in_type: *mut IMFMediaType,
    out_type: *mut *mut IMFMediaType,
) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);
    let mut supported: BOOL = (FALSE as i32);
    let mut i: ::core::ffi::c_uint;

    if in_type.is_null() {
        return E_POINTER;
    }

    if !out_type.is_null() {
        *out_type = ::core::ptr::null_mut::<IMFMediaType>();
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));

    supported = (!(*stream_desc).current_type.is_null()
        && stream_descriptor_is_mediatype_supported((*stream_desc).current_type, in_type) != 0)
        as ::core::ffi::c_int as BOOL;
    if supported == 0 {
        i = 0;
        while i < (*stream_desc).media_types_count {
            supported = stream_descriptor_is_mediatype_supported(
                *(*stream_desc).media_types.offset(i as isize),
                in_type,
            );
            if supported != 0 {
                break;
            }
            i = i.wrapping_add(1);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));

    return if supported != 0 {
        S_OK
    } else {
        MF_E_INVALIDMEDIATYPE
    };
}

unsafe extern "system" fn mediatype_handler_GetMediaTypeCount(iface: *mut IMFMediaTypeHandler, count: *mut DWORD) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);

    TRACE!("%p, %p.\n", iface, count);

    *count = (*stream_desc).media_types_count;

    return S_OK;
}

unsafe extern "system" fn mediatype_handler_GetMediaTypeByIndex(iface: *mut IMFMediaTypeHandler, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);

    if index >= (*stream_desc).media_types_count as DWORD {
        return MF_E_NO_MORE_TYPES;
    }

    if !(*(*stream_desc).media_types.offset(index as isize)).is_null() {
        *type_ = *(*stream_desc).media_types.offset(index as isize);
        IMFMediaType_AddRef(*type_);
    }

    return if !(*(*stream_desc).media_types.offset(index as isize)).is_null() {
        S_OK
    } else {
        E_FAIL
    };
}

unsafe extern "system" fn mediatype_handler_SetCurrentMediaType(iface: *mut IMFMediaTypeHandler, type_: *mut IMFMediaType) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);

    if type_.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));
    if !(*stream_desc).current_type.is_null() {
        IMFMediaType_Release((*stream_desc).current_type);
    }
    (*stream_desc).current_type = type_;
    IMFMediaType_AddRef((*stream_desc).current_type);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn mediatype_handler_GetCurrentMediaType(iface: *mut IMFMediaTypeHandler, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let mut stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));
    if !(*stream_desc).current_type.is_null() {
        *type_0 = (*stream_desc).current_type;
        IMFMediaType_AddRef(*type_0);
    } else {
        hr = MF_E_NOT_INITIALIZED;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));

    return hr;
}

unsafe extern "system" fn mediatype_handler_GetMajorType(iface: *mut IMFMediaTypeHandler, type_0: *mut GUID) -> HRESULT {
    let stream_desc: *mut stream_desc = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT;

    TRACE!("%p, %p.\n", iface, type_0);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));
    hr = IMFMediaType_GetGUID(
        if !(*stream_desc).current_type.is_null() {
            (*stream_desc).current_type
        } else {
            *(*stream_desc).media_types.offset(0 as isize)
        },
        &MF_MT_MAJOR_TYPE,
        type_0,
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream_desc).attributes.cs));

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateStreamDescriptor(
    identifier: DWORD,
    count: DWORD,
    types: *mut *mut IMFMediaType,
    descriptor: *mut *mut IMFStreamDescriptor,
) -> HRESULT {
    let mut object: *mut stream_desc = core::ptr::null_mut::<stream_desc>();
    let mut i: u32 = 0;
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const i8,
            b"MFCreateStreamDescriptor\0".as_ptr() as *const i8,
            2127 as i32,
            b"%ld, %ld, %p, %p.\n\0".as_ptr() as *const i8,
            identifier,
            count,
            types,
            descriptor,
        );
    }
    if count == 0 {
        return E_INVALIDARG;
    }
    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<stream_desc>() as usize) as _,
    ) as *mut stream_desc;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = init_attributes_object(
        core::ptr::addr_of_mut!((*object).attributes),
        0 as UINT32,
    );
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }
    (*object).IMFStreamDescriptor_iface.lpVtbl = core::ptr::addr_of!(streamdescriptorvtbl);
    (*object).IMFMediaTypeHandler_iface.lpVtbl = core::ptr::addr_of!(mediatypehandlervtbl);
    (*object).identifier = identifier;
    (*object).media_types = calloc(
        (count as usize) as _,
        (core::mem::size_of::<IMFMediaType>() as usize) as _,
    ) as *mut *mut IMFMediaType;
    if (*object).media_types.is_null() {
        IMFStreamDescriptor_Release(core::ptr::addr_of_mut!((*object).IMFStreamDescriptor_iface));
        return E_OUTOFMEMORY;
    }
    i = 0 as u32;
    while i < count {
        let fresh16 = (*object).media_types.offset(i as isize);
        *fresh16 = *types.offset(i as isize);
        if !(*(*object).media_types.offset(i as isize)).is_null() {
            IMFMediaType_AddRef(*(*object).media_types.offset(i as isize));
        }
        i = i.wrapping_add(1);
    }
    (*object).media_types_count = count as u32;

    *descriptor = core::ptr::addr_of_mut!((*object).IMFStreamDescriptor_iface);

    return S_OK;
}

unsafe extern "system" fn presentation_descriptor_QueryInterface(iface: *mut IMFPresentationDescriptor, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFPresentationDescriptor) as *const GUID) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes) as *const GUID) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown) as *const GUID) != 0
    {
        *out = iface as *mut core::ffi::c_void;
        IMFPresentationDescriptor_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn presentation_descriptor_AddRef(iface: *mut IMFPresentationDescriptor) -> ULONG {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*presentation_desc).attributes.ref_) as *mut i32,
    ) as ULONG;

    return refcount;
}

unsafe extern "system" fn presentation_descriptor_Release(iface: *mut IMFPresentationDescriptor) -> ULONG {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*presentation_desc).attributes.ref_) as *mut LONG,
    ) as ULONG;
    let mut i: core::ffi::c_uint;

    if refcount == 0 {
        i = 0;
        while i < (*presentation_desc).count {
            if !(*(*presentation_desc).descriptors.offset(i as isize))
                .descriptor
                .is_null()
            {
                IMFStreamDescriptor_Release(
                    (*(*presentation_desc).descriptors.offset(i as isize)).descriptor,
                );
            }
            i = i.wrapping_add(1);
        }
        clear_attributes_object(core::ptr::addr_of_mut!((*presentation_desc).attributes));
        free((*presentation_desc).descriptors as *mut core::ffi::c_void);
        free(presentation_desc as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn presentation_descriptor_GetItem(
    mut iface: *mut IMFPresentationDescriptor,
    key: *const GUID,
    mut value: *mut PROPVARIANT,
) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"presentation_descriptor_GetItem\0" as *const u8 as *const ::core::ffi::c_char,
            2218 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetItem(
        core::ptr::addr_of_mut!((*presentation_desc).attributes),
        key,
        value,
    );
}

unsafe extern "system" fn presentation_descriptor_GetItemType(iface: *mut IMFPresentationDescriptor, key: REFGUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_GetItemType(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, type_);
}

unsafe extern "system" fn presentation_descriptor_CompareItem(
    iface: *mut IMFPresentationDescriptor,
    key: *const GUID,
    value: *const PROPVARIANT,
    result: *mut BOOL,
) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"presentation_descriptor_CompareItem\0" as *const u8 as *const ::core::ffi::c_char,
            2238 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_propvar(value),
            result,
        );
    }
    return attributes_CompareItem(
        core::ptr::addr_of_mut!((*presentation_desc).attributes),
        key,
        value,
        result,
    );
}
unsafe extern "system" fn presentation_descriptor_Compare(iface: *mut IMFPresentationDescriptor, attrs: *mut IMFAttributes, type_: MF_ATTRIBUTES_MATCH_TYPE, result: *mut BOOL) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_Compare(core::ptr::addr_of_mut!((*presentation_desc).attributes), attrs, type_, result);
}
unsafe extern "system" fn presentation_descriptor_GetUINT32(
    mut iface: *mut IMFPresentationDescriptor,
    key: *const GUID,
    mut value: *mut UINT32,
) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"presentation_descriptor_GetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            2257 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetUINT32(
        core::ptr::addr_of_mut!((*presentation_desc).attributes),
        key,
        value,
    );
}

unsafe extern "system" fn presentation_descriptor_GetUINT64(
    iface: *mut IMFPresentationDescriptor,
    key: *const GUID,
    value: *mut UINT64,
) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);

    return attributes_GetUINT64(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_GetDouble(iface: *mut IMFPresentationDescriptor, key: REFGUID, value: *mut f64) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);

    return attributes_GetDouble(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_GetGUID(
    iface: *mut IMFPresentationDescriptor,
    key: REFGUID,
    value: *mut GUID,
) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"presentation_descriptor_GetGUID\0" as *const u8 as *const ::core::ffi::c_char,
            2284 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetGUID(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_GetStringLength(iface: *mut IMFPresentationDescriptor, key: REFGUID, length: *mut UINT32) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_GetStringLength(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, length);
}

unsafe extern "system" fn presentation_descriptor_GetString(
    iface: *mut IMFPresentationDescriptor,
    key: *const GUID,
    value: *mut WCHAR,
    size: UINT32,
    length: *mut UINT32,
) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"presentation_descriptor_GetString\0" as *const u8 as *const i8,
            2304i32,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
            size,
            length,
        );
    }
    return attributes_GetString(
        core::ptr::addr_of_mut!((*presentation_desc).attributes),
        key,
        value,
        size,
        length,
    );
}

unsafe extern "system" fn presentation_descriptor_GetAllocatedString(
    iface: *mut IMFPresentationDescriptor,
    key: *const GUID,
    value: *mut *mut WCHAR,
    length: *mut UINT32,
) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"presentation_descriptor_GetAllocatedString\0" as *const u8 as *const i8,
            2314i32,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            value,
            length,
        );
    }
    return attributes_GetAllocatedString(
        &raw mut (*presentation_desc).attributes,
        key,
        value,
        length,
    );
}

unsafe extern "system" fn presentation_descriptor_GetBlobSize(iface: *mut IMFPresentationDescriptor, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), size);

    return attributes_GetBlobSize(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, size);
}

unsafe extern "system" fn presentation_descriptor_GetBlob(iface: *mut IMFPresentationDescriptor, key: *const GUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_GetBlob(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, buf, bufsize, blobsize);
}

unsafe extern "system" fn presentation_descriptor_GetAllocatedBlob(
    iface: *mut IMFPresentationDescriptor,
    key: REFGUID,
    buf: *mut *mut UINT8,
    size: *mut UINT32,
) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_GetAllocatedBlob(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, buf, size);
}

unsafe extern "system" fn presentation_descriptor_GetUnknown(iface: *mut IMFPresentationDescriptor, key: REFGUID, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_GetUnknown(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, riid, out);
}

unsafe extern "system" fn presentation_descriptor_SetItem(iface: *mut IMFPresentationDescriptor, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_SetItem(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_DeleteItem(iface: *mut IMFPresentationDescriptor, key: *const GUID) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_DeleteItem(core::ptr::addr_of_mut!((*presentation_desc).attributes), key);
}
unsafe extern "system" fn presentation_descriptor_DeleteAllItems(
    iface: *mut IMFPresentationDescriptor,
) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl as i32) as u32,
            (__dbch as *mut core::ffi::c_void).cast(),
            b"dll/win32/mfplat/mediatype.c\0"
                .as_ptr() as *const i8,
            b"presentation_descriptor_DeleteAllItems\0".as_ptr() as *const i8,
            2381i32,
            b"%p.\n\0".as_ptr() as *const i8,
            iface,
        );
    }
    return attributes_DeleteAllItems(core::ptr::addr_of_mut!((*presentation_desc).attributes));
}

unsafe extern "system" fn presentation_descriptor_SetUINT32(iface: *mut IMFPresentationDescriptor, key: *const GUID, value: UINT32) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    TRACE!("%p, %s, %u.\n", iface, debugstr_attr(key), value);

    return attributes_SetUINT32(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}
unsafe extern "system" fn presentation_descriptor_SetUINT64(iface: *mut IMFPresentationDescriptor, key: *const GUID, value: UINT64) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    TRACE!("%p, %s, %s.\n", iface, debugstr_attr(key), wine_dbgstr_longlong(value));

    return attributes_SetUINT64(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_SetDouble(iface: *mut IMFPresentationDescriptor, key: *const GUID, value: f64) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_SetDouble(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_SetGUID(iface: *mut IMFPresentationDescriptor, key: *const GUID, value: *const GUID) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0".as_ptr() as *const ::core::ffi::c_char,
            b"presentation_descriptor_SetGUID\0".as_ptr() as *const ::core::ffi::c_char,
            2417 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0".as_ptr() as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_mf_guid(value),
        );
    }
    return attributes_SetGUID(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_SetString(iface: *mut IMFPresentationDescriptor, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_SetString(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, value);
}

unsafe extern "system" fn presentation_descriptor_SetBlob(
    iface: *mut IMFPresentationDescriptor,
    key: *const GUID,
    buf: *const UINT8,
    size: UINT32,
) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    TRACE!(
        "{:p}, {}, {:p}, {}.\n",
        iface,
        debugstr_attr(key),
        buf,
        size
    );

    return attributes_SetBlob(
        core::ptr::addr_of_mut!((*presentation_desc).attributes),
        key,
        buf,
        size,
    );
}

unsafe extern "system" fn presentation_descriptor_SetUnknown(iface: *mut IMFPresentationDescriptor, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_SetUnknown(core::ptr::addr_of_mut!((*presentation_desc).attributes), key, unknown);
}

unsafe extern "system" fn presentation_descriptor_LockStore(iface: *mut IMFPresentationDescriptor) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_LockStore(core::ptr::addr_of_mut!((*presentation_desc).attributes));
}

unsafe extern "system" fn presentation_descriptor_UnlockStore(iface: *mut IMFPresentationDescriptor) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_UnlockStore(core::ptr::addr_of_mut!((*presentation_desc).attributes));
}

unsafe extern "system" fn presentation_descriptor_GetCount(iface: *mut IMFPresentationDescriptor, count: *mut UINT32) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_GetCount(core::ptr::addr_of_mut!((*presentation_desc).attributes), count);
}
unsafe extern "system" fn presentation_descriptor_GetItemByIndex(iface: *mut IMFPresentationDescriptor, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_GetItemByIndex(core::ptr::addr_of_mut!((*presentation_desc).attributes), index, key, value);
}

unsafe extern "system" fn presentation_descriptor_CopyAllItems(iface: *mut IMFPresentationDescriptor, dest: *mut IMFAttributes) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*presentation_desc).attributes), dest);
}

unsafe extern "system" fn presentation_descriptor_GetStreamDescriptorCount(iface: *mut IMFPresentationDescriptor, count: *mut DWORD) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    *count = (*presentation_desc).count;

    return S_OK;
}

unsafe extern "system" fn presentation_descriptor_GetStreamDescriptorByIndex(
    iface: *mut IMFPresentationDescriptor,
    index: DWORD,
    selected: *mut BOOL,
    descriptor: *mut *mut IMFStreamDescriptor,
) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    if index >= (*presentation_desc).count as DWORD {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));
    *selected = (*(*presentation_desc).descriptors.offset(index as isize)).selected;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));

    *descriptor = (*(*presentation_desc).descriptors.offset(index as isize)).descriptor;
    IMFStreamDescriptor_AddRef(*descriptor);

    return S_OK;
}

unsafe extern "system" fn presentation_descriptor_SelectStream(iface: *mut IMFPresentationDescriptor, index: DWORD) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    if index >= (*presentation_desc).count {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));
    (*(*presentation_desc).descriptors.offset(index as isize)).selected = (TRUE as i32);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn presentation_descriptor_DeselectStream(
    iface: *mut IMFPresentationDescriptor,
    index: DWORD,
) -> HRESULT {
    let mut presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);

    if index >= (*presentation_desc).count as DWORD {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));
    (*(*presentation_desc).descriptors.offset(index as isize)).selected = (FALSE as i32);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn presentation_descriptor_Clone(
    iface: *mut IMFPresentationDescriptor,
    descriptor: *mut *mut IMFPresentationDescriptor,
) -> HRESULT {
    let presentation_desc: *mut presentation_desc = impl_from_IMFPresentationDescriptor(iface);
    let mut object: *mut presentation_desc;
    let mut i: u32;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<presentation_desc>()) as _) as *mut presentation_desc;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = presentation_descriptor_init(object, (*presentation_desc).count);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));

    i = 0;
    while i < (*presentation_desc).count {
        *(*object).descriptors.offset(i as isize) = core::ptr::read((*presentation_desc).descriptors.offset(i as isize));
        IMFStreamDescriptor_AddRef((*(*object).descriptors.offset(i as isize)).descriptor);
        i += 1;
    }

    attributes_CopyAllItems(
        core::ptr::addr_of_mut!((*presentation_desc).attributes),
        core::ptr::addr_of_mut!((*object).IMFPresentationDescriptor_iface) as *mut IMFAttributes,
    );

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presentation_desc).attributes.cs));

    *descriptor = core::ptr::addr_of_mut!((*object).IMFPresentationDescriptor_iface);

    return S_OK;
}

unsafe extern "C" fn presentation_descriptor_init(
    object: *mut presentation_desc,
    count: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    hr = init_attributes_object(
        core::ptr::addr_of_mut!((*object).attributes),
        0 as UINT32,
    );
    if hr < 0 as HRESULT {
        return hr;
    }
    (*object).IMFPresentationDescriptor_iface.lpVtbl = core::ptr::addr_of!(presentationdescriptorvtbl);
    (*object).descriptors = calloc(
        (count as size_t) as _,
        (core::mem::size_of::<presentation_desc_entry>() as size_t) as _,
    ) as *mut presentation_desc_entry;
    if (*object).descriptors.is_null() {
        IMFPresentationDescriptor_Release(core::ptr::addr_of_mut!((*object).IMFPresentationDescriptor_iface));
        return E_OUTOFMEMORY as HRESULT;
    }
    (*object).count = count as ::core::ffi::c_uint;

    return S_OK as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreatePresentationDescriptor(
    count: DWORD,
    descriptors: *mut *mut IMFStreamDescriptor,
    out: *mut *mut IMFPresentationDescriptor,
) -> HRESULT {
    let object: *mut presentation_desc;
    let mut i: u32;
    let hr: HRESULT;

    TRACE!("%lu, %p, %p.\n", count, descriptors, out);

    if count == 0 {
        return E_INVALIDARG;
    }

    i = 0;
    while i < count {
        if (*descriptors.offset(i as isize)).is_null() {
            return E_INVALIDARG;
        }
        i = i.wrapping_add(1);
    }

    object = calloc(
        1,
        (core::mem::size_of::<presentation_desc>()) as _,
    ) as *mut presentation_desc;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = presentation_descriptor_init(object, count);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    i = 0;
    while i < count {
        (*(*object).descriptors.offset(i as isize)).descriptor =
            *descriptors.offset(i as isize);
        IMFStreamDescriptor_AddRef((*(*object).descriptors.offset(i as isize)).descriptor);
        i = i.wrapping_add(1);
    }

    *out = core::ptr::addr_of_mut!((*object).IMFPresentationDescriptor_iface);

    return S_OK;
}

unsafe extern "C" fn uncompressed_video_format_compare(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    let a_format: *const uncompressed_video_format = a as *const uncompressed_video_format;
    let b_format: *const uncompressed_video_format = b as *const uncompressed_video_format;
    return memcmp(
        (*a_format).subtype as *const core::ffi::c_void,
        (*b_format).subtype as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
    );
}
unsafe extern "system" fn mf_video_formats_init(
    once: *mut INIT_ONCE,
    param: *mut ::core::ffi::c_void,
    context: *mut *mut ::core::ffi::c_void,
) -> BOOL {
    qsort(
        core::ptr::addr_of_mut!(video_formats) as *mut ::core::ffi::c_void,
        ARRAY_SIZE!(video_formats) as size_t,
        ::core::mem::size_of::<uncompressed_video_format>() as size_t,
        Some(
            uncompressed_video_format_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    return (TRUE) as BOOL;
}

unsafe extern "C" fn mf_get_video_format(subtype: *const GUID) -> *mut uncompressed_video_format {
    static mut init_once: INIT_ONCE = INIT_ONCE_STATIC_INIT;
    let mut key: uncompressed_video_format = uncompressed_video_format {
        subtype: subtype,
        bpp: 0,
        alignment: 0,
        bottom_up: 0,
        yuv: 0,
        compression: 0,
    };

    InitOnceExecuteOnce(
        core::ptr::addr_of_mut!(init_once),
        Some(
            mf_video_formats_init,
        ),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    return bsearch(
        core::ptr::addr_of_mut!(key) as *const core::ffi::c_void,
        core::ptr::addr_of_mut!(video_formats) as *mut uncompressed_video_format as *const core::ffi::c_void,
        (ARRAY_SIZE!(video_formats)) as _,
        core::mem::size_of::<uncompressed_video_format>() as size_t,
        Some(
            uncompressed_video_format_compare
                as unsafe extern "C" fn(
                    *const core::ffi::c_void,
                    *const core::ffi::c_void,
                ) -> core::ffi::c_int,
        ),
    ) as *mut uncompressed_video_format;
}

unsafe extern "C" fn mf_get_stride_for_format(
    format: *const uncompressed_video_format,
    width: u32,
) -> u32 {
    if ((*format).bpp as i32) < 8 {
        return width.wrapping_mul((*format).bpp as u32) / 8;
    } else if IsEqualGUID!((*format).subtype, core::ptr::addr_of!(MFVideoFormat_P010)) != false {
        return width.wrapping_mul(2);
    }
    return width
        .wrapping_mul(((*format).bpp as i32 / 8) as u32)
        .wrapping_add((*format).alignment as u32)
        & !((*format).alignment as u32);
}
#[no_mangle]
pub unsafe extern "C" fn mf_format_get_stride(mut subtype: *const GUID, mut width: u32, mut is_yuv: *mut BOOL) -> u32 {
    let mut format: *mut uncompressed_video_format = mf_get_video_format(subtype);

    if !format.is_null() {
        *is_yuv = (*format).yuv as BOOL;
        return mf_get_stride_for_format(format, width);
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn MFGetStrideForBitmapInfoHeader(fourcc: DWORD, width: DWORD, stride: *mut LONG) -> HRESULT {
    let mut format: *mut uncompressed_video_format = core::ptr::null_mut();
    let mut subtype: GUID = core::mem::zeroed();

    memcpy(
        core::ptr::addr_of_mut!(subtype) as *mut core::ffi::c_void,
        core::ptr::addr_of!(MFVideoFormat_Base) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
    );
    subtype.Data1 = fourcc;

    format = mf_get_video_format(core::ptr::addr_of_mut!(subtype));
    if format.is_null() {
        *stride = 0;
        return MF_E_INVALIDMEDIATYPE;
    }

    *stride = mf_get_stride_for_format(format, width) as LONG;
    if (*format).bottom_up != 0 {
        *stride *= -1;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCalculateImageSize(
    subtype: *const GUID,
    width: UINT32,
    height: UINT32,
    size: *mut UINT32,
) -> HRESULT {
    let format: *mut uncompressed_video_format = mf_get_video_format(subtype);
    let mut stride: u32;

    if height == 0 {
        return E_INVALIDARG;
    }

    if format.is_null() {
        *size = 0;
        return E_INVALIDARG;
    }

    match (*subtype).Data1 {
        // MAKEFOURCC('I','M','C','2'), ('I','M','C','4'), ('N','V','1','2'),
        // ('Y','V','1','2'), ('I','4','2','0'), ('I','Y','U','V')
        843271497 | 876825929 | 842094158 | 842094169 | 808596553 | 1448433993 => {
            /* 2 x 2 block, interleaving UV for half the height */
            *size = (width.wrapping_add(1) & !(1 as UINT32))
                .wrapping_mul(height)
                .wrapping_mul(3)
                .wrapping_div(2);
        }
        // MAKEFOURCC('P','0','1','0')
        808530000 => {
            /* width is rounded up to a multiple of two */
            *size = (width.wrapping_add(1) & !(1 as UINT32))
                .wrapping_mul(height)
                .wrapping_mul(3);
        }
        // MAKEFOURCC('N','V','1','1')
        825316942 => {
            *size = (width.wrapping_add(3) & !(3 as UINT32))
                .wrapping_mul(height)
                .wrapping_mul(3)
                .wrapping_div(2);
        }
        D3DFMT_L8 | D3DFMT_L16 | D3DFMT_D16 => {
            *size = width
                .wrapping_mul(((*format).bpp as UINT32) / 8)
                .wrapping_mul(height);
        }
        _ => {
            stride = mf_get_stride_for_format(format, width);
            *size = stride.wrapping_mul(height);
        }
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFGetPlaneSize(
    mut fourcc: DWORD,
    mut width: DWORD,
    mut height: DWORD,
    mut size: *mut DWORD,
) -> HRESULT {
    let mut format: *mut uncompressed_video_format =
        ::core::ptr::null_mut::<uncompressed_video_format>();
    let mut stride: ::core::ffi::c_uint = 0;
    let mut subtype: GUID = _GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFGetPlaneSize\0" as *const u8 as *const ::core::ffi::c_char,
            2873 as ::core::ffi::c_int,
            b"%s, %lu, %lu, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            mf_debugstr_fourcc(fourcc),
            width,
            height,
            size,
        );
    }
    memcpy(
        &raw mut subtype as *mut ::core::ffi::c_void,
        &raw const MFVideoFormat_Base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<GUID>() as size_t,
    );
    subtype.Data1 = fourcc as ::core::ffi::c_uint;
    format = mf_get_video_format(&raw mut subtype);
    if !format.is_null() {
        stride = mf_get_stride_for_format(format, width as ::core::ffi::c_uint);
    } else {
        stride = 0 as ::core::ffi::c_uint;
    }
    match fourcc {
        // MAKEFOURCC('I','M','C','2'), MAKEFOURCC('I','M','C','4'),
        // MAKEFOURCC('N','V','1','2'), MAKEFOURCC('Y','V','1','2'),
        // MAKEFOURCC('I','4','2','0'), MAKEFOURCC('I','Y','U','V'),
        // MAKEFOURCC('N','V','1','1'), MAKEFOURCC('P','0','1','0')
        843271497 | 876825929 | 842094158 | 842094169 | 808596553 | 1448433993 | 825316942
        | 808530000 => {
            *size = (stride as DWORD)
                .wrapping_mul(height)
                .wrapping_mul(3 as DWORD)
                .wrapping_div(2 as DWORD);
        }
        _ => {
            *size = (stride as DWORD).wrapping_mul(height);
        }
    }
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCompareFullToPartialMediaType(
    full_type: *mut IMFMediaType,
    partial_type: *mut IMFMediaType,
) -> BOOL {
    let mut result: BOOL = 0;
    let mut major: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };

    if (IMFMediaType_GetMajorType(partial_type, core::ptr::addr_of_mut!(major)) < 0) {
        return (FALSE) as BOOL;
    }

    if (IMFMediaType_Compare(
        partial_type,
        full_type as *mut IMFAttributes,
        MF_ATTRIBUTES_MATCH_OUR_ITEMS,
        core::ptr::addr_of_mut!(result),
    ) < 0) {
        return (FALSE) as BOOL;
    }

    return result;
}

#[no_mangle]
pub unsafe extern "system" fn MFWrapMediaType(
    original: *mut IMFMediaType,
    major: *const GUID,
    subtype: *const GUID,
    ret: *mut *mut IMFMediaType,
) -> HRESULT {
    let mut mediatype: *mut IMFMediaType = core::ptr::null_mut();
    let mut buffer: *mut UINT8 = core::ptr::null_mut();
    let mut size: UINT32 = 0;
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
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFWrapMediaType\0" as *const u8 as *const ::core::ffi::c_char,
            2931 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            original,
            debugstr_guid(major as *const _GUID),
            debugstr_guid(subtype as *const _GUID),
            ret,
        );
    }

    hr = MFGetAttributesAsBlobSize(original as *mut IMFAttributes, &raw mut size);
    if hr < 0 {
        return hr;
    }

    buffer = malloc(size as usize) as *mut UINT8;
    if buffer.is_null() {
        return E_OUTOFMEMORY;
    }

    'failed: {
        hr = MFGetAttributesAsBlob(original as *mut IMFAttributes, buffer, size);
        if hr < 0 {
            break 'failed;
        }

        hr = MFCreateMediaType(&raw mut mediatype);
        if hr < 0 {
            break 'failed;
        }

        hr = IMFMediaType_SetGUID(mediatype, &raw const MF_MT_MAJOR_TYPE, major);
        if hr < 0 {
            break 'failed;
        }

        hr = IMFMediaType_SetGUID(mediatype, &raw const MF_MT_SUBTYPE, subtype);
        if hr < 0 {
            break 'failed;
        }

        hr = IMFMediaType_SetBlob(mediatype, &raw const MF_MT_WRAPPED_TYPE, buffer, size);
        if hr < 0 {
            break 'failed;
        }

        *ret = mediatype;
    }

    free(buffer as *mut core::ffi::c_void);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFUnwrapMediaType(
    wrapper: *mut IMFMediaType,
    ret: *mut *mut IMFMediaType,
) -> HRESULT {
    let mut mediatype: *mut IMFMediaType = core::ptr::null_mut::<IMFMediaType>();
    let mut buffer: *mut UINT8 = core::ptr::null_mut::<UINT8>();
    let mut size: UINT32 = 0;
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
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFUnwrapMediaType\0" as *const u8 as *const ::core::ffi::c_char,
            2972 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            wrapper,
            ret,
        );
    }

    hr = MFCreateMediaType(&raw mut mediatype);
    if hr < 0 {
        return hr;
    }

    hr = IMFMediaType_GetAllocatedBlob(
        wrapper,
        &raw const MF_MT_WRAPPED_TYPE,
        &raw mut buffer,
        &raw mut size,
    );
    if hr < 0 {
        IMFMediaType_Release(mediatype);
        return hr;
    }

    hr = MFInitAttributesFromBlob(mediatype as *mut IMFAttributes, buffer, size);
    CoTaskMemFree(buffer as LPVOID);
    if hr < 0 {
        return hr;
    }

    *ret = mediatype;

    return S_OK;
}

unsafe extern "C" fn media_type_get_uint32(media_type: *mut IMFMediaType, guid: *const GUID) -> UINT32 {
    let mut value: UINT32 = 0;
    return if (IMFMediaType_GetUINT32(media_type, guid, core::ptr::addr_of_mut!(value))) >= 0 {
        value
    } else {
        0
    };
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateWaveFormatExFromMFMediaType(
    mediatype: *mut IMFMediaType,
    ret_format: *mut *mut WAVEFORMATEX,
    size: *mut UINT32,
    mut flags: UINT32,
) -> HRESULT {
    let mut extra_size: UINT32 = 0;
    let mut user_size: UINT32 = core::mem::zeroed();
    let mut format: *mut WAVEFORMATEX = core::ptr::null_mut();
    let mut major: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
    let mut subtype: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
    let mut basetype: GUID = MFAudioFormat_Base;
    let mut user_data: *mut core::ffi::c_void;
    let mut hr: HRESULT;

    hr = IMFMediaType_GetGUID(mediatype, &raw const MF_MT_MAJOR_TYPE, &raw mut major);
    if hr < 0 {
        return hr;
    }

    hr = IMFMediaType_GetGUID(mediatype, &raw const MF_MT_SUBTYPE, &raw mut subtype);
    if hr < 0 {
        return hr;
    }

    if IsEqualGUID!(core::ptr::addr_of!(major), core::ptr::addr_of!(MFMediaType_Audio)) == false {
        return E_INVALIDARG;
    }

    hr = IMFMediaType_GetBlobSize(mediatype, &raw const MF_MT_USER_DATA, &raw mut user_size);
    if hr < 0 {
        user_size = 0;
    }

    if media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_NUM_CHANNELS) > 2
        && IMFMediaType_GetItem(
            mediatype,
            &raw const MF_MT_AUDIO_CHANNEL_MASK,
            core::ptr::null_mut::<PROPVARIANT>(),
        ) >= 0
    {
        if IMFMediaType_GetItem(
            mediatype,
            &raw const MF_MT_AUDIO_VALID_BITS_PER_SAMPLE,
            core::ptr::null_mut::<PROPVARIANT>(),
        ) >= 0
        {
            flags = MFWaveFormatExConvertFlag_ForceExtensible;
        }
        if IMFMediaType_GetItem(
            mediatype,
            &raw const MF_MT_AUDIO_SAMPLES_PER_BLOCK,
            core::ptr::null_mut::<PROPVARIANT>(),
        ) >= 0
        {
            flags = MFWaveFormatExConvertFlag_ForceExtensible;
        }
    }

    basetype.Data1 = subtype.Data1;
    if (subtype.Data1 >> 16) != 0
        || IsEqualGUID!(core::ptr::addr_of!(subtype), core::ptr::addr_of!(basetype)) == false
    {
        flags = MFWaveFormatExConvertFlag_ForceExtensible;
    }

    if flags == MFWaveFormatExConvertFlag_ForceExtensible {
        extra_size = (core::mem::size_of::<WAVEFORMATEXTENSIBLE>()
            - core::mem::size_of::<WAVEFORMATEX>()) as UINT32;
    }

    *size = (core::mem::size_of::<WAVEFORMATEX>() as UINT32)
        .wrapping_add(user_size)
        .wrapping_add(extra_size);
    format = CoTaskMemAlloc((*size) as SIZE_T) as *mut WAVEFORMATEX;
    if format.is_null() {
        return E_OUTOFMEMORY;
    }

    memset(format as *mut core::ffi::c_void, 0, *size as usize);
    (*format).wFormatTag = subtype.Data1 as WORD;
    (*format).cbSize = user_size.wrapping_add(extra_size) as WORD;
    user_data = format.offset(1 as isize) as *mut core::ffi::c_void;

    (*format).nChannels = media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_NUM_CHANNELS) as WORD;
    (*format).nSamplesPerSec = media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_SAMPLES_PER_SECOND) as DWORD;
    (*format).nAvgBytesPerSec = media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_AVG_BYTES_PER_SECOND) as DWORD;
    (*format).nBlockAlign = media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_BLOCK_ALIGNMENT) as WORD;
    (*format).wBitsPerSample = media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_BITS_PER_SAMPLE) as WORD;

    if flags == MFWaveFormatExConvertFlag_ForceExtensible {
        let mut format_ext: *mut WAVEFORMATEXTENSIBLE =
            CONTAINING_RECORD!(format, WAVEFORMATEXTENSIBLE, Format);

        (*format_ext).Format.wFormatTag = WAVE_FORMAT_EXTENSIBLE as WORD;
        (*format_ext).SubFormat = subtype;
        user_data = format_ext.offset(1 as isize) as *mut core::ffi::c_void;

        (*format_ext).Samples.wValidBitsPerSample =
            media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_VALID_BITS_PER_SAMPLE) as WORD;
        (*format_ext).Samples.wSamplesPerBlock =
            media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_SAMPLES_PER_BLOCK) as WORD;

        if IMFMediaType_GetItem(
            mediatype,
            &raw const MF_MT_AUDIO_CHANNEL_MASK,
            core::ptr::null_mut::<PROPVARIANT>(),
        ) >= 0
        {
            (*format_ext).dwChannelMask =
                media_type_get_uint32(mediatype, &raw const MF_MT_AUDIO_CHANNEL_MASK) as DWORD;
        } else if ((*format_ext).Format.nChannels as usize) < ARRAY_SIZE!(default_channel_mask) as usize {
            (*format_ext).dwChannelMask =
                default_channel_mask[(*format_ext).Format.nChannels as usize] as DWORD;
        }
    }

    IMFMediaType_GetBlob(
        mediatype,
        &raw const MF_MT_USER_DATA,
        user_data as *mut UINT8,
        user_size,
        core::ptr::null_mut::<UINT32>(),
    );

    *ret_format = format;

    return S_OK;
}

unsafe extern "C" fn mediatype_set_uint32(mediatype: *mut IMFMediaType, attr: *const GUID, value: core::ffi::c_uint, hr: *mut HRESULT) {
    if (*hr) >= 0 {
        *hr = IMFMediaType_SetUINT32(mediatype, attr, value as UINT32);
    }
}

unsafe extern "C" fn mediatype_set_uint64(mediatype: *mut IMFMediaType, attr: *const GUID, high: ::core::ffi::c_uint, low: ::core::ffi::c_uint, hr: *mut HRESULT) {
    if (*hr) >= 0 {
        *hr = IMFMediaType_SetUINT64(mediatype, attr, ((high as UINT64) << 32) | (low as UINT64));
    }
}

unsafe extern "C" fn mediatype_set_guid(mediatype: *mut IMFMediaType, attr: *const GUID, value: *const GUID, hr: *mut HRESULT) {
    if (*hr) >= 0 {
        *hr = IMFMediaType_SetGUID(mediatype, attr, value);
    }
}

unsafe extern "C" fn mediatype_set_blob(mediatype: *mut IMFMediaType, attr: *const GUID, data: *const UINT8, size: ::core::ffi::c_uint, hr: *mut HRESULT) {
    if (*hr >= 0) {
        *hr = IMFMediaType_SetBlob(mediatype, attr, data, size as UINT32);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MFInitMediaTypeFromWaveFormatEx(
    mediatype: *mut IMFMediaType,
    format: *const WAVEFORMATEX,
    size: UINT32,
) -> HRESULT {
    let wfex: *const WAVEFORMATEXTENSIBLE = format as *const WAVEFORMATEXTENSIBLE;
    let mut user_data: *const core::ffi::c_void = core::ptr::null();
    let mut user_data_size: core::ffi::c_int;
    let mut subtype: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut hr: HRESULT;

    if mediatype.is_null() || format.is_null() {
        return E_POINTER;
    }

    if ((*format).cbSize as usize).wrapping_add(core::mem::size_of::<WAVEFORMATEX>() as usize)
        > size as usize
    {
        return E_INVALIDARG;
    }

    hr = IMFMediaType_DeleteAllItems(mediatype);

    mediatype_set_guid(
        mediatype,
        core::ptr::addr_of!(MF_MT_MAJOR_TYPE),
        core::ptr::addr_of!(MFMediaType_Audio),
        core::ptr::addr_of_mut!(hr),
    );

    if (*format).wFormatTag as core::ffi::c_int == (WAVE_FORMAT_EXTENSIBLE as i32) {
        if ((*format).cbSize as usize)
            < (core::mem::size_of::<WAVEFORMATEXTENSIBLE>() as usize).wrapping_sub(
                core::mem::size_of::<WAVEFORMATEX>() as usize,
            )
        {
            return E_INVALIDARG;
        }

        memcpy(
            core::ptr::addr_of_mut!(subtype) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*wfex).SubFormat) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>() as size_t,
        );

        if (*wfex).dwChannelMask != 0 {
            mediatype_set_uint32(
                mediatype,
                core::ptr::addr_of!(MF_MT_AUDIO_CHANNEL_MASK),
                (*wfex).dwChannelMask as core::ffi::c_uint,
                core::ptr::addr_of_mut!(hr),
            );
        }

        if (*format).wBitsPerSample as core::ffi::c_int != 0
            && (*wfex).Samples.wValidBitsPerSample as core::ffi::c_int != 0
        {
            mediatype_set_uint32(
                mediatype,
                core::ptr::addr_of!(MF_MT_AUDIO_VALID_BITS_PER_SAMPLE),
                (*wfex).Samples.wValidBitsPerSample as core::ffi::c_uint,
                core::ptr::addr_of_mut!(hr),
            );
        }

        user_data_size = ((*format).cbSize as usize)
            .wrapping_sub(core::mem::size_of::<WAVEFORMATEXTENSIBLE>() as usize)
            .wrapping_add(core::mem::size_of::<WAVEFORMATEX>() as usize)
            as core::ffi::c_int;
        user_data = wfex.offset(1 as isize) as *const core::ffi::c_void;
    } else {
        memcpy(
            core::ptr::addr_of_mut!(subtype) as *mut core::ffi::c_void,
            core::ptr::addr_of!(MFAudioFormat_Base) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>() as size_t,
        );
        subtype.Data1 = (*format).wFormatTag as core::ffi::c_uint;

        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AUDIO_PREFER_WAVEFORMATEX),
            1 as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
        user_data_size = (*format).cbSize as core::ffi::c_int;
        user_data = format.offset(1 as isize) as *const core::ffi::c_void;
    }

    mediatype_set_guid(
        mediatype,
        core::ptr::addr_of!(MF_MT_SUBTYPE),
        core::ptr::addr_of_mut!(subtype),
        core::ptr::addr_of_mut!(hr),
    );

    if (*format).nChannels != 0 {
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AUDIO_NUM_CHANNELS),
            (*format).nChannels as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    if (*format).nSamplesPerSec != 0 {
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AUDIO_SAMPLES_PER_SECOND),
            (*format).nSamplesPerSec as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    if (*format).nAvgBytesPerSec != 0 {
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AUDIO_AVG_BYTES_PER_SECOND),
            (*format).nAvgBytesPerSec as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    if (*format).nBlockAlign != 0 {
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AUDIO_BLOCK_ALIGNMENT),
            (*format).nBlockAlign as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    if (*format).wBitsPerSample != 0 {
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AUDIO_BITS_PER_SAMPLE),
            (*format).wBitsPerSample as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    if IsEqualGUID!(
        core::ptr::addr_of!(subtype),
        core::ptr::addr_of!(MFAudioFormat_PCM),
    ) != false
        || IsEqualGUID!(
            core::ptr::addr_of!(subtype),
            core::ptr::addr_of!(MFAudioFormat_Float),
        ) != false
    {
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_ALL_SAMPLES_INDEPENDENT),
            1 as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    if IsEqualGUID!(
        core::ptr::addr_of!(subtype),
        core::ptr::addr_of!(MFAudioFormat_AAC),
    ) != false
    {
        let info: *mut HEAACWAVEINFO =
            CONTAINING_RECORD!(format, HEAACWAVEINFO, wfx) as *mut HEAACWAVEINFO;
        if ((*format).cbSize as usize)
            < (core::mem::size_of::<HEAACWAVEINFO>() as usize).wrapping_sub(
                core::mem::size_of::<WAVEFORMATEX>() as usize,
            )
        {
            return E_INVALIDARG;
        }
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AAC_AUDIO_PROFILE_LEVEL_INDICATION),
            (*info).wAudioProfileLevelIndication as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
        mediatype_set_uint32(
            mediatype,
            core::ptr::addr_of!(MF_MT_AAC_PAYLOAD_TYPE),
            (*info).wPayloadType as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    if user_data_size > 0 {
        mediatype_set_blob(
            mediatype,
            core::ptr::addr_of!(MF_MT_USER_DATA),
            user_data as *const UINT8,
            user_data_size as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoMediaTypeFromSubtype(
    subtype: *const GUID,
    media_type: *mut *mut IMFVideoMediaType,
) -> HRESULT {
    let mut object: *mut media_type = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!("%s, %p.\n", debugstr_guid(subtype), media_type);

    if media_type.is_null() {
        return E_INVALIDARG;
    }

    hr = create_media_type(&raw mut object);
    if hr < 0 {
        return hr;
    }

    IMFMediaType_SetGUID(
        &raw mut (*object).IMFMediaType_iface,
        &raw const MF_MT_MAJOR_TYPE,
        &raw const MFMediaType_Video,
    );
    IMFMediaType_SetGUID(
        &raw mut (*object).IMFMediaType_iface,
        &raw const MF_MT_SUBTYPE,
        subtype,
    );

    *media_type = &raw mut (*object).IMFVideoMediaType_iface;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateAudioMediaType(
    format: *const WAVEFORMATEX,
    media_type: *mut *mut IMFAudioMediaType,
) -> HRESULT {
    let mut object: *mut media_type = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    DPRINT1!("%p, %p.\n", format, media_type);

    if media_type.is_null() {
        return E_INVALIDARG;
    }

    hr = create_media_type(&raw mut object);
    if hr < 0 {
        return hr;
    }

    hr = MFInitMediaTypeFromWaveFormatEx(
        &raw mut (*object).IMFMediaType_iface,
        format,
        (core::mem::size_of::<WAVEFORMATEX>() as usize).wrapping_add((*format).cbSize as usize)
            as UINT32,
    );
    if hr < 0 {
        IMFMediaType_Release(&raw mut (*object).IMFMediaType_iface);
        return hr;
    }

    *media_type = &raw mut (*object).IMFAudioMediaType_iface;

    return S_OK;
}

unsafe extern "C" fn media_type_get_ratio(media_type: *mut IMFMediaType, attr: *const GUID, numerator: *mut DWORD, denominator: *mut DWORD) {
    let mut value: UINT64 = 0;
    if (IMFMediaType_GetUINT64(media_type, attr, core::ptr::addr_of_mut!(value))) >= 0 {
        *numerator = (value >> 32) as DWORD;
        *denominator = value as DWORD;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateAMMediaTypeFromMFMediaType(
    media_type: *mut IMFMediaType,
    format: GUID,
    am_type: *mut *mut AM_MEDIA_TYPE,
) -> HRESULT {
    let mut mt: *mut AM_MEDIA_TYPE = core::ptr::null_mut::<AM_MEDIA_TYPE>();
    let mut hr: HRESULT = 0;

    *am_type = core::ptr::null_mut::<AM_MEDIA_TYPE>();
    mt = CoTaskMemAlloc(core::mem::size_of::<AM_MEDIA_TYPE>() as SIZE_T) as *mut AM_MEDIA_TYPE;
    if mt.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = MFInitAMMediaTypeFromMFMediaType(media_type, format, mt);
    if hr < 0 {
        CoTaskMemFree(mt as LPVOID);
        return hr;
    }

    *am_type = mt;
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMFVideoFormatFromMFMediaType(
    media_type: *mut IMFMediaType,
    video_format: *mut *mut MFVIDEOFORMAT,
    size: *mut UINT32,
) -> HRESULT {
    let mut palette_size: UINT32 = 0;
    let mut user_data_size: UINT32 = 0;
    let mut format: *mut MFVIDEOFORMAT = core::ptr::null_mut();
    let mut stride: INT32;
    let mut guid: GUID = core::mem::zeroed();

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const i8,
            b"MFCreateMFVideoFormatFromMFMediaType\0" as *const u8 as *const i8,
            3287i32,
            b"%p, %p, %p.\n\0" as *const u8 as *const i8,
            media_type,
            video_format,
            size,
        );
    }

    if (IMFMediaType_GetBlobSize(media_type, &raw const MF_MT_PALETTE, &raw mut palette_size))
        >= 0
    {
        *size = (core::mem::offset_of!(MFVIDEOFORMAT, surfaceInfo.Palette)
            + ((palette_size / (core::mem::size_of::<MFPaletteEntry>() as UINT32)
                + 1) as usize)
                * core::mem::size_of::<MFPaletteEntry>()) as UINT32;
    } else {
        *size = core::mem::size_of::<MFVIDEOFORMAT>() as UINT32;
    }

    if (IMFMediaType_GetBlobSize(media_type, &raw const MF_MT_USER_DATA, &raw mut user_data_size))
        >= 0
    {
        *size += user_data_size;
    }

    format = CoTaskMemAlloc(*size as SIZE_T) as *mut MFVIDEOFORMAT;
    if format.is_null() {
        return E_OUTOFMEMORY;
    }

    *video_format = format;

    memset(
        format as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<MFVIDEOFORMAT>() as size_t,
    );
    (*format).dwSize = *size;

    if (IMFMediaType_GetGUID(media_type, &raw const MF_MT_SUBTYPE, &raw mut guid))
        >= 0
    {
        memcpy(
            &raw mut (*format).guidFormat as *mut core::ffi::c_void,
            &raw mut guid as *const core::ffi::c_void,
            core::mem::size_of::<GUID>() as size_t,
        );
        (*format).surfaceInfo.Format = guid.Data1;
    }

    media_type_get_ratio(
        media_type,
        &raw const MF_MT_FRAME_SIZE,
        &raw mut (*format).videoInfo.dwWidth,
        &raw mut (*format).videoInfo.dwHeight,
    );
    media_type_get_ratio(
        media_type,
        &raw const MF_MT_PIXEL_ASPECT_RATIO,
        &raw mut (*format).videoInfo.PixelAspectRatio.Numerator,
        &raw mut (*format).videoInfo.PixelAspectRatio.Denominator,
    );
    media_type_get_ratio(
        media_type,
        &raw const MF_MT_FRAME_RATE,
        &raw mut (*format).videoInfo.FramesPerSecond.Numerator,
        &raw mut (*format).videoInfo.FramesPerSecond.Denominator,
    );

    (*format).videoInfo.SourceChromaSubsampling =
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_CHROMA_SITING) as MFVideoChromaSubsampling;
    (*format).videoInfo.InterlaceMode =
        media_type_get_uint32(media_type, &raw const MF_MT_INTERLACE_MODE) as MFVideoInterlaceMode;
    (*format).videoInfo.TransferFunction =
        media_type_get_uint32(media_type, &raw const MF_MT_TRANSFER_FUNCTION) as MFVideoTransferFunction;
    (*format).videoInfo.ColorPrimaries =
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_PRIMARIES) as MFVideoPrimaries;
    (*format).videoInfo.TransferMatrix =
        media_type_get_uint32(media_type, &raw const MF_MT_YUV_MATRIX) as MFVideoTransferMatrix;
    (*format).videoInfo.SourceLighting =
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_LIGHTING) as MFVideoLighting;
    (*format).videoInfo.NominalRange =
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_NOMINAL_RANGE) as MFNominalRange;

    IMFMediaType_GetBlob(
        media_type,
        &raw const MF_MT_GEOMETRIC_APERTURE,
        &raw mut (*format).videoInfo.GeometricAperture as *mut UINT8,
        core::mem::size_of::<MFVideoArea>() as UINT32,
        core::ptr::null_mut::<UINT32>(),
    );
    IMFMediaType_GetBlob(
        media_type,
        &raw const MF_MT_MINIMUM_DISPLAY_APERTURE,
        &raw mut (*format).videoInfo.MinimumDisplayAperture as *mut UINT8,
        core::mem::size_of::<MFVideoArea>() as UINT32,
        core::ptr::null_mut::<UINT32>(),
    );

    /* Video flags. */
    (*format).videoInfo.VideoFlags |=
        (media_type_get_uint32(media_type, &raw const MF_MT_PAD_CONTROL_FLAGS) as UINT32
            & MFVideoFlag_PAD_TO_Mask as UINT32) as UINT64;
    (*format).videoInfo.VideoFlags |=
        ((media_type_get_uint32(media_type, &raw const MF_MT_SOURCE_CONTENT_HINT) as UINT32)
            << 2
            & MFVideoFlag_SrcContentHintMask as UINT32) as UINT64;
    (*format).videoInfo.VideoFlags |=
        ((media_type_get_uint32(media_type, &raw const MF_MT_DRM_FLAGS) as UINT32)
            << 5
            & (MFVideoFlag_AnalogProtected as UINT32 | MFVideoFlag_DigitallyProtected as UINT32))
            as UINT64;

    if media_type_get_uint32(media_type, &raw const MF_MT_PAN_SCAN_ENABLED) != 0 {
        (*format).videoInfo.VideoFlags |= MFVideoFlag_PanScanEnabled as UINT64;
        IMFMediaType_GetBlob(
            media_type,
            &raw const MF_MT_PAN_SCAN_APERTURE,
            &raw mut (*format).videoInfo.PanScanAperture as *mut UINT8,
            core::mem::size_of::<MFVideoArea>() as UINT32,
            core::ptr::null_mut::<UINT32>(),
        );
    }

    stride = media_type_get_uint32(media_type, &raw const MF_MT_DEFAULT_STRIDE) as INT32;
    if stride < 0 {
        (*format).videoInfo.VideoFlags |= MFVideoFlag_BottomUpLinearRep as UINT64;
    }

    (*format).compressedInfo.AvgBitrate =
        media_type_get_uint32(media_type, &raw const MF_MT_AVG_BITRATE) as LONGLONG;
    (*format).compressedInfo.AvgBitErrorRate =
        media_type_get_uint32(media_type, &raw const MF_MT_AVG_BIT_ERROR_RATE) as LONGLONG;
    (*format).compressedInfo.MaxKeyFrameSpacing =
        media_type_get_uint32(media_type, &raw const MF_MT_MAX_KEYFRAME_SPACING) as DWORD;

    /* Palette. */
    if palette_size != 0 {
        (*format).surfaceInfo.PaletteEntries =
            palette_size / (core::mem::size_of::<MFPaletteEntry>() as UINT32);
        IMFMediaType_GetBlob(
            media_type,
            &raw const MF_MT_PALETTE,
            &raw mut (*format).surfaceInfo.Palette as *mut MFPaletteEntry as *mut UINT8,
            palette_size,
            core::ptr::null_mut::<UINT32>(),
        );
    }

    if user_data_size != 0 {
        IMFMediaType_GetBlob(
            media_type,
            &raw const MF_MT_USER_DATA,
            (format as *mut UINT8)
                .offset(*size as isize)
                .offset(-(user_data_size as isize)),
            user_data_size,
            core::ptr::null_mut::<UINT32>(),
        );
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFConvertColorInfoToDXVA(
    dxva_info: *mut DWORD,
    format: *const MFVIDEOFORMAT,
) -> HRESULT {
    // Bitfield layout of the DXVA format word (LSB-first allocation).
    unsafe fn set_bits(p: *mut DWORD, off: u32, width: u32, v: u32) {
        let mask: u32 = (((1u64 << width) - 1) as u32) << off;
        *p = (*p & !mask) | ((v << off) & mask);
    }

    let w0: u32 = 8;
    let w1: u32 = 4;
    let w2: u32 = 3;
    let w3: u32 = 3;
    let w4: u32 = 4;
    let w5: u32 = 5;
    let w6: u32 = 5;

    let o0: u32 = 0;
    let o1: u32 = o0 + w0;
    let o2: u32 = o1 + w1;
    let o3: u32 = o2 + w2;
    let o4: u32 = o3 + w3;
    let o5: u32 = o4 + w4;
    let o6: u32 = o5 + w5;
    let _end: u32 = o6 + w6;

    if (*format).videoInfo.InterlaceMode as ::core::ffi::c_uint
        == MFVideoInterlace_MixedInterlaceOrProgressive as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        set_bits(
            dxva_info,
            o0,
            w0,
            DXVA2_SampleFieldInterleavedEvenFirst as ::core::ffi::c_int as UINT as UINT,
        );
    } else {
        set_bits(
            dxva_info,
            o0,
            w0,
            (*format).videoInfo.InterlaceMode as UINT as UINT,
        );
    }
    set_bits(
        dxva_info,
        o1,
        w1,
        (*format).videoInfo.SourceChromaSubsampling as UINT as UINT,
    );
    set_bits(
        dxva_info,
        o2,
        w2,
        (*format).videoInfo.NominalRange as UINT as UINT,
    );
    set_bits(
        dxva_info,
        o3,
        w3,
        (*format).videoInfo.TransferMatrix as UINT as UINT,
    );
    set_bits(
        dxva_info,
        o4,
        w4,
        (*format).videoInfo.SourceLighting as UINT as UINT,
    );
    set_bits(
        dxva_info,
        o5,
        w5,
        (*format).videoInfo.ColorPrimaries as UINT as UINT,
    );
    set_bits(
        dxva_info,
        o6,
        w6,
        (*format).videoInfo.TransferFunction as UINT as UINT,
    );

    return S_OK as HRESULT;
}

unsafe extern "C" fn known_rate_from_rate(rate: UINT64) -> *const frame_rate {
    let mut i: UINT = 0;
    while {
        i = i;
        (i as usize) < ARRAY_SIZE!(known_rates) as usize
    } {
        if rate == known_rates[i as usize].rate {
            return (core::ptr::addr_of!(known_rates) as *const frame_rate).offset(i as isize);
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<frame_rate>();
}

unsafe extern "C" fn known_rate_from_time(mut time: UINT64) -> *const frame_rate {
    let mut i: UINT = 0 as UINT;
    while (i as usize) < ARRAY_SIZE!(known_rates) as usize as usize {
        if time >= known_rates[i as usize].time.wrapping_sub(30 as UINT64)
            && time <= known_rates[i as usize].time.wrapping_add(30 as UINT64)
        {
            return (core::ptr::addr_of!(known_rates) as *const frame_rate).add(i as usize);
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<frame_rate>();
}

#[no_mangle]
pub unsafe extern "system" fn MFFrameRateToAverageTimePerFrame(numerator: UINT32, denominator: UINT32, avgframetime: *mut UINT64) -> HRESULT {
    let rate: UINT64 = ((numerator as UINT64) << 32) | denominator as UINT64;
    let mut entry: *const frame_rate = core::ptr::null();

    entry = known_rate_from_rate(rate);
    if !entry.is_null() {
        *avgframetime = (*entry).time;
    } else {
        *avgframetime = if numerator != 0 {
            (denominator as UINT64).wrapping_mul(10000000 as UINT64).wrapping_div(numerator as UINT64)
        } else {
            0 as UINT64
        };
    }

    return S_OK;
}

unsafe extern "C" fn get_gcd(mut a: u32, mut b: u32) -> u32 {
    let mut m: u32;

    while b != 0 {
        m = a % b;
        a = b;
        b = m;
    }

    return a;
}

#[no_mangle]
pub unsafe extern "system" fn MFAverageTimePerFrameToFrameRate(
    mut avgtime: UINT64,
    numerator: *mut UINT32,
    denominator: *mut UINT32,
) -> HRESULT {
    let mut entry: *const frame_rate = ::core::ptr::null::<frame_rate>();
    let mut gcd: ::core::ffi::c_uint = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFAverageTimePerFrameToFrameRate\0" as *const u8 as *const ::core::ffi::c_char,
            3478 as ::core::ffi::c_int,
            b"%s, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_longlong(avgtime as ULONGLONG),
            numerator,
            denominator,
        );
    }
    entry = known_rate_from_time(avgtime);
    if !entry.is_null() {
        *numerator = ((*entry).rate >> 32) as UINT32;
        *denominator = (*entry).rate as UINT32;
    } else if avgtime != 0 {
        if avgtime > 100000000 as UINT64 {
            avgtime = 100000000 as UINT64;
        }
        gcd = get_gcd(
            10000000 as ::core::ffi::c_uint,
            avgtime as ::core::ffi::c_uint,
        );
        *numerator = (10000000 as ::core::ffi::c_uint).wrapping_div(gcd) as UINT32;
        *denominator = avgtime.wrapping_div(gcd as UINT64) as UINT32;
    } else {
        *denominator = 0 as UINT32;
        *numerator = *denominator;
    }
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFMapDXGIFormatToDX9Format(dxgi_format: DXGI_FORMAT) -> DWORD {
    match dxgi_format {
        DXGI_FORMAT_R32G32B32A32_FLOAT => {
            return D3DFMT_A32B32G32R32F;
        }
        DXGI_FORMAT_R16G16B16A16_FLOAT => {
            return D3DFMT_A16B16G16R16F;
        }
        DXGI_FORMAT_R16G16B16A16_UNORM => {
            return D3DFMT_A16B16G16R16;
        }
        DXGI_FORMAT_R16G16B16A16_SNORM => {
            return D3DFMT_Q16W16V16U16;
        }
        DXGI_FORMAT_R32G32_FLOAT => {
            return D3DFMT_G32R32F;
        }
        DXGI_FORMAT_R10G10B10A2_UNORM => {
            return D3DFMT_A2B10G10R10;
        }
        DXGI_FORMAT_R8G8B8A8_SNORM => {
            return D3DFMT_Q8W8V8U8;
        }
        DXGI_FORMAT_R16G16_FLOAT => {
            return D3DFMT_G16R16F;
        }
        DXGI_FORMAT_R16G16_UNORM => {
            return D3DFMT_G16R16;
        }
        DXGI_FORMAT_R16G16_SNORM => {
            return D3DFMT_V16U16;
        }
        DXGI_FORMAT_D32_FLOAT => {
            return D3DFMT_D32F_LOCKABLE;
        }
        DXGI_FORMAT_R32_FLOAT => {
            return D3DFMT_R32F;
        }
        DXGI_FORMAT_D24_UNORM_S8_UINT => {
            return D3DFMT_D24S8;
        }
        DXGI_FORMAT_R8G8_SNORM => {
            return D3DFMT_V8U8;
        }
        DXGI_FORMAT_R16_FLOAT => {
            return D3DFMT_R16F;
        }
        DXGI_FORMAT_D16_UNORM => {
            return D3DFMT_D16_LOCKABLE;
        }
        DXGI_FORMAT_R16_UNORM => {
            return D3DFMT_L16;
        }
        DXGI_FORMAT_R8_UNORM => {
            return D3DFMT_L8;
        }
        DXGI_FORMAT_A8_UNORM => {
            return D3DFMT_A8;
        }
        DXGI_FORMAT_BC1_UNORM | DXGI_FORMAT_BC1_UNORM_SRGB => {
            return D3DFMT_DXT1;
        }
        DXGI_FORMAT_BC2_UNORM | DXGI_FORMAT_BC2_UNORM_SRGB => {
            return D3DFMT_DXT2;
        }
        DXGI_FORMAT_BC3_UNORM | DXGI_FORMAT_BC3_UNORM_SRGB => {
            return D3DFMT_DXT4;
        }
        DXGI_FORMAT_R8G8B8A8_UNORM => {
            return D3DFMT_A8B8G8R8;
        }
        DXGI_FORMAT_B8G8R8A8_UNORM | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => {
            return D3DFMT_A8R8G8B8;
        }
        DXGI_FORMAT_B8G8R8X8_UNORM | DXGI_FORMAT_B8G8R8X8_UNORM_SRGB => {
            return D3DFMT_X8R8G8B8;
        }
        DXGI_FORMAT_AYUV => {
            return MAKEFOURCC!('A' as i32, 'Y' as i32, 'U' as i32, 'V' as i32);
        }
        DXGI_FORMAT_Y410 => {
            return MAKEFOURCC!('Y' as i32, '4' as i32, '1' as i32, '0' as i32);
        }
        DXGI_FORMAT_Y416 => {
            return MAKEFOURCC!('Y' as i32, '4' as i32, '1' as i32, '6' as i32);
        }
        DXGI_FORMAT_NV12 => {
            return MAKEFOURCC!('N' as i32, 'V' as i32, '1' as i32, '2' as i32);
        }
        DXGI_FORMAT_P010 => {
            return MAKEFOURCC!('P' as i32, '0' as i32, '1' as i32, '0' as i32);
        }
        DXGI_FORMAT_P016 => {
            return MAKEFOURCC!('P' as i32, '0' as i32, '1' as i32, '6' as i32);
        }
        DXGI_FORMAT_420_OPAQUE => {
            return MAKEFOURCC!('4' as i32, '2' as i32, '0' as i32, 'O' as i32);
        }
        DXGI_FORMAT_YUY2 => {
            return D3DFMT_YUY2;
        }
        DXGI_FORMAT_Y210 => {
            return MAKEFOURCC!('Y' as i32, '2' as i32, '1' as i32, '0' as i32);
        }
        DXGI_FORMAT_Y216 => {
            return MAKEFOURCC!('Y' as i32, '2' as i32, '1' as i32, '6' as i32);
        }
        DXGI_FORMAT_NV11 => {
            return MAKEFOURCC!('N' as i32, 'V' as i32, '1' as i32, '1' as i32);
        }
        DXGI_FORMAT_AI44 => {
            return MAKEFOURCC!('A' as i32, 'I' as i32, '4' as i32, '4' as i32);
        }
        DXGI_FORMAT_IA44 => {
            return MAKEFOURCC!('I' as i32, 'A' as i32, '4' as i32, '4' as i32);
        }
        DXGI_FORMAT_P8 => {
            return D3DFMT_P8;
        }
        DXGI_FORMAT_A8P8 => {
            return D3DFMT_A8P8;
        }
        _ => {
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MFMapDX9FormatToDXGIFormat(format: DWORD) -> DXGI_FORMAT {
    if format == D3DFMT_A32B32G32R32F {
        return DXGI_FORMAT_R32G32B32A32_FLOAT;
    } else if format == D3DFMT_A16B16G16R16F {
        return DXGI_FORMAT_R16G16B16A16_FLOAT;
    } else if format == D3DFMT_A16B16G16R16 {
        return DXGI_FORMAT_R16G16B16A16_UNORM;
    } else if format == D3DFMT_Q16W16V16U16 {
        return DXGI_FORMAT_R16G16B16A16_SNORM;
    } else if format == D3DFMT_G32R32F {
        return DXGI_FORMAT_R32G32_FLOAT;
    } else if format == D3DFMT_A2B10G10R10 {
        return DXGI_FORMAT_R10G10B10A2_UNORM;
    } else if format == D3DFMT_Q8W8V8U8 {
        return DXGI_FORMAT_R8G8B8A8_SNORM;
    } else if format == D3DFMT_G16R16F {
        return DXGI_FORMAT_R16G16_FLOAT;
    } else if format == D3DFMT_G16R16 {
        return DXGI_FORMAT_R16G16_UNORM;
    } else if format == D3DFMT_V16U16 {
        return DXGI_FORMAT_R16G16_SNORM;
    } else if format == D3DFMT_D32F_LOCKABLE {
        return DXGI_FORMAT_D32_FLOAT;
    } else if format == D3DFMT_R32F {
        return DXGI_FORMAT_R32_FLOAT;
    } else if format == D3DFMT_D24S8 {
        return DXGI_FORMAT_D24_UNORM_S8_UINT;
    } else if format == D3DFMT_V8U8 {
        return DXGI_FORMAT_R8G8_SNORM;
    } else if format == D3DFMT_R16F {
        return DXGI_FORMAT_R16_FLOAT;
    } else if format == D3DFMT_L16 {
        return DXGI_FORMAT_R16_UNORM;
    } else if format == D3DFMT_L8 {
        return DXGI_FORMAT_R8_UNORM;
    } else if format == D3DFMT_A8 {
        return DXGI_FORMAT_A8_UNORM;
    } else if format == D3DFMT_DXT1 {
        return DXGI_FORMAT_BC1_UNORM;
    } else if format == D3DFMT_DXT2 {
        return DXGI_FORMAT_BC2_UNORM;
    } else if format == D3DFMT_DXT4 {
        return DXGI_FORMAT_BC3_UNORM;
    } else if format == D3DFMT_A8R8G8B8 {
        return DXGI_FORMAT_B8G8R8A8_UNORM;
    } else if format == D3DFMT_X8R8G8B8 {
        return DXGI_FORMAT_B8G8R8X8_UNORM;
    } else if format == D3DFMT_A8B8G8R8 {
        return DXGI_FORMAT_R8G8B8A8_UNORM;
    } else if format == MAKEFOURCC!('A', 'Y', 'U', 'V') {
        return DXGI_FORMAT_AYUV;
    } else if format == MAKEFOURCC!('Y', '4', '1', '0') {
        return DXGI_FORMAT_Y410;
    } else if format == MAKEFOURCC!('Y', '4', '1', '6') {
        return DXGI_FORMAT_Y416;
    } else if format == MAKEFOURCC!('N', 'V', '1', '2') {
        return DXGI_FORMAT_NV12;
    } else if format == MAKEFOURCC!('P', '0', '1', '0') {
        return DXGI_FORMAT_P010;
    } else if format == MAKEFOURCC!('P', '0', '1', '6') {
        return DXGI_FORMAT_P016;
    } else if format == MAKEFOURCC!('4', '2', '0', 'O') {
        return DXGI_FORMAT_420_OPAQUE;
    } else if format == D3DFMT_YUY2 {
        return DXGI_FORMAT_YUY2;
    } else if format == MAKEFOURCC!('Y', '2', '1', '0') {
        return DXGI_FORMAT_Y210;
    } else if format == MAKEFOURCC!('Y', '2', '1', '6') {
        return DXGI_FORMAT_Y216;
    } else if format == MAKEFOURCC!('N', 'V', '1', '1') {
        return DXGI_FORMAT_NV11;
    } else if format == MAKEFOURCC!('A', 'I', '4', '4') {
        return DXGI_FORMAT_AI44;
    } else if format == MAKEFOURCC!('I', 'A', '4', '4') {
        return DXGI_FORMAT_IA44;
    } else if format == D3DFMT_P8 {
        return DXGI_FORMAT_P8;
    } else if format == D3DFMT_A8P8 {
        return DXGI_FORMAT_A8P8;
    } else {
        return DXGI_FORMAT_UNKNOWN;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MFInitVideoFormat_RGB(
    mut format: *mut MFVIDEOFORMAT,
    mut width: DWORD,
    mut height: DWORD,
    mut d3dformat: DWORD,
) -> HRESULT {
    let mut transfer_function: ::core::ffi::c_uint = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFInitVideoFormat_RGB\0" as *const u8 as *const ::core::ffi::c_char,
            3695 as ::core::ffi::c_int,
            b"%p, %lu, %lu, %#lx.\n\0" as *const u8 as *const ::core::ffi::c_char,
            format,
            width,
            height,
            d3dformat,
        );
    }
    if format.is_null() {
        return E_INVALIDARG;
    }
    if d3dformat == 0 {
        d3dformat = D3DFMT_X8R8G8B8;
    }
    if d3dformat == D3DFMT_X8R8G8B8
        || d3dformat == D3DFMT_R8G8B8
        || d3dformat == D3DFMT_A8R8G8B8
        || d3dformat == D3DFMT_R5G6B5
        || d3dformat == D3DFMT_X1R5G5B5
        || d3dformat == D3DFMT_A2B10G10R10
        || d3dformat == D3DFMT_A2R10G10B10
        || d3dformat == D3DFMT_P8
    {
        transfer_function = MFVideoTransFunc_sRGB as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else {
        transfer_function = MFVideoTransFunc_10 as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    memset(
        format as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<MFVIDEOFORMAT>() as size_t,
    );
    (*format).dwSize = ::core::mem::size_of::<MFVIDEOFORMAT>() as DWORD;
    (*format).videoInfo.dwWidth = width;
    (*format).videoInfo.dwHeight = height;
    (*format).videoInfo.PixelAspectRatio.Numerator = 1 as DWORD;
    (*format).videoInfo.PixelAspectRatio.Denominator = 1 as DWORD;
    (*format).videoInfo.InterlaceMode = MFVideoInterlace_Progressive;
    (*format).videoInfo.TransferFunction = transfer_function as MFVideoTransferFunction;
    (*format).videoInfo.ColorPrimaries = MFVideoPrimaries_BT709;
    (*format).videoInfo.SourceLighting = MFVideoLighting_office;
    (*format).videoInfo.FramesPerSecond.Numerator = 60 as DWORD;
    (*format).videoInfo.FramesPerSecond.Denominator = 1 as DWORD;
    (*format).videoInfo.NominalRange = MFNominalRange_Normal;
    (*format).videoInfo.GeometricAperture.Area.cx = width as LONG;
    (*format).videoInfo.GeometricAperture.Area.cy = height as LONG;
    (*format).videoInfo.MinimumDisplayAperture = (*format).videoInfo.GeometricAperture;
    memcpy(
        &raw mut (*format).guidFormat as *mut ::core::ffi::c_void,
        &raw const MFVideoFormat_Base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<GUID>() as size_t,
    );
    (*format).guidFormat.Data1 = d3dformat as ::core::ffi::c_uint;
    (*format).surfaceInfo.Format = d3dformat;
    return S_OK;
}

unsafe extern "C" fn mf_get_stride_for_bitmap_info_header(
    mut fourcc: DWORD,
    mut bih: *const BITMAPINFOHEADER,
    mut stride: *mut LONG,
) -> HRESULT {
    let mut hr: HRESULT;
    hr = MFGetStrideForBitmapInfoHeader(fourcc, (*bih).biWidth as DWORD, stride);
    if hr < 0 {
        return hr;
    }
    if (*bih).biHeight < 0 {
        *stride *= -1;
    }

    return hr;
}

unsafe extern "C" fn get_mf_subtype_for_am_subtype(subtype: *const GUID) -> *const GUID {
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_RGB1) != false {
        return core::ptr::addr_of!(MFVideoFormat_RGB1);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_RGB4) != false {
        return core::ptr::addr_of!(MFVideoFormat_RGB4);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_RGB8) != false {
        return core::ptr::addr_of!(MFVideoFormat_RGB8);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_RGB555) != false {
        return core::ptr::addr_of!(MFVideoFormat_RGB555);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_RGB565) != false {
        return core::ptr::addr_of!(MFVideoFormat_RGB565);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_RGB24) != false {
        return core::ptr::addr_of!(MFVideoFormat_RGB24);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_RGB32) != false {
        return core::ptr::addr_of!(MFVideoFormat_RGB32);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_ARGB1555) != false {
        return core::ptr::addr_of!(MFVideoFormat_ARGB1555);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_ARGB4444) != false {
        return core::ptr::addr_of!(MFVideoFormat_ARGB4444);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_ARGB32) != false {
        return core::ptr::addr_of!(MFVideoFormat_ARGB32);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_A2R10G10B10) != false {
        return core::ptr::addr_of!(MFVideoFormat_A2B10G10R10);
    }
    if IsEqualGUID!(subtype, &MEDIASUBTYPE_A2B10G10R10) != false {
        return core::ptr::addr_of!(MFVideoFormat_A2R10G10B10);
    }
    return subtype;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoMediaType(format: *const MFVIDEOFORMAT, media_type: *mut *mut IMFVideoMediaType) -> HRESULT {
    let mut object: *mut media_type = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if media_type.is_null() {
        return E_INVALIDARG;
    }

    hr = create_media_type(core::ptr::addr_of_mut!(object));
    if hr < 0 {
        return hr;
    }

    hr = MFInitMediaTypeFromMFVideoFormat(core::ptr::addr_of_mut!((*object).IMFMediaType_iface), format, (*format).dwSize);
    if hr < 0 {
        IMFMediaType_Release(core::ptr::addr_of_mut!((*object).IMFMediaType_iface));
        return hr;
    }

    *media_type = core::ptr::addr_of_mut!((*object).IMFVideoMediaType_iface);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoMediaTypeFromVideoInfoHeader(
    vih: *const KS_VIDEOINFOHEADER,
    size: DWORD,
    pixel_aspect_ratio_x: DWORD,
    pixel_aspect_ratio_y: DWORD,
    interlace_mode: MFVideoInterlaceMode,
    video_flags: QWORD,
    subtype: *const GUID,
    ret: *mut *mut IMFVideoMediaType,
) -> HRESULT {
    // FIXME("%p, %lu, %lu, %lu, %d, %I64x, %s, %p.\n", vih, size, pixel_aspect_ratio_x, pixel_aspect_ratio_y, interlace_mode, video_flags, debugstr_guid(subtype), ret);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn MFInitMediaTypeFromMFVideoFormat(
    media_type: *mut IMFMediaType,
    format: *const MFVIDEOFORMAT,
    size: UINT32,
) -> HRESULT {
    let mut stride: UINT32;
    let mut sample_size: UINT32 = 0;
    let mut palette_size: UINT32;
    let mut user_data_size: UINT32;
    let mut value: UINT32;
    let mut video_format: *mut uncompressed_video_format = core::ptr::null_mut();
    let mut user_data: *const core::ffi::c_void = core::ptr::null();
    let mut hr: HRESULT = S_OK;

    if format.is_null()
        || (size as usize) < core::mem::size_of::<MFVIDEOFORMAT>()
        || (*format).dwSize != size
    {
        return E_INVALIDARG;
    }
    if (size as usize)
        < core::mem::offset_of!(MFVIDEOFORMAT, surfaceInfo.Palette)
            + ((*format)
                .surfaceInfo
                .PaletteEntries
                .wrapping_add(1 as DWORD) as usize)
                * core::mem::size_of::<MFPaletteEntry>()
    {
        return E_INVALIDARG;
    }

    mediatype_set_guid(
        media_type,
        &raw const MF_MT_MAJOR_TYPE,
        &raw const MFMediaType_Video,
        &raw mut hr,
    );
    if IsEqualGUID!(
        &raw const (*format).guidFormat,
        &raw const GUID_NULL,
    ) == false
    {
        mediatype_set_guid(
            media_type,
            &raw const MF_MT_SUBTYPE,
            &raw const (*format).guidFormat,
            &raw mut hr,
        );
    }
    video_format = mf_get_video_format(&raw const (*format).guidFormat);
    if !video_format.is_null() {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_FIXED_SIZE_SAMPLES,
            1,
            &raw mut hr,
        );
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_ALL_SAMPLES_INDEPENDENT,
            1,
            &raw mut hr,
        );
    }

    if (*format).videoInfo.dwWidth != 0 && (*format).videoInfo.dwHeight != 0 {
        mediatype_set_uint64(
            media_type,
            &raw const MF_MT_FRAME_SIZE,
            (*format).videoInfo.dwWidth,
            (*format).videoInfo.dwHeight,
            &raw mut hr,
        );

        if !video_format.is_null() {
            stride = mf_get_stride_for_format(video_format, (*format).videoInfo.dwWidth);
            if stride != 0 {
                if (*video_format).yuv == 0
                    && ((*format).videoInfo.VideoFlags
                        & MFVideoFlag_BottomUpLinearRep as UINT64)
                        != 0
                {
                    stride = stride.wrapping_neg();
                }
                mediatype_set_uint32(
                    media_type,
                    &raw const MF_MT_DEFAULT_STRIDE,
                    stride,
                    &raw mut hr,
                );
            }
        }

        if (MFCalculateImageSize(
            &raw const (*format).guidFormat,
            (*format).videoInfo.dwWidth,
            (*format).videoInfo.dwHeight,
            &raw mut sample_size,
        )) >= 0
        {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_SAMPLE_SIZE,
                sample_size,
                &raw mut hr,
            );
        }
    }

    if (*format).videoInfo.PixelAspectRatio.Denominator != 0 {
        mediatype_set_uint64(
            media_type,
            &raw const MF_MT_PIXEL_ASPECT_RATIO,
            (*format).videoInfo.PixelAspectRatio.Numerator,
            (*format).videoInfo.PixelAspectRatio.Denominator,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.SourceChromaSubsampling != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_VIDEO_CHROMA_SITING,
            (*format).videoInfo.SourceChromaSubsampling as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.InterlaceMode != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_INTERLACE_MODE,
            (*format).videoInfo.InterlaceMode as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.TransferFunction != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_TRANSFER_FUNCTION,
            (*format).videoInfo.TransferFunction as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.ColorPrimaries != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_VIDEO_PRIMARIES,
            (*format).videoInfo.ColorPrimaries as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.TransferMatrix != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_YUV_MATRIX,
            (*format).videoInfo.TransferMatrix as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.SourceLighting != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_VIDEO_LIGHTING,
            (*format).videoInfo.SourceLighting as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.FramesPerSecond.Denominator != 0 {
        mediatype_set_uint64(
            media_type,
            &raw const MF_MT_FRAME_RATE,
            (*format).videoInfo.FramesPerSecond.Numerator,
            (*format).videoInfo.FramesPerSecond.Denominator,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.NominalRange != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_VIDEO_NOMINAL_RANGE,
            (*format).videoInfo.NominalRange as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.GeometricAperture.Area.cx != 0
        && (*format).videoInfo.GeometricAperture.Area.cy != 0
    {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_GEOMETRIC_APERTURE,
            &raw const (*format).videoInfo.GeometricAperture as *mut BYTE,
            core::mem::size_of::<MFVideoArea>() as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.MinimumDisplayAperture.Area.cx != 0
        && (*format).videoInfo.MinimumDisplayAperture.Area.cy != 0
    {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_MINIMUM_DISPLAY_APERTURE,
            &raw const (*format).videoInfo.MinimumDisplayAperture as *mut BYTE,
            core::mem::size_of::<MFVideoArea>() as UINT32,
            &raw mut hr,
        );
    }
    if (*format).videoInfo.PanScanAperture.Area.cx != 0
        && (*format).videoInfo.PanScanAperture.Area.cy != 0
    {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_PAN_SCAN_APERTURE,
            &raw const (*format).videoInfo.PanScanAperture as *mut BYTE,
            core::mem::size_of::<MFVideoArea>() as UINT32,
            &raw mut hr,
        );
    }
    value = (((*format).videoInfo.VideoFlags & MFVideoFlag_PanScanEnabled as UINT64) != 0)
        as UINT32;
    if value != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_PAN_SCAN_ENABLED,
            value,
            &raw mut hr,
        );
    }
    value = ((*format).videoInfo.VideoFlags & MFVideoFlag_PAD_TO_Mask as UINT64) as UINT32;
    if value != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_PAD_CONTROL_FLAGS,
            value,
            &raw mut hr,
        );
    }
    value = ((*format).videoInfo.VideoFlags & MFVideoFlag_SrcContentHintMask as UINT64) as UINT32;
    if value != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_SOURCE_CONTENT_HINT,
            value >> 2,
            &raw mut hr,
        );
    }
    value = ((*format).videoInfo.VideoFlags
        & (MFVideoFlag_AnalogProtected as UINT64 | MFVideoFlag_DigitallyProtected as UINT64))
        as UINT32;
    if value != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_DRM_FLAGS,
            value >> 5,
            &raw mut hr,
        );
    }

    if (*format).compressedInfo.AvgBitrate != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_AVG_BITRATE,
            ((*format).compressedInfo.AvgBitrate) as u32,
            &raw mut hr,
        );
    }
    if (*format).compressedInfo.AvgBitErrorRate != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_AVG_BIT_ERROR_RATE,
            ((*format).compressedInfo.AvgBitErrorRate) as u32,
            &raw mut hr,
        );
    }
    if (*format).compressedInfo.MaxKeyFrameSpacing != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_MAX_KEYFRAME_SPACING,
            (*format).compressedInfo.MaxKeyFrameSpacing,
            &raw mut hr,
        );
    }

    palette_size = ((*format).surfaceInfo.PaletteEntries as usize)
        .wrapping_mul(core::mem::size_of::<MFPaletteEntry>() as usize) as UINT32;
    if palette_size == 0 {
        user_data = format.add(1 as usize) as *const core::ffi::c_void;
    } else {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_PALETTE,
            &raw const (*format).surfaceInfo.Palette as *const MFPaletteEntry as *mut BYTE,
            palette_size,
            &raw mut hr,
        );
        user_data = (&raw const (*format).surfaceInfo.Palette as *const MFPaletteEntry).add(
            (*format)
                .surfaceInfo
                .PaletteEntries
                .wrapping_add(1 as DWORD) as usize,
        ) as *const core::ffi::c_void;
    }

    user_data_size = (format as *const BYTE)
        .add((*format).dwSize as usize)
        .offset_from(user_data as *const BYTE) as UINT32;
    if user_data_size != 0 {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_USER_DATA,
            (user_data).cast(),
            user_data_size,
            &raw mut hr,
        );
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFInitMediaTypeFromVideoInfoHeader2(
    media_type: *mut IMFMediaType,
    vih: *const VIDEOINFOHEADER2,
    size: UINT32,
    mut subtype: *const GUID,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut height: DWORD;
    let mut stride: LONG = core::mem::zeroed();

    TRACE!("%p, %p, %u, %s.\n", media_type, vih, size, debugstr_guid(subtype));

    IMFMediaType_DeleteAllItems(media_type);

    if subtype.is_null() {
        match (*vih).bmiHeader.biBitCount as i32 {
            1 => {
                subtype = &raw const MFVideoFormat_RGB1;
            }
            4 => {
                subtype = &raw const MFVideoFormat_RGB4;
            }
            8 => {
                subtype = &raw const MFVideoFormat_RGB8;
            }
            16 => {
                subtype = &raw const MFVideoFormat_RGB555;
            }
            24 => {
                subtype = &raw const MFVideoFormat_RGB24;
            }
            32 => {
                subtype = &raw const MFVideoFormat_RGB32;
            }
            _ => return E_INVALIDARG,
        }
    }

    height = abs((*vih).bmiHeader.biHeight as i32) as DWORD;

    mediatype_set_guid(
        media_type,
        &raw const MF_MT_MAJOR_TYPE,
        &raw const MFMediaType_Video,
        &raw mut hr,
    );
    mediatype_set_guid(media_type, &raw const MF_MT_SUBTYPE, subtype, &raw mut hr);
    mediatype_set_uint64(
        media_type,
        &raw const MF_MT_PIXEL_ASPECT_RATIO,
        1 as u32,
        1 as u32,
        &raw mut hr,
    );
    if (*vih).bmiHeader.biWidth != 0 || height != 0 {
        mediatype_set_uint64(
            media_type,
            &raw const MF_MT_FRAME_SIZE,
            (*vih).bmiHeader.biWidth as u32,
            height,
            &raw mut hr,
        );
    }

    if mf_get_stride_for_bitmap_info_header(
        (*subtype).Data1 as DWORD,
        &raw const (*vih).bmiHeader,
        &raw mut stride,
    ) >= 0 as HRESULT
    {
        if stride != 0 {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_DEFAULT_STRIDE,
                stride as u32,
                &raw mut hr,
            );
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_SAMPLE_SIZE,
                (abs(stride) as DWORD).wrapping_mul(height) as u32,
                &raw mut hr,
            );
        }
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_FIXED_SIZE_SAMPLES,
            1 as u32,
            &raw mut hr,
        );
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_ALL_SAMPLES_INDEPENDENT,
            1 as u32,
            &raw mut hr,
        );
    }

    if (*vih).bmiHeader.biSizeImage != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_SAMPLE_SIZE,
            (*vih).bmiHeader.biSizeImage as u32,
            &raw mut hr,
        );
    }

    if (*vih).rcSource.left != 0
        || (*vih).rcSource.top != 0
        || (*vih).rcSource.right != 0
        || (*vih).rcSource.bottom != 0
    {
        let mut aperture: MFVideoArea = _MFVideoArea {
            OffsetX: _MFOffset {
                fract: 0 as WORD,
                value: 0,
            },
            OffsetY: _MFOffset { fract: 0, value: 0 },
            Area: tagSIZE { cx: 0, cy: 0 },
        };

        aperture.OffsetX.value = (*vih).rcSource.left as i16;
        aperture.OffsetY.value = (*vih).rcSource.top as i16;
        aperture.Area.cx = (*vih).rcSource.right - (*vih).rcSource.left;
        aperture.Area.cy = (*vih).rcSource.bottom - (*vih).rcSource.top;

        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_PAN_SCAN_ENABLED,
            1 as u32,
            &raw mut hr,
        );
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_PAN_SCAN_APERTURE,
            &raw mut aperture as *mut BYTE,
            core::mem::size_of::<MFVideoArea>() as u32,
            &raw mut hr,
        );
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_MINIMUM_DISPLAY_APERTURE,
            &raw mut aperture as *mut BYTE,
            core::mem::size_of::<MFVideoArea>() as u32,
            &raw mut hr,
        );
    }

    if hr >= 0 as HRESULT && (*vih).AvgTimePerFrame != 0 {
        let mut num: UINT32 = 0;
        let mut den: UINT32 = 0;
        hr = MFAverageTimePerFrameToFrameRate((*vih).AvgTimePerFrame as UINT64, &raw mut num, &raw mut den);
        if hr >= 0 as HRESULT {
            mediatype_set_uint64(
                media_type,
                &raw const MF_MT_FRAME_RATE,
                num as u32,
                den as u32,
                &raw mut hr,
            );
        }
    }

    if (*vih).__bindgen_anon_1.dwControlFlags & AMCONTROL_COLORINFO_PRESENT as DWORD != 0 {
        let format: *mut DXVA_ExtendedFormat =
            &raw const (*vih).__bindgen_anon_1.dwControlFlags as *mut DXVA_ExtendedFormat;

        if (*format).VideoChromaSubsampling() != 0 {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_VIDEO_CHROMA_SITING,
                (*format).VideoChromaSubsampling() as u32,
                &raw mut hr,
            );
        }
        if (*format).NominalRange() as u64 != 0 {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_VIDEO_NOMINAL_RANGE,
                (*format).NominalRange() as u32,
                &raw mut hr,
            );
        }
        if (*format).VideoTransferMatrix() as u64 != 0 {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_YUV_MATRIX,
                (*format).VideoTransferMatrix() as u32,
                &raw mut hr,
            );
        }
        if (*format).VideoLighting() as u64 != 0 {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_VIDEO_LIGHTING,
                (*format).VideoLighting() as u32,
                &raw mut hr,
            );
        }
        if (*format).VideoPrimaries() as u64 != 0 {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_VIDEO_PRIMARIES,
                (*format).VideoPrimaries() as u32,
                &raw mut hr,
            );
        }
        if (*format).VideoTransferFunction() as u64 != 0 {
            mediatype_set_uint32(
                media_type,
                &raw const MF_MT_TRANSFER_FUNCTION,
                (*format).VideoTransferFunction() as u32,
                &raw mut hr,
            );
        }
    }

    if ((*vih).dwInterlaceFlags & AMINTERLACE_IsInterlaced as DWORD) == 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_INTERLACE_MODE,
            MFVideoInterlace_Progressive as i32 as u32,
            &raw mut hr,
        );
    } else if (*vih).dwInterlaceFlags & AMINTERLACE_DisplayModeBobOrWeave as DWORD != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_INTERLACE_MODE,
            MFVideoInterlace_MixedInterlaceOrProgressive as i32 as u32,
            &raw mut hr,
        );
    } else {
        FIXME!(
            "dwInterlaceFlags %#lx not implemented\n",
            (*vih).dwInterlaceFlags
        );
    }

    if size as usize > core::mem::size_of::<VIDEOINFOHEADER2>() {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_USER_DATA,
            vih.offset(1 as isize) as *mut BYTE,
            (size as usize).wrapping_sub(core::mem::size_of::<VIDEOINFOHEADER2>()) as u32,
            &raw mut hr,
        );
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFInitMediaTypeFromVideoInfoHeader(
    media_type: *mut IMFMediaType,
    vih: *const VIDEOINFOHEADER,
    size: UINT32,
    subtype: *const GUID,
) -> HRESULT {
    let mut vih2: VIDEOINFOHEADER2 = core::mem::zeroed();
    vih2.rcSource = (*vih).rcSource;
    vih2.rcTarget = (*vih).rcTarget;
    vih2.dwBitRate = (*vih).dwBitRate;
    vih2.dwBitErrorRate = (*vih).dwBitErrorRate;
    vih2.AvgTimePerFrame = (*vih).AvgTimePerFrame as REFERENCE_TIME;
    vih2.bmiHeader = (*vih).bmiHeader;
    let mut hr: HRESULT;

    hr = MFInitMediaTypeFromVideoInfoHeader2(
        media_type,
        &raw mut vih2,
        core::mem::size_of::<VIDEOINFOHEADER2>() as UINT32,
        subtype,
    );
    if size as usize > core::mem::size_of::<VIDEOINFOHEADER>() {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_USER_DATA,
            vih.add(1) as *mut BYTE,
            (size as usize - core::mem::size_of::<VIDEOINFOHEADER>()) as UINT32,
            &raw mut hr,
        );
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFInitMediaTypeFromMPEG1VideoInfo(
    media_type: *mut IMFMediaType,
    vih: *const MPEG1VIDEOINFO,
    size: UINT32,
    subtype: *const GUID,
) -> HRESULT {
    let mut hr: HRESULT = 0;

    TRACE!("%p, %p, %u, %s.\n", media_type, vih, size, debugstr_guid(subtype));

    hr = MFInitMediaTypeFromVideoInfoHeader(
        media_type,
        &raw const (*vih).hdr,
        core::mem::size_of::<VIDEOINFOHEADER>() as UINT32,
        subtype,
    );
    if hr < 0 {
        return hr;
    }

    if (*vih).dwStartTimeCode != 0 {
        mediatype_set_uint32(
            media_type,
            &MF_MT_MPEG_START_TIME_CODE,
            (*vih).dwStartTimeCode as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }
    if (*vih).cbSequenceHeader != 0 {
        mediatype_set_blob(
            media_type,
            &MF_MT_MPEG_SEQUENCE_HEADER,
            &raw const (*vih).bSequenceHeader as *const UINT8,
            (*vih).cbSequenceHeader as core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFInitMediaTypeFromMPEG2VideoInfo(
    media_type: *mut IMFMediaType,
    vih: *const MPEG2VIDEOINFO,
    size: UINT32,
    subtype: *const GUID,
) -> HRESULT {
    let mut hr: HRESULT = 0;

    if ({
        hr = MFInitMediaTypeFromVideoInfoHeader2(
            media_type,
            &raw const (*vih).hdr,
            core::mem::size_of::<VIDEOINFOHEADER2>() as UINT32,
            subtype,
        );
        hr
    }) < 0
    {
        return hr;
    }

    if (*vih).dwStartTimeCode != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_MPEG_START_TIME_CODE,
            (*vih).dwStartTimeCode as UINT32,
            &raw mut hr,
        );
    }
    if (*vih).cbSequenceHeader != 0 {
        mediatype_set_blob(
            media_type,
            &raw const MF_MT_MPEG_SEQUENCE_HEADER,
            &raw const (*vih).dwSequenceHeader as *const DWORD as *mut BYTE,
            (*vih).cbSequenceHeader as UINT32,
            &raw mut hr,
        );
    }

    if (*vih).dwProfile != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_MPEG2_PROFILE,
            (*vih).dwProfile as UINT32,
            &raw mut hr,
        );
    }
    if (*vih).dwLevel != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_MPEG2_LEVEL,
            (*vih).dwLevel as UINT32,
            &raw mut hr,
        );
    }
    if (*vih).dwFlags != 0 {
        mediatype_set_uint32(
            media_type,
            &raw const MF_MT_MPEG2_FLAGS,
            (*vih).dwFlags as UINT32,
            &raw mut hr,
        );
    }

    return hr;
}

unsafe extern "C" fn init_am_media_type_audio_format(am_type: *mut AM_MEDIA_TYPE, media_type: *mut IMFMediaType) -> HRESULT {
    let mut hr: HRESULT = 0;

    if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_VideoInfo)
        || IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_VideoInfo2)
        || IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_MFVideoFormat)
    {
        return E_INVALIDARG;
    }

    if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_WaveFormatEx)
        || IsEqualGUID!(&raw const (*am_type).formattype, &raw const GUID_NULL)
    {
        let mut flags: UINT32 = 0;
        let mut num_channels: UINT32 = media_type_get_uint32(media_type, &raw const MF_MT_AUDIO_NUM_CHANNELS);

        if ((IMFMediaType_GetItem(media_type, &raw const MF_MT_AUDIO_CHANNEL_MASK, core::ptr::null_mut())) >= 0)
            || ((IMFMediaType_GetItem(media_type, &raw const MF_MT_AUDIO_VALID_BITS_PER_SAMPLE, core::ptr::null_mut())) >= 0)
            || ((IMFMediaType_GetItem(media_type, &raw const MF_MT_AUDIO_SAMPLES_PER_BLOCK, core::ptr::null_mut())) >= 0)
            || num_channels > 2
        {
            flags = MFWaveFormatExConvertFlag_ForceExtensible;
        }

        hr = MFCreateWaveFormatExFromMFMediaType(
            media_type,
            &raw mut (*am_type).pbFormat as *mut *mut WAVEFORMATEX,
            &raw mut (*am_type).cbFormat as *mut UINT32,
            flags,
        );
        if hr < 0 {
            return hr;
        }

        (*am_type).subtype = get_am_subtype_for_mf_subtype((*am_type).subtype);
        (*am_type).formattype = FORMAT_WaveFormatEx;
    } else {
        WARN!("Unknown format %s\n", debugstr_mf_guid(&raw const (*am_type).formattype));
        (*am_type).formattype = GUID_NULL;
    }

    return S_OK;
}

unsafe extern "C" fn init_video_info_header2(
    vih: *mut VIDEOINFOHEADER2,
    subtype: *const GUID,
    media_type: *mut IMFMediaType,
) {
    let video_format: *mut uncompressed_video_format = mf_get_video_format(subtype);
    let format: *mut DXVA_ExtendedFormat =
        core::ptr::addr_of_mut!((*vih).__bindgen_anon_1.dwControlFlags) as *mut DXVA_ExtendedFormat;
    let mut image_size: UINT32 = 0;
    let mut width: UINT32 = 0;
    let mut height: UINT32 = 0;
    let mut value: UINT32 = 0;
    let mut frame_size: UINT64 = 0;
    let mut frame_rate: UINT64 = 0;

    (*vih).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    (*vih).bmiHeader.biPlanes = 1 as WORD;
    (*vih).bmiHeader.biBitCount = (if !video_format.is_null() {
        (*video_format).bpp as INT32
    } else {
        0 as INT32
    }) as WORD;

    if !video_format.is_null() && (*video_format).compression != -1 {
        (*vih).bmiHeader.biCompression = (*video_format).compression as DWORD;
    } else {
        (*vih).bmiHeader.biCompression = (*subtype).Data1 as DWORD;
    }

    (*vih).dwBitRate =
        media_type_get_uint32(media_type, &raw const MF_MT_AVG_BITRATE) as DWORD;
    (*vih).dwBitErrorRate =
        media_type_get_uint32(media_type, &raw const MF_MT_AVG_BIT_ERROR_RATE) as DWORD;
    if (IMFMediaType_GetUINT64(media_type, &raw const MF_MT_FRAME_RATE, &raw mut frame_rate)
        >= 0)
        && (frame_rate >> 32) != 0
    {
        (*vih).AvgTimePerFrame = round(
            10000000.0f64 * (frame_rate as UINT32 as f64)
                / ((frame_rate >> 32) as f64),
        ) as REFERENCE_TIME;
    }
    (*vih).bmiHeader.biSizeImage =
        media_type_get_uint32(media_type, &raw const MF_MT_SAMPLE_SIZE) as DWORD;

    if IMFMediaType_GetUINT64(media_type, &raw const MF_MT_FRAME_SIZE, &raw mut frame_size)
        >= 0
    {
        let bottom_up: BOOL = (((*vih).bmiHeader.biCompression
            == BI_RGB as DWORD)
            || ((*vih).bmiHeader.biCompression == BI_BITFIELDS as DWORD))
            as BOOL;
        let mut stride: INT32 = 0;

        width = (frame_size >> 32) as UINT32;
        stride = media_type_get_uint32(media_type, &raw const MF_MT_DEFAULT_STRIDE) as INT32;
        if stride == 0 {
            stride = width.wrapping_mul(
                (if bottom_up != 0 {
                    -1i32
                } else {
                    1i32
                }) as UINT32,
            ) as INT32;
        } else if !video_format.is_null() {
            stride /= (*video_format).bpp as INT32 / 8;
        }
        height = frame_size as UINT32;

        (*vih).bmiHeader.biWidth = abs(stride) as LONG;
        (*vih).bmiHeader.biHeight = height.wrapping_mul(
            (if bottom_up != 0 && stride >= 0 {
                -1i32
            } else {
                1i32
            }) as UINT32,
        ) as LONG;

        if MFCalculateImageSize(
            subtype,
            abs(stride) as UINT32,
            height,
            &raw mut image_size,
        ) >= 0
        {
            (*vih).bmiHeader.biSizeImage = image_size as DWORD;
        }

        if ((*vih).bmiHeader.biWidth as UINT32) > width {
            (*vih).rcTarget.right = width as LONG;
            (*vih).rcSource.right = (*vih).rcTarget.right;
            (*vih).rcTarget.bottom = height as LONG;
            (*vih).rcSource.bottom = (*vih).rcTarget.bottom;
        }
    }

    (*format).set_VideoChromaSubsampling(
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_CHROMA_SITING) as UINT,
    );
    (*format).set_NominalRange(
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_NOMINAL_RANGE)
            as DXVA_NominalRange,
    );
    (*format).set_VideoTransferMatrix(
        media_type_get_uint32(media_type, &raw const MF_MT_YUV_MATRIX)
            as DXVA_VideoTransferMatrix,
    );
    (*format).set_VideoLighting(
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_LIGHTING)
            as DXVA_VideoLighting,
    );
    (*format).set_VideoPrimaries(
        media_type_get_uint32(media_type, &raw const MF_MT_VIDEO_PRIMARIES)
            as DXVA_VideoPrimaries,
    );
    (*format).set_VideoTransferFunction(
        media_type_get_uint32(media_type, &raw const MF_MT_TRANSFER_FUNCTION)
            as DXVA_VideoTransferFunction,
    );

    if (*format).VideoChromaSubsampling() as INT32 != 0
        || (*format).NominalRange() as INT32 != 0
        || (*format).VideoTransferMatrix() as INT32 != 0
        || (*format).VideoLighting() as INT32 != 0
        || (*format).VideoPrimaries() as INT32 != 0
        || (*format).VideoTransferFunction() as INT32 != 0
    {
        (*format).set_SampleFormat(AMCONTROL_COLORINFO_PRESENT as UINT);
    }

    value = media_type_get_uint32(media_type, &raw const MF_MT_INTERLACE_MODE);
    match value {
        0 | 2 => {}
        7 => {
            (*vih).dwInterlaceFlags =
                (AMINTERLACE_DisplayModeBobOrWeave | AMINTERLACE_IsInterlaced) as DWORD;
        }
        _ => {
            FIXME!("MF_MT_INTERLACE_MODE %u not implemented!\n", value);
            (*vih).dwInterlaceFlags = AMINTERLACE_IsInterlaced as DWORD;
        }
    }
}

unsafe extern "C" fn init_video_info_header(
    mut vih: *mut VIDEOINFOHEADER,
    mut subtype: *const GUID,
    mut media_type: *mut IMFMediaType,
) {
    let mut vih2: VIDEOINFOHEADER2 = tagVIDEOINFOHEADER2 {
        rcSource: tagRECT {
            left: 0 as LONG,
            top: 0,
            right: 0,
            bottom: 0,
        },
        rcTarget: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        dwBitRate: 0,
        dwBitErrorRate: 0,
        AvgTimePerFrame: 0,
        dwInterlaceFlags: 0,
        dwCopyProtectFlags: 0,
        dwPictAspectRatioX: 0,
        dwPictAspectRatioY: 0,
        __bindgen_anon_1: tagVIDEOINFOHEADER2__bindgen_ty_1 { dwControlFlags: 0 },
        dwReserved2: 0,
        bmiHeader: tagBITMAPINFOHEADER {
            biSize: 0,
            biWidth: 0,
            biHeight: 0,
            biPlanes: 0,
            biBitCount: 0,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
    };
    init_video_info_header2(core::ptr::addr_of_mut!(vih2), subtype, media_type);
    (*vih).rcSource = vih2.rcSource;
    (*vih).rcTarget = vih2.rcTarget;
    (*vih).dwBitRate = vih2.dwBitRate;
    (*vih).dwBitErrorRate = vih2.dwBitErrorRate;
    (*vih).AvgTimePerFrame = vih2.AvgTimePerFrame as REFERENCE_TIME;
    (*vih).bmiHeader = vih2.bmiHeader;
}

unsafe extern "C" fn get_am_media_type_video_format_size(format_type: *const GUID, media_type: *mut IMFMediaType) -> UINT32 {
    if IsEqualGUID!(format_type, core::ptr::addr_of!(FORMAT_VideoInfo)) {
        let mut size: UINT32 = core::mem::size_of::<VIDEOINFOHEADER>() as UINT32;
        let mut user_size: UINT32 = 0;
        if (IMFMediaType_GetBlobSize(media_type, core::ptr::addr_of!(MF_MT_USER_DATA), core::ptr::addr_of_mut!(user_size))) >= 0 {
            size += user_size;
        }
        return size;
    }

    if IsEqualGUID!(format_type, core::ptr::addr_of!(FORMAT_VideoInfo2)) {
        let mut size: UINT32 = core::mem::size_of::<VIDEOINFOHEADER2>() as UINT32;
        let mut user_size: UINT32 = 0;
        if (IMFMediaType_GetBlobSize(media_type, core::ptr::addr_of!(MF_MT_USER_DATA), core::ptr::addr_of_mut!(user_size))) >= 0 {
            size += user_size;
        }
        return size;
    }

    if IsEqualGUID!(format_type, core::ptr::addr_of!(FORMAT_MPEGVideo)) {
        let mut size: UINT32 = core::mem::size_of::<MPEG1VIDEOINFO>() as UINT32;
        let mut sequence_size: UINT32 = 0;
        if (IMFMediaType_GetBlobSize(media_type, core::ptr::addr_of!(MF_MT_MPEG_SEQUENCE_HEADER), core::ptr::addr_of_mut!(sequence_size))) >= 0 {
            size += sequence_size;
        }
        return size;
    }

    if IsEqualGUID!(format_type, core::ptr::addr_of!(FORMAT_MPEG2Video)) {
        let mut size: UINT32 = core::mem::size_of::<MPEG2VIDEOINFO>() as UINT32;
        let mut sequence_size: UINT32 = 0;
        if (IMFMediaType_GetBlobSize(media_type, core::ptr::addr_of!(MF_MT_MPEG_SEQUENCE_HEADER), core::ptr::addr_of_mut!(sequence_size))) >= 0 {
            size += sequence_size;
        }
        return size;
    }

    return 0;
}

unsafe extern "C" fn init_am_media_type_video_format(
    am_type: *mut AM_MEDIA_TYPE,
    media_type: *mut IMFMediaType,
) -> HRESULT {
    let video_format: *mut uncompressed_video_format =
        mf_get_video_format(&raw mut (*am_type).subtype);
    let mut hr: HRESULT;

    if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_WaveFormatEx) {
        return E_INVALIDARG;
    }

    if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_MFVideoFormat) {
        return MFCreateMFVideoFormatFromMFMediaType(
            media_type,
            &raw mut (*am_type).pbFormat as *mut *mut MFVIDEOFORMAT,
            &raw mut (*am_type).cbFormat as *mut UINT32,
        );
    }

    if IsEqualGUID!(&raw const (*am_type).formattype, &raw const GUID_NULL) {
        if IsEqualGUID!(&raw const (*am_type).subtype, &raw const MEDIASUBTYPE_MPEG1Payload)
            || IsEqualGUID!(&raw const (*am_type).subtype, &raw const MEDIASUBTYPE_MPEG1Packet)
        {
            (*am_type).formattype = FORMAT_MPEGVideo;
        } else if IsEqualGUID!(&raw const (*am_type).subtype, &raw const MEDIASUBTYPE_MPEG2_VIDEO) {
            (*am_type).formattype = FORMAT_MPEG2Video;
        } else {
            (*am_type).formattype = FORMAT_VideoInfo;
        }
    }

    (*am_type).cbFormat = get_am_media_type_video_format_size(&raw const (*am_type).formattype, media_type);
    (*am_type).pbFormat = CoTaskMemAlloc(((*am_type).cbFormat) as SIZE_T) as *mut BYTE;
    if (*am_type).pbFormat.is_null() {
        return E_OUTOFMEMORY;
    }
    memset(
        (*am_type).pbFormat as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (*am_type).cbFormat as usize,
    );

    if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_VideoInfo) {
        let format: *mut VIDEOINFOHEADER = (*am_type).pbFormat as *mut VIDEOINFOHEADER;
        init_video_info_header(format, &raw mut (*am_type).subtype, media_type);

        if (*am_type).cbFormat as usize > ::core::mem::size_of::<VIDEOINFOHEADER>() as usize && {
            hr = IMFMediaType_GetBlob(
                media_type,
                &raw const MF_MT_USER_DATA,
                format.add(1 as usize) as *mut UINT8,
                ((*am_type).cbFormat as usize)
                    .wrapping_sub(::core::mem::size_of::<VIDEOINFOHEADER>() as usize)
                    as UINT32,
                ::core::ptr::null_mut::<UINT32>(),
            );
            hr < 0
        } {
            return hr;
        }
        (*format).bmiHeader.biSize = ((*format).bmiHeader.biSize as u64)
            .wrapping_add(
                ((*am_type).cbFormat as usize)
                    .wrapping_sub(::core::mem::size_of::<VIDEOINFOHEADER>() as usize)
                    as u64,
            ) as DWORD;

        (*am_type).subtype = get_am_subtype_for_mf_subtype((*am_type).subtype);
        (*am_type).bFixedSizeSamples = (!video_format.is_null()) as ::core::ffi::c_int as BOOL;
        (*am_type).bTemporalCompression = (video_format.is_null()) as ::core::ffi::c_int as BOOL;
    } else if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_VideoInfo2) {
        let format: *mut VIDEOINFOHEADER2 = (*am_type).pbFormat as *mut VIDEOINFOHEADER2;
        init_video_info_header2(format, &raw mut (*am_type).subtype, media_type);

        if (*am_type).cbFormat as usize > ::core::mem::size_of::<VIDEOINFOHEADER2>() as usize && {
            hr = IMFMediaType_GetBlob(
                media_type,
                &raw const MF_MT_USER_DATA,
                format.add(1 as usize) as *mut UINT8,
                ((*am_type).cbFormat as usize)
                    .wrapping_sub(::core::mem::size_of::<VIDEOINFOHEADER2>() as usize)
                    as UINT32,
                ::core::ptr::null_mut::<UINT32>(),
            );
            hr < 0
        } {
            return hr;
        }
        (*format).bmiHeader.biSize = ((*format).bmiHeader.biSize as u64)
            .wrapping_add(
                ((*am_type).cbFormat as usize)
                    .wrapping_sub(::core::mem::size_of::<VIDEOINFOHEADER2>() as usize)
                    as u64,
            ) as DWORD;

        (*am_type).subtype = get_am_subtype_for_mf_subtype((*am_type).subtype);
        (*am_type).bFixedSizeSamples = (!video_format.is_null()) as ::core::ffi::c_int as BOOL;
        (*am_type).bTemporalCompression = (video_format.is_null()) as ::core::ffi::c_int as BOOL;
    } else if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_MPEGVideo) {
        let format: *mut MPEG1VIDEOINFO = (*am_type).pbFormat as *mut MPEG1VIDEOINFO;

        init_video_info_header(&raw mut (*format).hdr, &raw mut (*am_type).subtype, media_type);
        (*format).hdr.bmiHeader.biSize = 0 as DWORD;

        (*format).dwStartTimeCode =
            media_type_get_uint32(media_type, &raw const MF_MT_MPEG_START_TIME_CODE) as DWORD;

        if (*am_type).cbFormat as usize > ::core::mem::size_of::<MPEG1VIDEOINFO>() as usize && {
            hr = IMFMediaType_GetBlob(
                media_type,
                &raw const MF_MT_MPEG_SEQUENCE_HEADER,
                &raw mut (*format).bSequenceHeader as *mut UINT8,
                ((*am_type).cbFormat as usize)
                    .wrapping_sub(::core::mem::size_of::<MPEG1VIDEOINFO>() as usize)
                    as UINT32,
                ::core::ptr::null_mut::<UINT32>(),
            );
            hr < 0
        } {
            return hr;
        }
        (*format).cbSequenceHeader = ((*am_type).cbFormat as usize)
            .wrapping_sub(::core::mem::size_of::<MPEG1VIDEOINFO>() as usize)
            as DWORD;

        (*am_type).subtype = get_am_subtype_for_mf_subtype((*am_type).subtype);
        (*am_type).bFixedSizeSamples = (!video_format.is_null()) as ::core::ffi::c_int as BOOL;
        (*am_type).bTemporalCompression = (video_format.is_null()) as ::core::ffi::c_int as BOOL;
    } else if IsEqualGUID!(&raw const (*am_type).formattype, &raw const FORMAT_MPEG2Video) {
        let format: *mut MPEG2VIDEOINFO = (*am_type).pbFormat as *mut MPEG2VIDEOINFO;

        init_video_info_header2(&raw mut (*format).hdr, &raw mut (*am_type).subtype, media_type);

        (*format).dwStartTimeCode =
            media_type_get_uint32(media_type, &raw const MF_MT_MPEG_START_TIME_CODE) as DWORD;
        (*format).dwProfile =
            media_type_get_uint32(media_type, &raw const MF_MT_MPEG2_PROFILE) as DWORD;
        (*format).dwLevel =
            media_type_get_uint32(media_type, &raw const MF_MT_MPEG2_LEVEL) as DWORD;
        (*format).dwFlags =
            media_type_get_uint32(media_type, &raw const MF_MT_MPEG2_FLAGS) as DWORD;

        if (*am_type).cbFormat as usize > ::core::mem::size_of::<MPEG2VIDEOINFO>() as usize && {
            hr = IMFMediaType_GetBlob(
                media_type,
                &raw const MF_MT_MPEG_SEQUENCE_HEADER,
                &raw mut (*format).dwSequenceHeader as *mut DWORD as *mut UINT8,
                ((*am_type).cbFormat as usize)
                    .wrapping_sub(::core::mem::size_of::<MPEG2VIDEOINFO>() as usize)
                    as UINT32,
                ::core::ptr::null_mut::<UINT32>(),
            );
            hr < 0
        } {
            return hr;
        }
        (*format).cbSequenceHeader = ((*am_type).cbFormat as usize)
            .wrapping_sub(::core::mem::size_of::<MPEG2VIDEOINFO>() as usize)
            as DWORD;

        (*am_type).subtype = get_am_subtype_for_mf_subtype((*am_type).subtype);
        (*am_type).bFixedSizeSamples = (!video_format.is_null()) as ::core::ffi::c_int as BOOL;
        (*am_type).bTemporalCompression = (video_format.is_null()) as ::core::ffi::c_int as BOOL;
    } else {
        WARN!(
            "Unknown format %s\n",
            debugstr_mf_guid(&raw mut (*am_type).formattype)
        );
        (*am_type).formattype = GUID_NULL;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFInitAMMediaTypeFromMFMediaType(
    media_type: *mut IMFMediaType,
    format: GUID,
    am_type: *mut AM_MEDIA_TYPE,
) -> HRESULT {
    let mut hr: HRESULT = 0;

    DPRINT1!("%p, %s, %p.\n", media_type, debugstr_mf_guid(core::ptr::addr_of!(format)), am_type);

    memset(am_type as *mut core::ffi::c_void, 0, core::mem::size_of::<AM_MEDIA_TYPE>() as size_t);
    (*am_type).formattype = format;

  'done: {
        hr = IMFMediaType_GetMajorType(media_type, core::ptr::addr_of_mut!((*am_type).majortype));
        if hr < 0
            || {
                hr = IMFMediaType_GetGUID(media_type, core::ptr::addr_of!(MF_MT_SUBTYPE), core::ptr::addr_of_mut!((*am_type).subtype));
                hr < 0
            }
        {
            break 'done;
        }

        (*am_type).bTemporalCompression =
            (media_type_get_uint32(media_type, core::ptr::addr_of!(MF_MT_ALL_SAMPLES_INDEPENDENT)) == 0) as BOOL;
        (*am_type).bFixedSizeSamples =
            media_type_get_uint32(media_type, core::ptr::addr_of!(MF_MT_FIXED_SIZE_SAMPLES)) as BOOL;
        (*am_type).lSampleSize =
            media_type_get_uint32(media_type, core::ptr::addr_of!(MF_MT_SAMPLE_SIZE)) as ULONG;

        if IsEqualGUID!(core::ptr::addr_of!((*am_type).majortype), core::ptr::addr_of!(MFMediaType_Audio)) {
            hr = init_am_media_type_audio_format(am_type, media_type);
        } else if IsEqualGUID!(core::ptr::addr_of!((*am_type).majortype), core::ptr::addr_of!(MFMediaType_Video)) {
            hr = init_am_media_type_video_format(am_type, media_type);
        } else {
            DPRINT1!("Not implemented!\n");
            hr = E_NOTIMPL;
        }
    }

    if hr < 0 {
        CoTaskMemFree((*am_type).pbFormat as LPVOID);
        (*am_type).pbFormat = core::ptr::null_mut();
        (*am_type).cbFormat = 0;
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFInitMediaTypeFromAMMediaType(media_type: *mut IMFMediaType, am_type: *const AM_MEDIA_TYPE) -> HRESULT {
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
            b"dll/win32/mfplat/mediatype.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFInitMediaTypeFromAMMediaType\0" as *const u8 as *const ::core::ffi::c_char,
            4401 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            media_type,
            am_type,
        );
    }

    IMFMediaType_DeleteAllItems(media_type);

    if IsEqualGUID!(
        core::ptr::addr_of!((*am_type).majortype),
        core::ptr::addr_of!(MEDIATYPE_Video),
    ) != false
    {
        let subtype: *const GUID = get_mf_subtype_for_am_subtype(core::ptr::addr_of!((*am_type).subtype));

        if (*am_type).cbFormat != 0 && (*am_type).pbFormat.is_null() {
            hr = E_INVALIDARG;
        } else if IsEqualGUID!(
            core::ptr::addr_of!((*am_type).formattype),
            core::ptr::addr_of!(FORMAT_VideoInfo),
        ) != false
            && (*am_type).cbFormat as usize >= core::mem::size_of::<VIDEOINFOHEADER>()
        {
            hr = MFInitMediaTypeFromVideoInfoHeader(
                media_type,
                (*am_type).pbFormat as *mut VIDEOINFOHEADER,
                (*am_type).cbFormat as UINT32,
                subtype,
            );
        } else if IsEqualGUID!(
            core::ptr::addr_of!((*am_type).formattype),
            core::ptr::addr_of!(FORMAT_VideoInfo2),
        ) != false
            && (*am_type).cbFormat as usize >= core::mem::size_of::<VIDEOINFOHEADER2>()
        {
            hr = MFInitMediaTypeFromVideoInfoHeader2(
                media_type,
                (*am_type).pbFormat as *mut VIDEOINFOHEADER2,
                (*am_type).cbFormat as UINT32,
                subtype,
            );
        } else if IsEqualGUID!(
            core::ptr::addr_of!((*am_type).formattype),
            core::ptr::addr_of!(FORMAT_MPEGVideo),
        ) != false
            && (*am_type).cbFormat as usize >= core::mem::size_of::<MPEG1VIDEOINFO>()
        {
            hr = MFInitMediaTypeFromMPEG1VideoInfo(
                media_type,
                (*am_type).pbFormat as *mut MPEG1VIDEOINFO,
                (*am_type).cbFormat as UINT32,
                subtype,
            );
        } else if IsEqualGUID!(
            core::ptr::addr_of!((*am_type).formattype),
            core::ptr::addr_of!(FORMAT_MPEG2Video),
        ) != false
            && (*am_type).cbFormat as usize >= core::mem::size_of::<MPEG2VIDEOINFO>()
        {
            hr = MFInitMediaTypeFromMPEG2VideoInfo(
                media_type,
                (*am_type).pbFormat as *mut MPEG2VIDEOINFO,
                (*am_type).cbFormat as UINT32,
                subtype,
            );
        } else if IsEqualGUID!(
            core::ptr::addr_of!((*am_type).formattype),
            core::ptr::addr_of!(FORMAT_MFVideoFormat),
        ) != false
            && (*am_type).cbFormat as usize >= core::mem::size_of::<MFVIDEOFORMAT>()
        {
            hr = MFInitMediaTypeFromMFVideoFormat(
                media_type,
                (*am_type).pbFormat as *mut MFVIDEOFORMAT,
                (*am_type).cbFormat as UINT32,
            );
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    (__dbcl_0) as u32,
                    __dbch_0,
                    b"dll/win32/mfplat/mediatype.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"MFInitMediaTypeFromAMMediaType\0" as *const u8
                        as *const ::core::ffi::c_char,
                    4428 as ::core::ffi::c_int,
                    b"Unsupported format type %s / size %ld.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    debugstr_guid(core::ptr::addr_of!((*am_type).formattype)),
                    (*am_type).cbFormat,
                );
            }
            return E_NOTIMPL;
        }
    } else if IsEqualGUID!(
        core::ptr::addr_of!((*am_type).majortype),
        core::ptr::addr_of!(MEDIATYPE_Audio),
    ) != false
    {
        if (*am_type).cbFormat != 0 && (*am_type).pbFormat.is_null() {
            hr = E_INVALIDARG;
        } else if IsEqualGUID!(
            core::ptr::addr_of!((*am_type).formattype),
            core::ptr::addr_of!(FORMAT_WaveFormatEx),
        ) != false
            && (*am_type).cbFormat as usize >= core::mem::size_of::<WAVEFORMATEX>()
        {
            hr = MFInitMediaTypeFromWaveFormatEx(
                media_type,
                (*am_type).pbFormat as *mut WAVEFORMATEX,
                (*am_type).cbFormat as UINT32,
            );
        } else if IsEqualGUID!(
            core::ptr::addr_of!((*am_type).formattype),
            core::ptr::addr_of!(GUID_NULL),
        ) != false
        {
            mediatype_set_guid(
                media_type,
                core::ptr::addr_of!(MF_MT_MAJOR_TYPE),
                core::ptr::addr_of!(MFMediaType_Audio),
                core::ptr::addr_of_mut!(hr),
            );
            mediatype_set_guid(
                media_type,
                core::ptr::addr_of!(MF_MT_SUBTYPE),
                core::ptr::addr_of!((*am_type).subtype),
                core::ptr::addr_of_mut!(hr),
            );
            mediatype_set_guid(
                media_type,
                core::ptr::addr_of!(MF_MT_AM_FORMAT_TYPE),
                core::ptr::addr_of!(GUID_NULL),
                core::ptr::addr_of_mut!(hr),
            );
            mediatype_set_uint32(
                media_type,
                core::ptr::addr_of!(MF_MT_ALL_SAMPLES_INDEPENDENT),
                1 as ::core::ffi::c_uint,
                core::ptr::addr_of_mut!(hr),
            );
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    (__dbcl_1) as u32,
                    __dbch_1,
                    b"dll/win32/mfplat/mediatype.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"MFInitMediaTypeFromAMMediaType\0" as *const u8
                        as *const ::core::ffi::c_char,
                    4448 as ::core::ffi::c_int,
                    b"Unsupported format type %s / size %ld.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    debugstr_guid(core::ptr::addr_of!((*am_type).formattype)),
                    (*am_type).cbFormat,
                );
            }
            return E_NOTIMPL;
        }
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl_2) as u32,
                __dbch_2,
                b"dll/win32/mfplat/mediatype.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MFInitMediaTypeFromAMMediaType\0" as *const u8 as *const ::core::ffi::c_char,
                4454 as ::core::ffi::c_int,
                b"Unsupported major type %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_guid(core::ptr::addr_of!((*am_type).majortype)),
            );
        }
        return E_NOTIMPL;
    }

    if (*am_type).bTemporalCompression == 0
        && (IMFMediaType_GetItem(
            media_type,
            core::ptr::addr_of!(MF_MT_ALL_SAMPLES_INDEPENDENT),
            core::ptr::null_mut::<PROPVARIANT>(),
        )) < 0
    {
        mediatype_set_uint32(
            media_type,
            core::ptr::addr_of!(MF_MT_ALL_SAMPLES_INDEPENDENT),
            1 as ::core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }
    if (*am_type).bFixedSizeSamples != 0
        && (IMFMediaType_GetItem(
            media_type,
            core::ptr::addr_of!(MF_MT_FIXED_SIZE_SAMPLES),
            core::ptr::null_mut::<PROPVARIANT>(),
        )) < 0
    {
        mediatype_set_uint32(
            media_type,
            core::ptr::addr_of!(MF_MT_FIXED_SIZE_SAMPLES),
            1 as ::core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }
    if (*am_type).lSampleSize != 0
        && (IMFMediaType_GetItem(
            media_type,
            core::ptr::addr_of!(MF_MT_SAMPLE_SIZE),
            core::ptr::null_mut::<PROPVARIANT>(),
        )) < 0
    {
        mediatype_set_uint32(
            media_type,
            core::ptr::addr_of!(MF_MT_SAMPLE_SIZE),
            (*am_type).lSampleSize as ::core::ffi::c_uint,
            core::ptr::addr_of_mut!(hr),
        );
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMediaTypeFromRepresentation(
    guid_representation: GUID,
    representation: *mut core::ffi::c_void,
    media_type: *mut *mut IMFMediaType,
) -> HRESULT {
    let mut hr: HRESULT;

    if IsEqualGUID!(&raw const guid_representation, &AM_MEDIA_TYPE_REPRESENTATION) == false {
        return MF_E_UNSUPPORTED_REPRESENTATION;
    }
    if representation.is_null() || media_type.is_null() {
        return E_INVALIDARG;
    }

    hr = MFCreateMediaType(media_type);
    if hr < 0 {
        return hr;
    }
    hr = MFInitMediaTypeFromAMMediaType(*media_type, representation as *const AM_MEDIA_TYPE);
    if hr < 0 {
        IMFMediaType_Release(*media_type);
        *media_type = core::ptr::null_mut();
    }

    return hr;
}

// COM vtable tables — generated from mediatype.c by vtbl_gen.py
/// mediatype.c:667 `static const IMFMediaTypeVtbl mediatypevtbl`  (slot order: rust:types_amd64.rs)
pub static mediatypevtbl: IMFMediaTypeVtbl = IMFMediaTypeVtbl {
    QueryInterface: Some(mediatype_QueryInterface),
    AddRef: Some(mediatype_AddRef),
    Release: Some(mediatype_Release),
    GetItem: Some(mediatype_GetItem),
    GetItemType: Some(mediatype_GetItemType),
    CompareItem: Some(mediatype_CompareItem),
    Compare: Some(mediatype_Compare),
    GetUINT32: Some(mediatype_GetUINT32),
    GetUINT64: Some(mediatype_GetUINT64),
    GetDouble: Some(mediatype_GetDouble),
    GetGUID: Some(mediatype_GetGUID),
    GetStringLength: Some(mediatype_GetStringLength),
    GetString: Some(mediatype_GetString),
    GetAllocatedString: Some(mediatype_GetAllocatedString),
    GetBlobSize: Some(mediatype_GetBlobSize),
    GetBlob: Some(mediatype_GetBlob),
    GetAllocatedBlob: Some(mediatype_GetAllocatedBlob),
    GetUnknown: Some(mediatype_GetUnknown),
    SetItem: Some(mediatype_SetItem),
    DeleteItem: Some(mediatype_DeleteItem),
    DeleteAllItems: Some(mediatype_DeleteAllItems),
    SetUINT32: Some(mediatype_SetUINT32),
    SetUINT64: Some(mediatype_SetUINT64),
    SetDouble: Some(mediatype_SetDouble),
    SetGUID: Some(mediatype_SetGUID),
    SetString: Some(mediatype_SetString),
    SetBlob: Some(mediatype_SetBlob),
    SetUnknown: Some(mediatype_SetUnknown),
    LockStore: Some(mediatype_LockStore),
    UnlockStore: Some(mediatype_UnlockStore),
    GetCount: Some(mediatype_GetCount),
    GetItemByIndex: Some(mediatype_GetItemByIndex),
    CopyAllItems: Some(mediatype_CopyAllItems),
    GetMajorType: Some(mediatype_GetMajorType),
    IsCompressedFormat: Some(mediatype_IsCompressedFormat),
    IsEqual: Some(mediatype_IsEqual),
    GetRepresentation: Some(mediatype_GetRepresentation),
    FreeRepresentation: Some(mediatype_FreeRepresentation),
};

/// mediatype.c:1068 `static const IMFVideoMediaTypeVtbl videomediatypevtbl`  (slot order: rust:types_amd64.rs)
pub static videomediatypevtbl: IMFVideoMediaTypeVtbl = IMFVideoMediaTypeVtbl {
    QueryInterface: Some(video_mediatype_QueryInterface),
    AddRef: Some(video_mediatype_AddRef),
    Release: Some(video_mediatype_Release),
    GetItem: Some(video_mediatype_GetItem),
    GetItemType: Some(video_mediatype_GetItemType),
    CompareItem: Some(video_mediatype_CompareItem),
    Compare: Some(video_mediatype_Compare),
    GetUINT32: Some(video_mediatype_GetUINT32),
    GetUINT64: Some(video_mediatype_GetUINT64),
    GetDouble: Some(video_mediatype_GetDouble),
    GetGUID: Some(video_mediatype_GetGUID),
    GetStringLength: Some(video_mediatype_GetStringLength),
    GetString: Some(video_mediatype_GetString),
    GetAllocatedString: Some(video_mediatype_GetAllocatedString),
    GetBlobSize: Some(video_mediatype_GetBlobSize),
    GetBlob: Some(video_mediatype_GetBlob),
    GetAllocatedBlob: Some(video_mediatype_GetAllocatedBlob),
    GetUnknown: Some(video_mediatype_GetUnknown),
    SetItem: Some(video_mediatype_SetItem),
    DeleteItem: Some(video_mediatype_DeleteItem),
    DeleteAllItems: Some(video_mediatype_DeleteAllItems),
    SetUINT32: Some(video_mediatype_SetUINT32),
    SetUINT64: Some(video_mediatype_SetUINT64),
    SetDouble: Some(video_mediatype_SetDouble),
    SetGUID: Some(video_mediatype_SetGUID),
    SetString: Some(video_mediatype_SetString),
    SetBlob: Some(video_mediatype_SetBlob),
    SetUnknown: Some(video_mediatype_SetUnknown),
    LockStore: Some(video_mediatype_LockStore),
    UnlockStore: Some(video_mediatype_UnlockStore),
    GetCount: Some(video_mediatype_GetCount),
    GetItemByIndex: Some(video_mediatype_GetItemByIndex),
    CopyAllItems: Some(video_mediatype_CopyAllItems),
    GetMajorType: Some(video_mediatype_GetMajorType),
    IsCompressedFormat: Some(video_mediatype_IsCompressedFormat),
    IsEqual: Some(video_mediatype_IsEqual),
    GetRepresentation: Some(video_mediatype_GetRepresentation),
    FreeRepresentation: Some(video_mediatype_FreeRepresentation),
    GetVideoFormat: Some(video_mediatype_GetVideoFormat),
    GetVideoRepresentation: Some(video_mediatype_GetVideoRepresentation),
};

/// mediatype.c:1468 `static const IMFAudioMediaTypeVtbl audiomediatypevtbl`  (slot order: rust:types_amd64.rs)
pub static audiomediatypevtbl: IMFAudioMediaTypeVtbl = IMFAudioMediaTypeVtbl {
    QueryInterface: Some(audio_mediatype_QueryInterface),
    AddRef: Some(audio_mediatype_AddRef),
    Release: Some(audio_mediatype_Release),
    GetItem: Some(audio_mediatype_GetItem),
    GetItemType: Some(audio_mediatype_GetItemType),
    CompareItem: Some(audio_mediatype_CompareItem),
    Compare: Some(audio_mediatype_Compare),
    GetUINT32: Some(audio_mediatype_GetUINT32),
    GetUINT64: Some(audio_mediatype_GetUINT64),
    GetDouble: Some(audio_mediatype_GetDouble),
    GetGUID: Some(audio_mediatype_GetGUID),
    GetStringLength: Some(audio_mediatype_GetStringLength),
    GetString: Some(audio_mediatype_GetString),
    GetAllocatedString: Some(audio_mediatype_GetAllocatedString),
    GetBlobSize: Some(audio_mediatype_GetBlobSize),
    GetBlob: Some(audio_mediatype_GetBlob),
    GetAllocatedBlob: Some(audio_mediatype_GetAllocatedBlob),
    GetUnknown: Some(audio_mediatype_GetUnknown),
    SetItem: Some(audio_mediatype_SetItem),
    DeleteItem: Some(audio_mediatype_DeleteItem),
    DeleteAllItems: Some(audio_mediatype_DeleteAllItems),
    SetUINT32: Some(audio_mediatype_SetUINT32),
    SetUINT64: Some(audio_mediatype_SetUINT64),
    SetDouble: Some(audio_mediatype_SetDouble),
    SetGUID: Some(audio_mediatype_SetGUID),
    SetString: Some(audio_mediatype_SetString),
    SetBlob: Some(audio_mediatype_SetBlob),
    SetUnknown: Some(audio_mediatype_SetUnknown),
    LockStore: Some(audio_mediatype_LockStore),
    UnlockStore: Some(audio_mediatype_UnlockStore),
    GetCount: Some(audio_mediatype_GetCount),
    GetItemByIndex: Some(audio_mediatype_GetItemByIndex),
    CopyAllItems: Some(audio_mediatype_CopyAllItems),
    GetMajorType: Some(audio_mediatype_GetMajorType),
    IsCompressedFormat: Some(audio_mediatype_IsCompressedFormat),
    IsEqual: Some(audio_mediatype_IsEqual),
    GetRepresentation: Some(audio_mediatype_GetRepresentation),
    FreeRepresentation: Some(audio_mediatype_FreeRepresentation),
    GetAudioFormat: Some(audio_mediatype_GetAudioFormat),
};

/// mediatype.c:1909 `static const IMFStreamDescriptorVtbl streamdescriptorvtbl`  (slot order: rust:types_amd64.rs)
pub static streamdescriptorvtbl: IMFStreamDescriptorVtbl = IMFStreamDescriptorVtbl {
    QueryInterface: Some(stream_descriptor_QueryInterface),
    AddRef: Some(stream_descriptor_AddRef),
    Release: Some(stream_descriptor_Release),
    GetItem: Some(stream_descriptor_GetItem),
    GetItemType: Some(stream_descriptor_GetItemType),
    CompareItem: Some(stream_descriptor_CompareItem),
    Compare: Some(stream_descriptor_Compare),
    GetUINT32: Some(stream_descriptor_GetUINT32),
    GetUINT64: Some(stream_descriptor_GetUINT64),
    GetDouble: Some(stream_descriptor_GetDouble),
    GetGUID: Some(stream_descriptor_GetGUID),
    GetStringLength: Some(stream_descriptor_GetStringLength),
    GetString: Some(stream_descriptor_GetString),
    GetAllocatedString: Some(stream_descriptor_GetAllocatedString),
    GetBlobSize: Some(stream_descriptor_GetBlobSize),
    GetBlob: Some(stream_descriptor_GetBlob),
    GetAllocatedBlob: Some(stream_descriptor_GetAllocatedBlob),
    GetUnknown: Some(stream_descriptor_GetUnknown),
    SetItem: Some(stream_descriptor_SetItem),
    DeleteItem: Some(stream_descriptor_DeleteItem),
    DeleteAllItems: Some(stream_descriptor_DeleteAllItems),
    SetUINT32: Some(stream_descriptor_SetUINT32),
    SetUINT64: Some(stream_descriptor_SetUINT64),
    SetDouble: Some(stream_descriptor_SetDouble),
    SetGUID: Some(stream_descriptor_SetGUID),
    SetString: Some(stream_descriptor_SetString),
    SetBlob: Some(stream_descriptor_SetBlob),
    SetUnknown: Some(stream_descriptor_SetUnknown),
    LockStore: Some(stream_descriptor_LockStore),
    UnlockStore: Some(stream_descriptor_UnlockStore),
    GetCount: Some(stream_descriptor_GetCount),
    GetItemByIndex: Some(stream_descriptor_GetItemByIndex),
    CopyAllItems: Some(stream_descriptor_CopyAllItems),
    GetStreamIdentifier: Some(stream_descriptor_GetStreamIdentifier),
    GetMediaTypeHandler: Some(stream_descriptor_GetMediaTypeHandler),
};

/// mediatype.c:2104 `static const IMFMediaTypeHandlerVtbl mediatypehandlervtbl`  (slot order: rust:types_amd64.rs)
pub static mediatypehandlervtbl: IMFMediaTypeHandlerVtbl = IMFMediaTypeHandlerVtbl {
    QueryInterface: Some(mediatype_handler_QueryInterface),
    AddRef: Some(mediatype_handler_AddRef),
    Release: Some(mediatype_handler_Release),
    IsMediaTypeSupported: Some(mediatype_handler_IsMediaTypeSupported),
    GetMediaTypeCount: Some(mediatype_handler_GetMediaTypeCount),
    GetMediaTypeByIndex: Some(mediatype_handler_GetMediaTypeByIndex),
    SetCurrentMediaType: Some(mediatype_handler_SetCurrentMediaType),
    GetCurrentMediaType: Some(mediatype_handler_GetCurrentMediaType),
    GetMajorType: Some(mediatype_handler_GetMajorType),
};

/// mediatype.c:2597 `static const IMFPresentationDescriptorVtbl presentationdescriptorvtbl`  (slot order: rust:types_amd64.rs)
pub static presentationdescriptorvtbl: IMFPresentationDescriptorVtbl = IMFPresentationDescriptorVtbl {
    QueryInterface: Some(presentation_descriptor_QueryInterface),
    AddRef: Some(presentation_descriptor_AddRef),
    Release: Some(presentation_descriptor_Release),
    GetItem: Some(presentation_descriptor_GetItem),
    GetItemType: Some(presentation_descriptor_GetItemType),
    CompareItem: Some(presentation_descriptor_CompareItem),
    Compare: Some(presentation_descriptor_Compare),
    GetUINT32: Some(presentation_descriptor_GetUINT32),
    GetUINT64: Some(presentation_descriptor_GetUINT64),
    GetDouble: Some(presentation_descriptor_GetDouble),
    GetGUID: Some(presentation_descriptor_GetGUID),
    GetStringLength: Some(presentation_descriptor_GetStringLength),
    GetString: Some(presentation_descriptor_GetString),
    GetAllocatedString: Some(presentation_descriptor_GetAllocatedString),
    GetBlobSize: Some(presentation_descriptor_GetBlobSize),
    GetBlob: Some(presentation_descriptor_GetBlob),
    GetAllocatedBlob: Some(presentation_descriptor_GetAllocatedBlob),
    GetUnknown: Some(presentation_descriptor_GetUnknown),
    SetItem: Some(presentation_descriptor_SetItem),
    DeleteItem: Some(presentation_descriptor_DeleteItem),
    DeleteAllItems: Some(presentation_descriptor_DeleteAllItems),
    SetUINT32: Some(presentation_descriptor_SetUINT32),
    SetUINT64: Some(presentation_descriptor_SetUINT64),
    SetDouble: Some(presentation_descriptor_SetDouble),
    SetGUID: Some(presentation_descriptor_SetGUID),
    SetString: Some(presentation_descriptor_SetString),
    SetBlob: Some(presentation_descriptor_SetBlob),
    SetUnknown: Some(presentation_descriptor_SetUnknown),
    LockStore: Some(presentation_descriptor_LockStore),
    UnlockStore: Some(presentation_descriptor_UnlockStore),
    GetCount: Some(presentation_descriptor_GetCount),
    GetItemByIndex: Some(presentation_descriptor_GetItemByIndex),
    CopyAllItems: Some(presentation_descriptor_CopyAllItems),
    GetStreamDescriptorCount: Some(presentation_descriptor_GetStreamDescriptorCount),
    GetStreamDescriptorByIndex: Some(presentation_descriptor_GetStreamDescriptorByIndex),
    SelectStream: Some(presentation_descriptor_SelectStream),
    DeselectStream: Some(presentation_descriptor_DeselectStream),
    Clone: Some(presentation_descriptor_Clone),
};
