use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(ddraw): Wine debug-channel registration macro, no runtime-visible item here.
// Channel name for reference: "ddraw".
pub const __WINE_DEFAULT_DEBUG_CHANNEL: &str = "ddraw";
// static void DDRAW_dump_pixelformat(const DDPIXELFORMAT *pf): forward prototype only, no declaration emitted (the function is translated separately).

#[no_mangle]
pub unsafe extern "C" fn ddrawformat_from_wined3dformat(
    DDPixelFormat: *mut DDPIXELFORMAT,
    wined3d_format: wined3d_format_id,
) {
    let mut Size: DWORD = (*DDPixelFormat).dwSize;

    if Size == 0 {
        return;
    }

    memset(DDPixelFormat as *mut core::ffi::c_void, 0x00, Size as usize);
    (*DDPixelFormat).dwSize = Size;
    match wined3d_format as u32 {
        WINED3DFMT_B8G8R8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 24;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x00ff0000;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x0000ff00;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x000000ff;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_B8G8R8A8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB | DDPF_ALPHAPIXELS;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x00ff0000;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x0000ff00;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x000000ff;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0xff000000;
        }

        WINED3DFMT_B8G8R8X8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x00ff0000;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x0000ff00;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x000000ff;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_R8G8B8X8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x000000ff;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x0000ff00;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x00ff0000;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_B5G6R5_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0xF800;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x07E0;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x001F;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_B5G5R5X1_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x7C00;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x03E0;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x001F;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_B5G5R5A1_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB | DDPF_ALPHAPIXELS;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x7C00;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x03E0;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x001F;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x8000;
        }

        WINED3DFMT_B4G4R4A4_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB | DDPF_ALPHAPIXELS;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x0F00;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x00F0;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x000F;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0xF000;
        }

        WINED3DFMT_B2G3R3_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 8;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0xE0;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x1C;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x03;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0x0;
        }

        WINED3DFMT_P8_UINT => {
            (*DDPixelFormat).dwFlags = DDPF_PALETTEINDEXED8 | DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 8;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x00;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x00;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x00;
        }

        WINED3DFMT_A8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ALPHA;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwAlphaBitDepth = 8;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_3.dwZBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_4.dwStencilBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0x0;
        }

        WINED3DFMT_B2G3R3A8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB | DDPF_ALPHAPIXELS;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x00E0;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x001C;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x0003;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0xFF00;
        }

        WINED3DFMT_B4G4R4X4_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_RGB;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwRBitMask = 0x0F00;
            (*DDPixelFormat).__bindgen_anon_3.dwGBitMask = 0x00F0;
            (*DDPixelFormat).__bindgen_anon_4.dwBBitMask = 0x000F;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        /* How are Z buffer bit depth and Stencil buffer bit depth related?
         */
        WINED3DFMT_D16_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ZBUFFER;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth = 0;
            (*DDPixelFormat).__bindgen_anon_3.dwZBitMask = 0x0000FFFF;
            (*DDPixelFormat).__bindgen_anon_4.dwStencilBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBZBitMask = 0x00000000;
        }

        WINED3DFMT_D32_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ZBUFFER;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth = 0;
            (*DDPixelFormat).__bindgen_anon_3.dwZBitMask = 0xFFFFFFFF;
            (*DDPixelFormat).__bindgen_anon_4.dwStencilBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBZBitMask = 0x00000000;
        }

        WINED3DFMT_S4X4_UINT_D24_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ZBUFFER | DDPF_STENCILBUFFER;
            (*DDPixelFormat).dwFourCC = 0;
            /* Should I set dwZBufferBitDepth to 32 here? */
            (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth = 4;
            (*DDPixelFormat).__bindgen_anon_3.dwZBitMask = 0x00FFFFFF;
            (*DDPixelFormat).__bindgen_anon_4.dwStencilBitMask = 0x0F000000;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_D24_UNORM_S8_UINT => {
            (*DDPixelFormat).dwFlags = DDPF_ZBUFFER | DDPF_STENCILBUFFER;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth = 8;
            (*DDPixelFormat).__bindgen_anon_3.dwZBitMask = 0x00FFFFFF;
            (*DDPixelFormat).__bindgen_anon_4.dwStencilBitMask = 0xFF000000;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_X8D24_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ZBUFFER;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth = 0;
            (*DDPixelFormat).__bindgen_anon_3.dwZBitMask = 0x00FFFFFF;
            (*DDPixelFormat).__bindgen_anon_4.dwStencilBitMask = 0x00000000;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_S1_UINT_D15_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ZBUFFER | DDPF_STENCILBUFFER;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth = 1;
            (*DDPixelFormat).__bindgen_anon_3.dwZBitMask = 0x7fff;
            (*DDPixelFormat).__bindgen_anon_4.dwStencilBitMask = 0x8000;
            (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask = 0x0;
        }

        WINED3DFMT_UYVY | WINED3DFMT_YUY2 => {
            (*DDPixelFormat).__bindgen_anon_1.dwYUVBitCount = 16;
            (*DDPixelFormat).dwFlags = DDPF_FOURCC;
            (*DDPixelFormat).dwFourCC = wined3d_format as DWORD;
        }

        WINED3DFMT_YV12 => {
            (*DDPixelFormat).__bindgen_anon_1.dwYUVBitCount = 12;
            (*DDPixelFormat).dwFlags = DDPF_FOURCC;
            (*DDPixelFormat).dwFourCC = wined3d_format as DWORD;
        }

        WINED3DFMT_DXT1
        | WINED3DFMT_DXT2
        | WINED3DFMT_DXT3
        | WINED3DFMT_DXT4
        | WINED3DFMT_DXT5
        | WINED3DFMT_MULTI2_ARGB8
        | WINED3DFMT_G8R8_G8B8
        | WINED3DFMT_R8G8_B8G8 => {
            (*DDPixelFormat).dwFlags = DDPF_FOURCC;
            (*DDPixelFormat).dwFourCC = wined3d_format as DWORD;
        }

        /* Luminance */
        WINED3DFMT_L8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_LUMINANCE;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwLuminanceBitCount = 8;
            (*DDPixelFormat).__bindgen_anon_2.dwLuminanceBitMask = 0xff;
            (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0x0;
        }

        WINED3DFMT_L4A4_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ALPHAPIXELS | DDPF_LUMINANCE;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwLuminanceBitCount = 4;
            (*DDPixelFormat).__bindgen_anon_2.dwLuminanceBitMask = 0x0f;
            (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0xf0;
        }

        WINED3DFMT_L8A8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_ALPHAPIXELS | DDPF_LUMINANCE;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwLuminanceBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwLuminanceBitMask = 0x00ff;
            (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask = 0x0;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0xff00;
        }

        /* Bump mapping */
        WINED3DFMT_R8G8_SNORM => {
            (*DDPixelFormat).dwFlags = DDPF_BUMPDUDV;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwBumpBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwBumpDuBitMask = 0x000000ff;
            (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask = 0x0000ff00;
            (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask = 0x00000000;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0x00000000;
        }

        WINED3DFMT_R5G5_SNORM_L6_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_BUMPDUDV | DDPF_BUMPLUMINANCE;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwBumpBitCount = 16;
            (*DDPixelFormat).__bindgen_anon_2.dwBumpDuBitMask = 0x0000001f;
            (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask = 0x000003e0;
            (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask = 0x0000fc00;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0x00000000;
        }

        WINED3DFMT_R8G8_SNORM_L8X8_UNORM => {
            (*DDPixelFormat).dwFlags = DDPF_BUMPDUDV | DDPF_BUMPLUMINANCE;
            (*DDPixelFormat).dwFourCC = 0;
            (*DDPixelFormat).__bindgen_anon_1.dwBumpBitCount = 32;
            (*DDPixelFormat).__bindgen_anon_2.dwBumpDuBitMask = 0x000000ff;
            (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask = 0x0000ff00;
            (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask = 0x00ff0000;
            (*DDPixelFormat).__bindgen_anon_5.dwLuminanceAlphaBitMask = 0x00000000;
        }

        _ => {
            FIXME!("Unhandled wined3d format %#x.\n", wined3d_format);
        }
    }

    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        DDRAW_dump_pixelformat(DDPixelFormat);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3dformat_from_ddrawformat(
    DDPixelFormat: *const DDPIXELFORMAT,
) -> wined3d_format_id {
    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        DDRAW_dump_pixelformat(DDPixelFormat);
    }

    if (*DDPixelFormat).dwFlags & DDPF_PALETTEINDEXED8 != 0 {
        return WINED3DFMT_P8_UINT;
    } else if (*DDPixelFormat).dwFlags
        & (DDPF_PALETTEINDEXED1 | DDPF_PALETTEINDEXED2 | DDPF_PALETTEINDEXED4)
        != 0
    {
        return WINED3DFMT_P8_UINT;
    } else if (*DDPixelFormat).dwFlags & DDPF_RGB != 0 {
        match (*DDPixelFormat).__bindgen_anon_1.dwRGBBitCount {
            8 => return WINED3DFMT_B2G3R3_UNORM,
            16 => {
                if (*DDPixelFormat).__bindgen_anon_2.dwRBitMask == 0xF800
                    && (*DDPixelFormat).__bindgen_anon_3.dwGBitMask == 0x07E0
                    && (*DDPixelFormat).__bindgen_anon_4.dwBBitMask == 0x001F
                {
                    return WINED3DFMT_B5G6R5_UNORM;
                }
                if (*DDPixelFormat).__bindgen_anon_2.dwRBitMask == 0x7C00
                    && (*DDPixelFormat).__bindgen_anon_3.dwGBitMask == 0x03E0
                    && (*DDPixelFormat).__bindgen_anon_4.dwBBitMask == 0x001F
                {
                    if (*DDPixelFormat).dwFlags & DDPF_ALPHAPIXELS != 0
                        && (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask
                            == 0x8000
                    {
                        return WINED3DFMT_B5G5R5A1_UNORM;
                    } else {
                        return WINED3DFMT_B5G5R5X1_UNORM;
                    }
                }
                if (*DDPixelFormat).__bindgen_anon_2.dwRBitMask == 0x0F00
                    && (*DDPixelFormat).__bindgen_anon_3.dwGBitMask == 0x00F0
                    && (*DDPixelFormat).__bindgen_anon_4.dwBBitMask == 0x000F
                {
                    if (*DDPixelFormat).dwFlags & DDPF_ALPHAPIXELS != 0
                        && (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask
                            == 0xF000
                    {
                        return WINED3DFMT_B4G4R4A4_UNORM;
                    } else {
                        return WINED3DFMT_B4G4R4X4_UNORM;
                    }
                }
                if (*DDPixelFormat).dwFlags & DDPF_ALPHAPIXELS != 0
                    && (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask
                        == 0xFF00
                    && (*DDPixelFormat).__bindgen_anon_2.dwRBitMask == 0x00E0
                    && (*DDPixelFormat).__bindgen_anon_3.dwGBitMask == 0x001C
                    && (*DDPixelFormat).__bindgen_anon_4.dwBBitMask == 0x0003
                {
                    return WINED3DFMT_B2G3R3A8_UNORM;
                }
                WARN!("16 bit RGB Pixel format does not match.\n");
                return WINED3DFMT_UNKNOWN;
            }
            24 => return WINED3DFMT_B8G8R8_UNORM,
            32 => {
                if (*DDPixelFormat).__bindgen_anon_2.dwRBitMask == 0x00FF0000
                    && (*DDPixelFormat).__bindgen_anon_3.dwGBitMask == 0x0000FF00
                    && (*DDPixelFormat).__bindgen_anon_4.dwBBitMask == 0x000000FF
                {
                    if (*DDPixelFormat).dwFlags & DDPF_ALPHAPIXELS != 0
                        && (*DDPixelFormat).__bindgen_anon_5.dwRGBAlphaBitMask
                            == 0xFF000000
                    {
                        return WINED3DFMT_B8G8R8A8_UNORM;
                    } else {
                        return WINED3DFMT_B8G8R8X8_UNORM;
                    }
                }
                WARN!("32 bit RGB pixel format does not match.\n");
                return WINED3DFMT_UNKNOWN;
            }
            _ => {
                WARN!("Invalid dwRGBBitCount in Pixelformat structure.\n");
                return WINED3DFMT_UNKNOWN;
            }
        }
    } else if (*DDPixelFormat).dwFlags & DDPF_ALPHA != 0 {
        match (*DDPixelFormat).__bindgen_anon_1.dwAlphaBitDepth {
            8 => return WINED3DFMT_A8_UNORM,
            _ => {
                WARN!("Invalid AlphaBitDepth in Alpha-Only Pixelformat.\n");
                return WINED3DFMT_UNKNOWN;
            }
        }
    } else if (*DDPixelFormat).dwFlags & DDPF_LUMINANCE != 0 {
        if (*DDPixelFormat).dwFlags & DDPF_ALPHAPIXELS != 0 {
            match (*DDPixelFormat).__bindgen_anon_1.dwLuminanceBitCount {
                4 => {
                    if (*DDPixelFormat).__bindgen_anon_1.dwAlphaBitDepth
                        == 4
                    {
                        return WINED3DFMT_L4A4_UNORM;
                    }
                    WARN!("Unknown Alpha / Luminance bit depth combination.\n");
                    return WINED3DFMT_UNKNOWN;
                }
                6 => {
                    FIXME!("A luminance Pixelformat shouldn't have 6 luminance bits. Returning D3DFMT_L6V5U5 for now.\n");
                    return WINED3DFMT_R5G5_SNORM_L6_UNORM;
                }
                8 => {
                    if (*DDPixelFormat).__bindgen_anon_1.dwAlphaBitDepth
                        == 8
                    {
                        return WINED3DFMT_L8A8_UNORM;
                    }
                    WARN!("Unknown Alpha / Lumincase bit depth combination.\n");
                    return WINED3DFMT_UNKNOWN;
                }
                _ => {}
            }
        } else {
            match (*DDPixelFormat).__bindgen_anon_1.dwLuminanceBitCount {
                6 => {
                    FIXME!("A luminance Pixelformat shouldn't have 6 luminance bits. Returning D3DFMT_L6V5U5 for now.\n");
                    return WINED3DFMT_R5G5_SNORM_L6_UNORM;
                }
                8 => return WINED3DFMT_L8_UNORM,
                _ => {
                    WARN!(
                        "Unknown luminance-only bit depth %lu.\n",
                        (*DDPixelFormat).__bindgen_anon_1.dwLuminanceBitCount
                    );
                    return WINED3DFMT_UNKNOWN;
                }
            }
        }
    } else if (*DDPixelFormat).dwFlags & DDPF_ZBUFFER != 0 {
        if (*DDPixelFormat).dwFlags & DDPF_STENCILBUFFER != 0 {
            match (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth {
                16 => {
                    if (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth
                        == 1
                    {
                        return WINED3DFMT_S1_UINT_D15_UNORM;
                    }
                    WARN!(
                        "Unknown depth stencil format: 16 z bits, %lu stencil bits.\n",
                        (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth
                    );
                    return WINED3DFMT_UNKNOWN;
                }
                32 => {
                    if (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth
                        == 8
                    {
                        return WINED3DFMT_D24_UNORM_S8_UINT;
                    } else if (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth
                        == 4
                    {
                        return WINED3DFMT_S4X4_UINT_D24_UNORM;
                    }
                    WARN!(
                        "Unknown depth stencil format: 32 z bits, %lu stencil bits.\n",
                        (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth
                    );
                    return WINED3DFMT_UNKNOWN;
                }
                _ => {
                    WARN!(
                        "Unknown depth stencil format: %lu z bits, %lu stencil bits.\n",
                        (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth,
                        (*DDPixelFormat).__bindgen_anon_2.dwStencilBitDepth
                    );
                    return WINED3DFMT_UNKNOWN;
                }
            }
        } else {
            match (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth {
                16 => return WINED3DFMT_D16_UNORM,
                24 => return WINED3DFMT_X8D24_UNORM,
                32 => {
                    if (*DDPixelFormat).__bindgen_anon_3.dwZBitMask == 0x00FFFFFF {
                        return WINED3DFMT_X8D24_UNORM;
                    } else if (*DDPixelFormat).__bindgen_anon_3.dwZBitMask
                        == 0xFFFFFF00
                    {
                        return WINED3DFMT_X8D24_UNORM;
                    } else if (*DDPixelFormat).__bindgen_anon_3.dwZBitMask
                        == 0xFFFFFFFF
                    {
                        return WINED3DFMT_D32_UNORM;
                    }
                    WARN!(
                        "Unknown depth-only format: 32 z bits, mask 0x%08lx\n",
                        (*DDPixelFormat).__bindgen_anon_3.dwZBitMask
                    );
                    return WINED3DFMT_UNKNOWN;
                }
                _ => {
                    WARN!(
                        "Unknown depth-only format: %lu z bits, mask 0x%08lx\n",
                        (*DDPixelFormat).__bindgen_anon_1.dwZBufferBitDepth,
                        (*DDPixelFormat).__bindgen_anon_3.dwZBitMask
                    );
                    return WINED3DFMT_UNKNOWN;
                }
            }
        }
    } else if (*DDPixelFormat).dwFlags & DDPF_FOURCC != 0 {
        return (*DDPixelFormat).dwFourCC as wined3d_format_id;
    } else if (*DDPixelFormat).dwFlags & DDPF_BUMPDUDV != 0 {
        if (*DDPixelFormat).__bindgen_anon_1.dwBumpBitCount == 16
            && (*DDPixelFormat).__bindgen_anon_2.dwBumpDuBitMask == 0x000000ff
            && (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask == 0x0000ff00
            && (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask
                == 0x00000000
        {
            return WINED3DFMT_R8G8_SNORM;
        } else if (*DDPixelFormat).__bindgen_anon_1.dwBumpBitCount == 16
            && (*DDPixelFormat).__bindgen_anon_2.dwBumpDuBitMask == 0x0000001f
            && (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask == 0x000003e0
            && (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask
                == 0x0000fc00
        {
            return WINED3DFMT_R5G5_SNORM_L6_UNORM;
        } else if (*DDPixelFormat).__bindgen_anon_1.dwBumpBitCount == 32
            && (*DDPixelFormat).__bindgen_anon_2.dwBumpDuBitMask == 0x000000ff
            && (*DDPixelFormat).__bindgen_anon_3.dwBumpDvBitMask == 0x0000ff00
            && (*DDPixelFormat).__bindgen_anon_4.dwBumpLuminanceBitMask
                == 0x00ff0000
        {
            return WINED3DFMT_R8G8_SNORM_L8X8_UNORM;
        }
    }

    WARN!("Unknown Pixelformat.\n");
    return WINED3DFMT_UNKNOWN;
}

#[no_mangle]
pub unsafe extern "C" fn wined3dmapflags_from_ddrawmapflags(mut flags: ::core::ffi::c_uint) -> ::core::ffi::c_uint {
    static handled: ::core::ffi::c_uint = (DDLOCK_NOSYSLOCK as ::core::ffi::c_uint
        | DDLOCK_NOOVERWRITE as ::core::ffi::c_uint
        | DDLOCK_DISCARDCONTENTS as ::core::ffi::c_uint
        | DDLOCK_DONOTWAIT as ::core::ffi::c_uint);
    let mut wined3d_flags: ::core::ffi::c_uint;

    wined3d_flags = flags & handled;
    if (flags
        & (DDLOCK_NOOVERWRITE as ::core::ffi::c_uint
            | DDLOCK_DISCARDCONTENTS as ::core::ffi::c_uint
            | DDLOCK_WRITEONLY as ::core::ffi::c_uint))
        == 0
    {
        wined3d_flags |= WINED3D_MAP_READ as ::core::ffi::c_uint;
    }
    if (flags & DDLOCK_READONLY as ::core::ffi::c_uint) == 0 {
        wined3d_flags |= WINED3D_MAP_WRITE as ::core::ffi::c_uint;
    }
    if (wined3d_flags
        & (WINED3D_MAP_READ as ::core::ffi::c_uint
            | WINED3D_MAP_WRITE as ::core::ffi::c_uint))
        == 0
    {
        wined3d_flags |= (WINED3D_MAP_READ as ::core::ffi::c_uint
            | WINED3D_MAP_WRITE as ::core::ffi::c_uint);
    }
    if (flags & DDLOCK_NODIRTYUPDATE as ::core::ffi::c_uint) != 0 {
        wined3d_flags |= WINED3D_MAP_NO_DIRTY_UPDATE as ::core::ffi::c_uint;
    }
    flags &= !(handled
        | DDLOCK_WAIT as ::core::ffi::c_uint
        | DDLOCK_READONLY as ::core::ffi::c_uint
        | DDLOCK_WRITEONLY as ::core::ffi::c_uint
        | DDLOCK_NODIRTYUPDATE as ::core::ffi::c_uint);

    if flags != 0 {
        FIXME!("Unhandled flags %#x.\n", flags);
    }

    return wined3d_flags;
}

unsafe extern "C" fn colour_to_float(colour: DWORD, mask: DWORD) -> f32 {
    if mask == 0 {
        return 0.0f32;
    }
    return (colour & mask) as f32 / mask as f32;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_colour_from_ddraw_colour(
    pf: *const DDPIXELFORMAT,
    palette: *const ddraw_palette,
    mut colour: DWORD,
    wined3d_colour: *mut wined3d_color,
) -> BOOL {
    if ((*pf).dwFlags & DDPF_ALPHA) != 0 {
        let mut size: DWORD;
        let mut mask: DWORD;

        size = (*pf).__bindgen_anon_1.dwAlphaBitDepth;
        mask = if size < 32 {
            (1 << size) - 1
        } else {
            !0
        };
        (*wined3d_colour).r = 0.0f32;
        (*wined3d_colour).g = 0.0f32;
        (*wined3d_colour).b = 0.0f32;
        (*wined3d_colour).a = colour_to_float(colour, mask);
        return (TRUE) as BOOL;
    }

    'fail: {
        if ((*pf).dwFlags & DDPF_FOURCC) != 0 {
            WARN!("FourCC formats not supported.\n");
            break 'fail;
        }

        if ((*pf).dwFlags & DDPF_PALETTEINDEXED8) != 0 {
            let mut entry: PALETTEENTRY = core::mem::zeroed();

            colour &= 0xff;
            if palette.is_null()
                || (wined3d_palette_get_entries(
                    (*palette).wined3d_palette,
                    0,
                    colour,
                    1,
                    core::ptr::addr_of_mut!(entry),
                ) < 0)
            {
                (*wined3d_colour).r = 0.0f32;
                (*wined3d_colour).g = 0.0f32;
                (*wined3d_colour).b = 0.0f32;
            } else {
                (*wined3d_colour).r = entry.peRed as f32 / 255.0f32;
                (*wined3d_colour).g = entry.peGreen as f32 / 255.0f32;
                (*wined3d_colour).b = entry.peBlue as f32 / 255.0f32;
            }
            (*wined3d_colour).a = colour as f32 / 255.0f32;
            return (TRUE) as BOOL;
        }

        if ((*pf).dwFlags & DDPF_RGB) != 0 {
            (*wined3d_colour).r = colour_to_float(colour, (*pf).__bindgen_anon_2.dwRBitMask);
            (*wined3d_colour).g = colour_to_float(colour, (*pf).__bindgen_anon_3.dwGBitMask);
            (*wined3d_colour).b = colour_to_float(colour, (*pf).__bindgen_anon_4.dwBBitMask);
            if ((*pf).dwFlags & DDPF_ALPHAPIXELS) != 0 {
                (*wined3d_colour).a =
                    colour_to_float(colour, (*pf).__bindgen_anon_5.dwRGBAlphaBitMask);
            } else {
                (*wined3d_colour).a = 0.0f32;
            }
            return (TRUE) as BOOL;
        }

        if ((*pf).dwFlags & DDPF_ZBUFFER) != 0 {
            (*wined3d_colour).r = colour_to_float(colour, (*pf).__bindgen_anon_3.dwZBitMask);
            if ((*pf).dwFlags & DDPF_STENCILBUFFER) != 0 {
                (*wined3d_colour).g =
                    colour_to_float(colour, (*pf).__bindgen_anon_4.dwStencilBitMask);
            } else {
                (*wined3d_colour).g = 0.0f32;
            }
            (*wined3d_colour).b = 0.0f32;
            (*wined3d_colour).a = 0.0f32;
            return (TRUE) as BOOL;
        }

        DDRAW_dump_pixelformat(pf);
    }

    (*wined3d_colour).r = 0.0f32;
    (*wined3d_colour).g = 0.0f32;
    (*wined3d_colour).b = 0.0f32;
    (*wined3d_colour).a = 0.0f32;

    (FALSE as i32)
}

unsafe extern "C" fn DDRAW_dump_DWORD(r#in: *const ::core::ffi::c_void) {
    // TRACE("%ld\n", *((const DWORD *) in));
}

unsafe extern "C" fn DDRAW_dump_PTR(r#in: *const ::core::ffi::c_void) {
    // TRACE("%p\n", *((const void * const*) in));
}

unsafe extern "C" fn DDRAW_dump_DDCOLORKEY(ddck: *const DDCOLORKEY) {
    // TRACE("Low : 0x%08lx - High : 0x%08lx\n", ddck->dwColorSpaceLowValue, ddck->dwColorSpaceHighValue);
}

unsafe extern "C" fn DDRAW_dump_flags_nolf(flags: DWORD, names: *const flag_info, num_names: size_t) {
    let mut i: ::core::ffi::c_uint = 0;
    while (i as size_t) < num_names {
        if (flags & (*names.offset(i as isize)).val) != 0
            || ((flags == 0) && ((*names.offset(i as isize)).val == 0))
        {
            TRACE!("%s ", (*names.offset(i as isize)).name);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn DDRAW_dump_flags(
    flags: DWORD,
    names: *const flag_info,
    num_names: size_t,
) {
    // TRACE("\n");
    DDRAW_dump_flags_nolf(flags, names, num_names);
}
#[no_mangle]
pub unsafe extern "C" fn DDRAW_dump_DDSCAPS2(in_: *const DDSCAPS2) {
    static mut flags: [flag_info; 30] = [
        FE!(DDSCAPS_RESERVED1),
        FE!(DDSCAPS_ALPHA),
        FE!(DDSCAPS_BACKBUFFER),
        FE!(DDSCAPS_COMPLEX),
        FE!(DDSCAPS_FLIP),
        FE!(DDSCAPS_FRONTBUFFER),
        FE!(DDSCAPS_OFFSCREENPLAIN),
        FE!(DDSCAPS_OVERLAY),
        FE!(DDSCAPS_PALETTE),
        FE!(DDSCAPS_PRIMARYSURFACE),
        FE!(DDSCAPS_PRIMARYSURFACELEFT),
        FE!(DDSCAPS_SYSTEMMEMORY),
        FE!(DDSCAPS_TEXTURE),
        FE!(DDSCAPS_3DDEVICE),
        FE!(DDSCAPS_VIDEOMEMORY),
        FE!(DDSCAPS_VISIBLE),
        FE!(DDSCAPS_WRITEONLY),
        FE!(DDSCAPS_ZBUFFER),
        FE!(DDSCAPS_OWNDC),
        FE!(DDSCAPS_LIVEVIDEO),
        FE!(DDSCAPS_HWCODEC),
        FE!(DDSCAPS_MODEX),
        FE!(DDSCAPS_MIPMAP),
        FE!(DDSCAPS_RESERVED2),
        FE!(DDSCAPS_ALLOCONLOAD),
        FE!(DDSCAPS_VIDEOPORT),
        FE!(DDSCAPS_LOCALVIDMEM),
        FE!(DDSCAPS_NONLOCALVIDMEM),
        FE!(DDSCAPS_STANDARDVGAMODE),
        FE!(DDSCAPS_OPTIMIZED)
    ];
    static mut flags2: [flag_info; 19] = [
        FE!(DDSCAPS2_HARDWAREDEINTERLACE),
        FE!(DDSCAPS2_HINTDYNAMIC),
        FE!(DDSCAPS2_HINTSTATIC),
        FE!(DDSCAPS2_TEXTUREMANAGE),
        FE!(DDSCAPS2_RESERVED1),
        FE!(DDSCAPS2_RESERVED2),
        FE!(DDSCAPS2_OPAQUE),
        FE!(DDSCAPS2_HINTANTIALIASING),
        FE!(DDSCAPS2_CUBEMAP),
        FE!(DDSCAPS2_CUBEMAP_POSITIVEX),
        FE!(DDSCAPS2_CUBEMAP_NEGATIVEX),
        FE!(DDSCAPS2_CUBEMAP_POSITIVEY),
        FE!(DDSCAPS2_CUBEMAP_NEGATIVEY),
        FE!(DDSCAPS2_CUBEMAP_POSITIVEZ),
        FE!(DDSCAPS2_CUBEMAP_NEGATIVEZ),
        FE!(DDSCAPS2_MIPMAPSUBLEVEL),
        FE!(DDSCAPS2_D3DTEXTUREMANAGE),
        FE!(DDSCAPS2_DONOTPERSIST),
        FE!(DDSCAPS2_STEREOSURFACELEFT)
    ];

    DDRAW_dump_flags_nolf(
        (*in_).dwCaps,
        core::ptr::addr_of!(flags) as *const flag_info,
        (ARRAY_SIZE!(flags)) as size_t
    );
    DDRAW_dump_flags(
        (*in_).dwCaps2,
        core::ptr::addr_of!(flags2) as *const flag_info,
        (ARRAY_SIZE!(flags2)) as size_t
    );
}

unsafe extern "C" fn DDRAW_dump_DDSCAPS(in_: *const DDSCAPS) {
    let mut in_bis: DDSCAPS2 = core::mem::zeroed();

    in_bis.dwCaps = (*in_).dwCaps;
    in_bis.dwCaps2 = 0;
    in_bis.dwCaps3 = 0;
    in_bis.__bindgen_anon_1.dwCaps4 = 0;

    DDRAW_dump_DDSCAPS2(core::ptr::addr_of!(in_bis));
}

unsafe extern "C" fn DDRAW_dump_pixelformat_flag(flagmask: DWORD) {
    static mut flags: [flag_info; 14] = [
        FE!(DDPF_ALPHAPIXELS),
        FE!(DDPF_ALPHA),
        FE!(DDPF_FOURCC),
        FE!(DDPF_PALETTEINDEXED4),
        FE!(DDPF_PALETTEINDEXEDTO8),
        FE!(DDPF_PALETTEINDEXED8),
        FE!(DDPF_RGB),
        FE!(DDPF_COMPRESSED),
        FE!(DDPF_RGBTOYUV),
        FE!(DDPF_YUV),
        FE!(DDPF_ZBUFFER),
        FE!(DDPF_PALETTEINDEXED1),
        FE!(DDPF_PALETTEINDEXED2),
        FE!(DDPF_ZPIXELS)
    ];

    DDRAW_dump_flags_nolf(flagmask, core::ptr::addr_of!(flags) as *const flag_info, (ARRAY_SIZE!(flags)) as size_t);
}

unsafe extern "C" fn DDRAW_dump_members(
    mut flags: DWORD,
    mut data: *const ::core::ffi::c_void,
    mut mems: *const member_info,
    mut num_mems: size_t,
) {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while (i as size_t) < num_mems {
        if ((*mems.offset(i as isize)).val & flags) != 0 {
            WINE_TRACE!(" - %s : ", (*mems.offset(i as isize)).name);
            (*mems.offset(i as isize))
                .func
                .unwrap_unchecked()(
                (data as *const ::core::ffi::c_char)
                    .offset((*mems.offset(i as isize)).offset as isize)
                    as *const ::core::ffi::c_void,
            );
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn DDRAW_dump_pixelformat(pf: *const DDPIXELFORMAT) {
    TRACE!("( ");
    DDRAW_dump_pixelformat_flag((*pf).dwFlags);
    if (*pf).dwFlags & (DDPF_FOURCC as DWORD) != 0 {
        TRACE!(
            ", dwFourCC code %s - %lu bits per pixel",
            debugstr_fourcc((*pf).dwFourCC),
            (*pf).__bindgen_anon_1.dwYUVBitCount
        );
    }
    if (*pf).dwFlags & (DDPF_RGB as DWORD) != 0 {
        if (*pf).dwFlags & (DDPF_ALPHAPIXELS as DWORD) != 0 {
            TRACE!(" A 0x%08lx", (*pf).__bindgen_anon_5.dwRGBAlphaBitMask);
        }
        if (*pf).dwFlags & (DDPF_ZPIXELS as DWORD) != 0 {
            TRACE!(" Z 0x%08lx", (*pf).__bindgen_anon_5.dwRGBZBitMask);
        }
    }
    if (*pf).dwFlags & (DDPF_ZBUFFER as DWORD) != 0 {
        TRACE!(", Z bits: %lu", (*pf).__bindgen_anon_1.dwZBufferBitDepth);
    }
    if (*pf).dwFlags & (DDPF_ALPHA as DWORD) != 0 {
        TRACE!(", Alpha bits: %lu", (*pf).__bindgen_anon_1.dwAlphaBitDepth);
    }
    if (*pf).dwFlags & (DDPF_BUMPDUDV as DWORD) != 0 {
        TRACE!(
            ", Bump bits: %lu, U 0x%08lx V 0x%08lx L 0x%08lx",
            (*pf).__bindgen_anon_1.dwBumpBitCount,
            (*pf).__bindgen_anon_2.dwBumpDuBitMask,
            (*pf).__bindgen_anon_3.dwBumpDvBitMask,
            (*pf).__bindgen_anon_4.dwBumpLuminanceBitMask
        );
    }
    TRACE!(")\n");
}
#[no_mangle]
pub unsafe extern "C" fn DDRAW_dump_surface_desc(lpddsd: *const DDSURFACEDESC2) {
    // C: `#define STRUCT DDSURFACEDESC2` around these tables.
    type STRUCT = DDSURFACEDESC2;
    static mut members: [member_info; 14] = [
        ME!(DDSD_HEIGHT, DDRAW_dump_DWORD, dwHeight),
        ME!(DDSD_WIDTH, DDRAW_dump_DWORD, dwWidth),
        ME!(DDSD_PITCH, DDRAW_dump_DWORD, __bindgen_anon_1.lPitch),
        ME!(DDSD_LINEARSIZE, DDRAW_dump_DWORD, __bindgen_anon_1.dwLinearSize),
        ME!(DDSD_BACKBUFFERCOUNT, DDRAW_dump_DWORD, __bindgen_anon_2.dwBackBufferCount),
        ME!(DDSD_MIPMAPCOUNT, DDRAW_dump_DWORD, __bindgen_anon_3.dwMipMapCount),
        ME!(DDSD_REFRESHRATE, DDRAW_dump_DWORD, __bindgen_anon_3.dwRefreshRate),
        ME!(DDSD_ALPHABITDEPTH, DDRAW_dump_DWORD, dwAlphaBitDepth),
        ME!(DDSD_LPSURFACE, DDRAW_dump_PTR, lpSurface),
        ME!(DDSD_CKDESTOVERLAY, DDRAW_dump_DDCOLORKEY, __bindgen_anon_4.ddckCKDestOverlay),
        ME!(DDSD_CKDESTBLT, DDRAW_dump_DDCOLORKEY, ddckCKDestBlt),
        ME!(DDSD_CKSRCOVERLAY, DDRAW_dump_DDCOLORKEY, ddckCKSrcOverlay),
        ME!(DDSD_CKSRCBLT, DDRAW_dump_DDCOLORKEY, ddckCKSrcBlt),
        ME!(DDSD_PIXELFORMAT, DDRAW_dump_pixelformat, __bindgen_anon_5.ddpfPixelFormat)
    ];
    static mut members_caps: [member_info; 1] = [
        ME!(DDSD_CAPS, DDRAW_dump_DDSCAPS, ddsCaps)
    ];
    static mut members_caps2: [member_info; 1] = [
        ME!(DDSD_CAPS, DDRAW_dump_DDSCAPS2, ddsCaps)
    ];

    if lpddsd.is_null() {
    } else {
        if (*lpddsd).dwSize as usize >= core::mem::size_of::<DDSURFACEDESC2>() {
            DDRAW_dump_members(
                (*lpddsd).dwFlags,
                lpddsd as *const core::ffi::c_void,
                core::ptr::addr_of!(members_caps2) as *const member_info,
                1,
            );
        } else {
            DDRAW_dump_members(
                (*lpddsd).dwFlags,
                lpddsd as *const core::ffi::c_void,
                core::ptr::addr_of!(members_caps) as *const member_info,
                1,
            );
        }
        DDRAW_dump_members(
            (*lpddsd).dwFlags,
            lpddsd as *const core::ffi::c_void,
            core::ptr::addr_of!(members) as *const member_info,
            (ARRAY_SIZE!(members)) as size_t,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn dump_D3DMATRIX(mat: *const D3DMATRIX) {
    // TRACE(" %f %f %f %f\n", mat->_11, mat->_12, mat->_13, mat->_14);
    // TRACE(" %f %f %f %f\n", mat->_21, mat->_22, mat->_23, mat->_24);
    // TRACE(" %f %f %f %f\n", mat->_31, mat->_32, mat->_33, mat->_34);
    // TRACE(" %f %f %f %f\n", mat->_41, mat->_42, mat->_43, mat->_44);
}
#[no_mangle]
pub unsafe extern "C" fn get_flexible_vertex_size(mut d3dvtVertexType: DWORD) -> DWORD {
    let mut size: DWORD = 0;
    let mut i: DWORD = 0;

    if (d3dvtVertexType & D3DFVF_NORMAL) != 0 {
        size += 3 * (core::mem::size_of::<D3DVALUE>() as DWORD);
    }
    if (d3dvtVertexType & D3DFVF_DIFFUSE) != 0 {
        size += core::mem::size_of::<DWORD>() as DWORD;
    }
    if (d3dvtVertexType & D3DFVF_SPECULAR) != 0 {
        size += core::mem::size_of::<DWORD>() as DWORD;
    }
    if (d3dvtVertexType & D3DFVF_RESERVED1) != 0 {
        size += core::mem::size_of::<DWORD>() as DWORD;
    }
    match d3dvtVertexType & D3DFVF_POSITION_MASK {
        D3DFVF_XYZ => {
            size += 3 * (core::mem::size_of::<D3DVALUE>() as DWORD);
        }
        D3DFVF_XYZRHW => {
            size += 4 * (core::mem::size_of::<D3DVALUE>() as DWORD);
        }
        D3DFVF_XYZB1 => {
            size += 4 * (core::mem::size_of::<D3DVALUE>() as DWORD);
        }
        D3DFVF_XYZB2 => {
            size += 5 * (core::mem::size_of::<D3DVALUE>() as DWORD);
        }
        D3DFVF_XYZB3 => {
            size += 6 * (core::mem::size_of::<D3DVALUE>() as DWORD);
        }
        D3DFVF_XYZB4 => {
            size += 7 * (core::mem::size_of::<D3DVALUE>() as DWORD);
        }
        D3DFVF_XYZB5 => {
            size += 8 * (core::mem::size_of::<D3DVALUE>() as DWORD);
        }
        _ => {
            ERR!("Unexpected position mask\n");
        }
    }
    i = 0;
    while i < GET_TEXCOUNT_FROM_FVF!(d3dvtVertexType) {
        // GET_TEXCOORD_SIZE_FROM_FVF(t, i) is `(((t >> (16 + 2*i)) + 1) & 3) + 1`. The texcoord
        // count reaches 15, so the shift reaches 44: undefined in C, masked by x86, and a
        // panic (ud2, scan_ud2) in Rust. wrapping_shr is the hardware's masked shift, and the
        // DWORD arithmetic wraps as C's unsigned does.
        let tex = ((d3dvtVertexType.wrapping_shr(16 + 2 * i)).wrapping_add(1) & 0x03) + 1;
        size = size.wrapping_add(tex.wrapping_mul(core::mem::size_of::<D3DVALUE>() as DWORD));
        i += 1;
    }

    return size;
}
#[no_mangle]
pub unsafe extern "C" fn DDRAW_Convert_DDSCAPS_1_To_2(pIn: *const DDSCAPS, pOut: *mut DDSCAPS2) {
    (*pOut).dwCaps = (*pIn).dwCaps;
    (*pOut).dwCaps2 = 0 as DWORD;
    (*pOut).dwCaps3 = 0 as DWORD;
    (*pOut).__bindgen_anon_1.dwCaps4 = 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn DDRAW_Convert_DDDEVICEIDENTIFIER_2_To_1(pIn: *const DDDEVICEIDENTIFIER2, pOut: *mut DDDEVICEIDENTIFIER) {
    memcpy(pOut as *mut core::ffi::c_void, pIn as *const core::ffi::c_void, core::mem::size_of::<DDDEVICEIDENTIFIER>());
}
#[no_mangle]
pub unsafe extern "C" fn DDRAW_dump_cooperativelevel(cooplevel: DWORD) {
    let flags: [flag_info; 9] = [
        FE!(DDSCL_FULLSCREEN),
        FE!(DDSCL_ALLOWREBOOT),
        FE!(DDSCL_NOWINDOWCHANGES),
        FE!(DDSCL_NORMAL),
        FE!(DDSCL_ALLOWMODEX),
        FE!(DDSCL_EXCLUSIVE),
        FE!(DDSCL_SETFOCUSWINDOW),
        FE!(DDSCL_SETDEVICEWINDOW),
        FE!(DDSCL_CREATEDEVICEWINDOW),
    ];

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        DDRAW_dump_flags(
            cooplevel,
            &raw const flags as *const flag_info,
            ARRAY_SIZE!(flags) as size_t,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn DDRAW_dump_DDCAPS(lpcaps: *const DDCAPS) {
    static mut flags1: [flag_info; 32] = [
        FE!(DDCAPS_3D),
        FE!(DDCAPS_ALIGNBOUNDARYDEST),
        FE!(DDCAPS_ALIGNSIZEDEST),
        FE!(DDCAPS_ALIGNBOUNDARYSRC),
        FE!(DDCAPS_ALIGNSIZESRC),
        FE!(DDCAPS_ALIGNSTRIDE),
        FE!(DDCAPS_BLT),
        FE!(DDCAPS_BLTQUEUE),
        FE!(DDCAPS_BLTFOURCC),
        FE!(DDCAPS_BLTSTRETCH),
        FE!(DDCAPS_GDI),
        FE!(DDCAPS_OVERLAY),
        FE!(DDCAPS_OVERLAYCANTCLIP),
        FE!(DDCAPS_OVERLAYFOURCC),
        FE!(DDCAPS_OVERLAYSTRETCH),
        FE!(DDCAPS_PALETTE),
        FE!(DDCAPS_PALETTEVSYNC),
        FE!(DDCAPS_READSCANLINE),
        FE!(DDCAPS_STEREOVIEW),
        FE!(DDCAPS_VBI),
        FE!(DDCAPS_ZBLTS),
        FE!(DDCAPS_ZOVERLAYS),
        FE!(DDCAPS_COLORKEY),
        FE!(DDCAPS_ALPHA),
        FE!(DDCAPS_COLORKEYHWASSIST),
        FE!(DDCAPS_NOHARDWARE),
        FE!(DDCAPS_BLTCOLORFILL),
        FE!(DDCAPS_BANKSWITCHED),
        FE!(DDCAPS_BLTDEPTHFILL),
        FE!(DDCAPS_CANCLIP),
        FE!(DDCAPS_CANCLIPSTRETCHED),
        FE!(DDCAPS_CANBLTSYSMEM),
    ];
    static mut flags2: [flag_info; 25] = [
        FE!(DDCAPS2_CERTIFIED),
        FE!(DDCAPS2_NO2DDURING3DSCENE),
        FE!(DDCAPS2_VIDEOPORT),
        FE!(DDCAPS2_AUTOFLIPOVERLAY),
        FE!(DDCAPS2_CANBOBINTERLEAVED),
        FE!(DDCAPS2_CANBOBNONINTERLEAVED),
        FE!(DDCAPS2_COLORCONTROLOVERLAY),
        FE!(DDCAPS2_COLORCONTROLPRIMARY),
        FE!(DDCAPS2_CANDROPZ16BIT),
        FE!(DDCAPS2_NONLOCALVIDMEM),
        FE!(DDCAPS2_NONLOCALVIDMEMCAPS),
        FE!(DDCAPS2_NOPAGELOCKREQUIRED),
        FE!(DDCAPS2_WIDESURFACES),
        FE!(DDCAPS2_CANFLIPODDEVEN),
        FE!(DDCAPS2_CANBOBHARDWARE),
        FE!(DDCAPS2_COPYFOURCC),
        FE!(DDCAPS2_PRIMARYGAMMA),
        FE!(DDCAPS2_CANRENDERWINDOWED),
        FE!(DDCAPS2_CANCALIBRATEGAMMA),
        FE!(DDCAPS2_FLIPINTERVAL),
        FE!(DDCAPS2_FLIPNOVSYNC),
        FE!(DDCAPS2_CANMANAGETEXTURE),
        FE!(DDCAPS2_TEXMANINNONLOCALVIDMEM),
        FE!(DDCAPS2_STEREO),
        FE!(DDCAPS2_SYSTONONLOCAL_AS_SYSTOLOCAL),
    ];
    static mut flags3: [flag_info; 19] = [
        FE!(DDCKEYCAPS_DESTBLT),
        FE!(DDCKEYCAPS_DESTBLTCLRSPACE),
        FE!(DDCKEYCAPS_DESTBLTCLRSPACEYUV),
        FE!(DDCKEYCAPS_DESTBLTYUV),
        FE!(DDCKEYCAPS_DESTOVERLAY),
        FE!(DDCKEYCAPS_DESTOVERLAYCLRSPACE),
        FE!(DDCKEYCAPS_DESTOVERLAYCLRSPACEYUV),
        FE!(DDCKEYCAPS_DESTOVERLAYONEACTIVE),
        FE!(DDCKEYCAPS_DESTOVERLAYYUV),
        FE!(DDCKEYCAPS_SRCBLT),
        FE!(DDCKEYCAPS_SRCBLTCLRSPACE),
        FE!(DDCKEYCAPS_SRCBLTCLRSPACEYUV),
        FE!(DDCKEYCAPS_SRCBLTYUV),
        FE!(DDCKEYCAPS_SRCOVERLAY),
        FE!(DDCKEYCAPS_SRCOVERLAYCLRSPACE),
        FE!(DDCKEYCAPS_SRCOVERLAYCLRSPACEYUV),
        FE!(DDCKEYCAPS_SRCOVERLAYONEACTIVE),
        FE!(DDCKEYCAPS_SRCOVERLAYYUV),
        FE!(DDCKEYCAPS_NOCOSTOVERLAY),
    ];
    static mut flags4: [flag_info; 28] = [
        FE!(DDFXCAPS_BLTALPHA),
        FE!(DDFXCAPS_OVERLAYALPHA),
        FE!(DDFXCAPS_BLTARITHSTRETCHYN),
        FE!(DDFXCAPS_BLTARITHSTRETCHY),
        FE!(DDFXCAPS_BLTMIRRORLEFTRIGHT),
        FE!(DDFXCAPS_BLTMIRRORUPDOWN),
        FE!(DDFXCAPS_BLTROTATION),
        FE!(DDFXCAPS_BLTROTATION90),
        FE!(DDFXCAPS_BLTSHRINKX),
        FE!(DDFXCAPS_BLTSHRINKXN),
        FE!(DDFXCAPS_BLTSHRINKY),
        FE!(DDFXCAPS_BLTSHRINKYN),
        FE!(DDFXCAPS_BLTSTRETCHX),
        FE!(DDFXCAPS_BLTSTRETCHXN),
        FE!(DDFXCAPS_BLTSTRETCHY),
        FE!(DDFXCAPS_BLTSTRETCHYN),
        FE!(DDFXCAPS_OVERLAYARITHSTRETCHY),
        FE!(DDFXCAPS_OVERLAYARITHSTRETCHYN),
        FE!(DDFXCAPS_OVERLAYSHRINKX),
        FE!(DDFXCAPS_OVERLAYSHRINKXN),
        FE!(DDFXCAPS_OVERLAYSHRINKY),
        FE!(DDFXCAPS_OVERLAYSHRINKYN),
        FE!(DDFXCAPS_OVERLAYSTRETCHX),
        FE!(DDFXCAPS_OVERLAYSTRETCHXN),
        FE!(DDFXCAPS_OVERLAYSTRETCHY),
        FE!(DDFXCAPS_OVERLAYSTRETCHYN),
        FE!(DDFXCAPS_OVERLAYMIRRORLEFTRIGHT),
        FE!(DDFXCAPS_OVERLAYMIRRORUPDOWN),
    ];
    static mut flags5: [flag_info; 10] = [
        FE!(DDFXALPHACAPS_BLTALPHAEDGEBLEND),
        FE!(DDFXALPHACAPS_BLTALPHAPIXELS),
        FE!(DDFXALPHACAPS_BLTALPHAPIXELSNEG),
        FE!(DDFXALPHACAPS_BLTALPHASURFACES),
        FE!(DDFXALPHACAPS_BLTALPHASURFACESNEG),
        FE!(DDFXALPHACAPS_OVERLAYALPHAEDGEBLEND),
        FE!(DDFXALPHACAPS_OVERLAYALPHAPIXELS),
        FE!(DDFXALPHACAPS_OVERLAYALPHAPIXELSNEG),
        FE!(DDFXALPHACAPS_OVERLAYALPHASURFACES),
        FE!(DDFXALPHACAPS_OVERLAYALPHASURFACESNEG),
    ];
    static mut flags6: [flag_info; 11] = [
        FE!(DDPCAPS_4BIT),
        FE!(DDPCAPS_8BITENTRIES),
        FE!(DDPCAPS_8BIT),
        FE!(DDPCAPS_INITIALIZE),
        FE!(DDPCAPS_PRIMARYSURFACE),
        FE!(DDPCAPS_PRIMARYSURFACELEFT),
        FE!(DDPCAPS_ALLOW256),
        FE!(DDPCAPS_VSYNC),
        FE!(DDPCAPS_1BIT),
        FE!(DDPCAPS_2BIT),
        FE!(DDPCAPS_ALPHA),
    ];
    static mut flags7: [flag_info; 5] = [
        FE!(DDSVCAPS_RESERVED1),
        FE!(DDSVCAPS_RESERVED2),
        FE!(DDSVCAPS_RESERVED3),
        FE!(DDSVCAPS_RESERVED4),
        FE!(DDSVCAPS_STEREOSEQUENTIAL),
    ];

    WINE_TRACE!(" - dwSize : %lu\n", (*lpcaps).dwSize);
    WINE_TRACE!(" - dwCaps : ");
    DDRAW_dump_flags(
        (*lpcaps).dwCaps,
        &raw const flags1 as *const flag_info,
        (ARRAY_SIZE!(flags1)) as size_t,
    );
    WINE_TRACE!(" - dwCaps2 : ");
    DDRAW_dump_flags(
        (*lpcaps).dwCaps2,
        &raw const flags2 as *const flag_info,
        (ARRAY_SIZE!(flags2)) as size_t,
    );
    WINE_TRACE!(" - dwCKeyCaps : ");
    DDRAW_dump_flags(
        (*lpcaps).dwCKeyCaps,
        &raw const flags3 as *const flag_info,
        (ARRAY_SIZE!(flags3)) as size_t,
    );
    WINE_TRACE!(" - dwFXCaps : ");
    DDRAW_dump_flags(
        (*lpcaps).dwFXCaps,
        &raw const flags4 as *const flag_info,
        (ARRAY_SIZE!(flags4)) as size_t,
    );
    WINE_TRACE!(" - dwFXAlphaCaps : ");
    DDRAW_dump_flags(
        (*lpcaps).dwFXAlphaCaps,
        &raw const flags5 as *const flag_info,
        (ARRAY_SIZE!(flags5)) as size_t,
    );
    WINE_TRACE!(" - dwPalCaps : ");
    DDRAW_dump_flags(
        (*lpcaps).dwPalCaps,
        &raw const flags6 as *const flag_info,
        (ARRAY_SIZE!(flags6)) as size_t,
    );
    WINE_TRACE!(" - dwSVCaps : ");
    DDRAW_dump_flags(
        (*lpcaps).dwSVCaps,
        &raw const flags7 as *const flag_info,
        (ARRAY_SIZE!(flags7)) as size_t,
    );
    WINE_TRACE!("...\n");
    WINE_TRACE!(" - dwNumFourCCCodes : %lu\n", (*lpcaps).dwNumFourCCCodes);
    WINE_TRACE!(" - dwCurrVisibleOverlays : %lu\n", (*lpcaps).dwCurrVisibleOverlays);
    WINE_TRACE!(" - dwMinOverlayStretch : %lu\n", (*lpcaps).dwMinOverlayStretch);
    WINE_TRACE!(" - dwMaxOverlayStretch : %lu\n", (*lpcaps).dwMaxOverlayStretch);
    WINE_TRACE!("...\n");
    WINE_TRACE!(" - ddsCaps : ");
    DDRAW_dump_DDSCAPS2(&raw const (*lpcaps).ddsCaps);
}
#[no_mangle]
pub unsafe extern "C" fn multiply_matrix(
    dst: *mut wined3d_matrix,
    src1: *const wined3d_matrix,
    src2: *const wined3d_matrix,
) {
    let mut temp: wined3d_matrix = wined3d_matrix {
        _11: 0.0,
        _12: 0.0,
        _13: 0.0,
        _14: 0.0,
        _21: 0.0,
        _22: 0.0,
        _23: 0.0,
        _24: 0.0,
        _31: 0.0,
        _32: 0.0,
        _33: 0.0,
        _34: 0.0,
        _41: 0.0,
        _42: 0.0,
        _43: 0.0,
        _44: 0.0,
    };
    temp._11 = (*src1)._11 * (*src2)._11
        + (*src1)._21 * (*src2)._12
        + (*src1)._31 * (*src2)._13
        + (*src1)._41 * (*src2)._14;
    temp._21 = (*src1)._11 * (*src2)._21
        + (*src1)._21 * (*src2)._22
        + (*src1)._31 * (*src2)._23
        + (*src1)._41 * (*src2)._24;
    temp._31 = (*src1)._11 * (*src2)._31
        + (*src1)._21 * (*src2)._32
        + (*src1)._31 * (*src2)._33
        + (*src1)._41 * (*src2)._34;
    temp._41 = (*src1)._11 * (*src2)._41
        + (*src1)._21 * (*src2)._42
        + (*src1)._31 * (*src2)._43
        + (*src1)._41 * (*src2)._44;
    temp._12 = (*src1)._12 * (*src2)._11
        + (*src1)._22 * (*src2)._12
        + (*src1)._32 * (*src2)._13
        + (*src1)._42 * (*src2)._14;
    temp._22 = (*src1)._12 * (*src2)._21
        + (*src1)._22 * (*src2)._22
        + (*src1)._32 * (*src2)._23
        + (*src1)._42 * (*src2)._24;
    temp._32 = (*src1)._12 * (*src2)._31
        + (*src1)._22 * (*src2)._32
        + (*src1)._32 * (*src2)._33
        + (*src1)._42 * (*src2)._34;
    temp._42 = (*src1)._12 * (*src2)._41
        + (*src1)._22 * (*src2)._42
        + (*src1)._32 * (*src2)._43
        + (*src1)._42 * (*src2)._44;
    temp._13 = (*src1)._13 * (*src2)._11
        + (*src1)._23 * (*src2)._12
        + (*src1)._33 * (*src2)._13
        + (*src1)._43 * (*src2)._14;
    temp._23 = (*src1)._13 * (*src2)._21
        + (*src1)._23 * (*src2)._22
        + (*src1)._33 * (*src2)._23
        + (*src1)._43 * (*src2)._24;
    temp._33 = (*src1)._13 * (*src2)._31
        + (*src1)._23 * (*src2)._32
        + (*src1)._33 * (*src2)._33
        + (*src1)._43 * (*src2)._34;
    temp._43 = (*src1)._13 * (*src2)._41
        + (*src1)._23 * (*src2)._42
        + (*src1)._33 * (*src2)._43
        + (*src1)._43 * (*src2)._44;
    temp._14 = (*src1)._14 * (*src2)._11
        + (*src1)._24 * (*src2)._12
        + (*src1)._34 * (*src2)._13
        + (*src1)._44 * (*src2)._14;
    temp._24 = (*src1)._14 * (*src2)._21
        + (*src1)._24 * (*src2)._22
        + (*src1)._34 * (*src2)._23
        + (*src1)._44 * (*src2)._24;
    temp._34 = (*src1)._14 * (*src2)._31
        + (*src1)._24 * (*src2)._32
        + (*src1)._34 * (*src2)._33
        + (*src1)._44 * (*src2)._34;
    temp._44 = (*src1)._14 * (*src2)._41
        + (*src1)._24 * (*src2)._42
        + (*src1)._34 * (*src2)._43
        + (*src1)._44 * (*src2)._44;
    *dst = temp;
}
#[no_mangle]
pub unsafe extern "C" fn hr_ddraw_from_wined3d(hr: HRESULT) -> HRESULT {
    if hr == WINED3DERR_INVALIDCALL {
        return DDERR_INVALIDPARAMS;
    } else if hr == WINED3DERR_NOTAVAILABLE {
        return DDERR_UNSUPPORTED;
    } else if hr == WINEDDERR_NOTAOVERLAYSURFACE {
        return DDERR_NOTAOVERLAYSURFACE;
    } else if hr == WINEDDERR_OVERLAYNOTVISIBLE {
        return DDERR_OVERLAYNOTVISIBLE;
    } else {
        return hr;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DDSD_to_DDSD2(
    in_0: *const DDSURFACEDESC,
    out: *mut DDSURFACEDESC2,
) {
    /* The output of this function is never passed to the application directly, so
     * the memset is not strictly needed. CreateSurface still has problems with this
     * though. Don't forget to set ddsCaps.dwCaps2/3/4 to 0 when removing this */
    memset(
        out as *mut ::core::ffi::c_void,
        0x00 as ::core::ffi::c_int,
        ::core::mem::size_of::<DDSURFACEDESC2>() as size_t,
    );
    (*out).dwSize = ::core::mem::size_of::<DDSURFACEDESC2>() as DWORD;
    (*out).dwFlags = (*in_0).dwFlags & !(DDSD_ZBUFFERBITDEPTH as DWORD);
    if ((*in_0).dwFlags & (DDSD_WIDTH as DWORD)) != 0 {
        (*out).dwWidth = (*in_0).dwWidth;
    }
    if ((*in_0).dwFlags & (DDSD_HEIGHT as DWORD)) != 0 {
        (*out).dwHeight = (*in_0).dwHeight;
    }
    if ((*in_0).dwFlags & (DDSD_PIXELFORMAT as DWORD)) != 0 {
        (*out).__bindgen_anon_5.ddpfPixelFormat = (*in_0).ddpfPixelFormat;
    } else if ((*in_0).dwFlags & (DDSD_ZBUFFERBITDEPTH as DWORD)) != 0 {
        (*out).dwFlags |= DDSD_PIXELFORMAT as DWORD;
        memset(
            ::core::ptr::addr_of_mut!((*out).__bindgen_anon_5.ddpfPixelFormat)
                as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<DDPIXELFORMAT>() as size_t,
        );
        (*out).__bindgen_anon_5.ddpfPixelFormat.dwSize =
            ::core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
        (*out).__bindgen_anon_5.ddpfPixelFormat.dwFlags = DDPF_ZBUFFER as DWORD;
        (*out)

            .__bindgen_anon_5.ddpfPixelFormat

            .__bindgen_anon_1.dwZBufferBitDepth = (*in_0).__bindgen_anon_2.dwZBufferBitDepth;
        /* 0 is not a valid DDSURFACEDESC / DDPIXELFORMAT on either side of the
         * conversion */
        (*out)

            .__bindgen_anon_5.ddpfPixelFormat

            .__bindgen_anon_3.dwZBitMask = (!(0 as ::core::ffi::c_uint)
            >> ((32 as DWORD)
                .wrapping_sub((*in_0).__bindgen_anon_2.dwZBufferBitDepth)))
            as DWORD;
    }
    /* ddsCaps is read even without DDSD_CAPS set. See dsurface:no_ddsd_caps_test */
    (*out).ddsCaps.dwCaps = (*in_0).ddsCaps.dwCaps;
    if ((*in_0).dwFlags & (DDSD_PITCH as DWORD)) != 0 {
        (*out).__bindgen_anon_1.lPitch = (*in_0).__bindgen_anon_1.lPitch;
    }
    if ((*in_0).dwFlags & (DDSD_BACKBUFFERCOUNT as DWORD)) != 0 {
        (*out).__bindgen_anon_2.dwBackBufferCount = (*in_0).dwBackBufferCount;
    }
    if ((*in_0).dwFlags & (DDSD_ALPHABITDEPTH as DWORD)) != 0 {
        (*out).dwAlphaBitDepth = (*in_0).dwAlphaBitDepth;
    }
    /* DDraw(native, and wine) does not set the DDSD_LPSURFACE, so always copy */
    (*out).lpSurface = (*in_0).lpSurface;
    if ((*in_0).dwFlags & (DDSD_CKDESTOVERLAY as DWORD)) != 0 {
        (*out).__bindgen_anon_4.ddckCKDestOverlay = (*in_0).ddckCKDestOverlay;
    }
    if ((*in_0).dwFlags & (DDSD_CKDESTBLT as DWORD)) != 0 {
        (*out).ddckCKDestBlt = (*in_0).ddckCKDestBlt;
    }
    if ((*in_0).dwFlags & (DDSD_CKSRCOVERLAY as DWORD)) != 0 {
        (*out).ddckCKSrcOverlay = (*in_0).ddckCKSrcOverlay;
    }
    if ((*in_0).dwFlags & (DDSD_CKSRCBLT as DWORD)) != 0 {
        (*out).ddckCKSrcBlt = (*in_0).ddckCKSrcBlt;
    }
    if ((*in_0).dwFlags & (DDSD_MIPMAPCOUNT as DWORD)) != 0 {
        (*out).__bindgen_anon_3.dwMipMapCount = (*in_0).__bindgen_anon_2.dwMipMapCount;
    }
    if ((*in_0).dwFlags & (DDSD_REFRESHRATE as DWORD)) != 0 {
        (*out).__bindgen_anon_3.dwRefreshRate = (*in_0).__bindgen_anon_2.dwRefreshRate;
    }
    if ((*in_0).dwFlags & (DDSD_LINEARSIZE as DWORD)) != 0 {
        (*out).__bindgen_anon_1.dwLinearSize = (*in_0).__bindgen_anon_1.dwLinearSize;
    }
    /* Does not exist in DDSURFACEDESC:
     * DDSD_TEXTURESTAGE, DDSD_FVF, DDSD_SRCVBHANDLE,
     */
}
#[no_mangle]
pub unsafe extern "C" fn DDSD2_to_DDSD(in_0: *const DDSURFACEDESC2, out: *mut DDSURFACEDESC) {
    memset(
        out as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<DDSURFACEDESC>() as size_t,
    );
    (*out).dwSize = ::core::mem::size_of::<DDSURFACEDESC>() as DWORD;
    (*out).dwFlags = (*in_0).dwFlags;
    if (*in_0).dwFlags & DDSD_WIDTH as DWORD != 0 {
        (*out).dwWidth = (*in_0).dwWidth;
    }
    if (*in_0).dwFlags & DDSD_HEIGHT as DWORD != 0 {
        (*out).dwHeight = (*in_0).dwHeight;
    }
    if (*in_0).dwFlags & DDSD_PIXELFORMAT as DWORD != 0 {
        (*out).ddpfPixelFormat = (*in_0).__bindgen_anon_5.ddpfPixelFormat;
        if (*in_0).dwFlags & DDSD_CAPS as DWORD != 0
            && (*in_0).ddsCaps.dwCaps & DDSCAPS_ZBUFFER as DWORD != 0
        {
            /* Z buffers have DDSD_ZBUFFERBITDEPTH set, but not DDSD_PIXELFORMAT. They do
             * have valid data in ddpfPixelFormat though */
            (*out).dwFlags &= !(DDSD_PIXELFORMAT as DWORD);
            (*out).dwFlags |= DDSD_ZBUFFERBITDEPTH as DWORD;
            (*out).__bindgen_anon_2.dwZBufferBitDepth = (*in_0)

                .__bindgen_anon_5.ddpfPixelFormat

                .__bindgen_anon_1.dwZBufferBitDepth;
        }
    }
    /* ddsCaps is read even without DDSD_CAPS set. See dsurface:no_ddsd_caps_test */
    (*out).ddsCaps.dwCaps = (*in_0).ddsCaps.dwCaps;
    if (*in_0).dwFlags & DDSD_PITCH as DWORD != 0 {
        (*out).__bindgen_anon_1.lPitch = (*in_0).__bindgen_anon_1.lPitch;
    }
    if (*in_0).dwFlags & DDSD_BACKBUFFERCOUNT as DWORD != 0 {
        (*out).dwBackBufferCount = (*in_0).__bindgen_anon_2.dwBackBufferCount;
    }
    if (*in_0).dwFlags & DDSD_ZBUFFERBITDEPTH as DWORD != 0 {
        (*out).__bindgen_anon_2.dwZBufferBitDepth = (*in_0).__bindgen_anon_3.dwMipMapCount; /* same union */
    }
    if (*in_0).dwFlags & DDSD_ALPHABITDEPTH as DWORD != 0 {
        (*out).dwAlphaBitDepth = (*in_0).dwAlphaBitDepth;
    }
    /* DDraw(native, and wine) does not set the DDSD_LPSURFACE, so always copy */
    (*out).lpSurface = (*in_0).lpSurface;
    if (*in_0).dwFlags & DDSD_CKDESTOVERLAY as DWORD != 0 {
        (*out).ddckCKDestOverlay = (*in_0).__bindgen_anon_4.ddckCKDestOverlay;
    }
    if (*in_0).dwFlags & DDSD_CKDESTBLT as DWORD != 0 {
        (*out).ddckCKDestBlt = (*in_0).ddckCKDestBlt;
    }
    if (*in_0).dwFlags & DDSD_CKSRCOVERLAY as DWORD != 0 {
        (*out).ddckCKSrcOverlay = (*in_0).ddckCKSrcOverlay;
    }
    if (*in_0).dwFlags & DDSD_CKSRCBLT as DWORD != 0 {
        (*out).ddckCKSrcBlt = (*in_0).ddckCKSrcBlt;
    }
    if (*in_0).dwFlags & DDSD_MIPMAPCOUNT as DWORD != 0 {
        (*out).__bindgen_anon_2.dwMipMapCount = (*in_0).__bindgen_anon_3.dwMipMapCount;
    }
    if (*in_0).dwFlags & DDSD_REFRESHRATE as DWORD != 0 {
        (*out).__bindgen_anon_2.dwRefreshRate = (*in_0).__bindgen_anon_3.dwRefreshRate;
    }
    if (*in_0).dwFlags & DDSD_LINEARSIZE as DWORD != 0 {
        (*out).__bindgen_anon_1.dwLinearSize = (*in_0).__bindgen_anon_1.dwLinearSize;
    }
    /* Does not exist in DDSURFACEDESC:
     * DDSD_TEXTURESTAGE, DDSD_FVF, DDSD_SRCVBHANDLE,
     */
    if (*in_0).dwFlags & DDSD_TEXTURESTAGE as DWORD != 0 {
        WARN!("Does not exist in DDSURFACEDESC: DDSD_TEXTURESTAGE\n");
    }
    if (*in_0).dwFlags & DDSD_FVF as DWORD != 0 {
        WARN!("Does not exist in DDSURFACEDESC: DDSD_FVF\n");
    }
    if (*in_0).dwFlags & DDSD_SRCVBHANDLE as DWORD != 0 {
        WARN!("Does not exist in DDSURFACEDESC: DDSD_SRCVBHANDLE\n");
    }
    (*out).dwFlags &= !((DDSD_TEXTURESTAGE | DDSD_FVF | DDSD_SRCVBHANDLE)
        as DWORD);
}
