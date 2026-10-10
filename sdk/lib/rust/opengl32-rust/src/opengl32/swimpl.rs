use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut pixel_formats_32: [pixel_format; 36] = [
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
];

static mut pixel_formats_24: [pixel_format; 36] = [
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
];

static mut pixel_formats_16: [pixel_format; 36] = [
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 11 as BYTE,
        cGreenBits: 6 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
];

static mut pixel_formats_8: [pixel_format; 36] = [
    pixel_format {
        dwFlags: SB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS_PALETTE as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS_WINDOW as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: DB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 8 as BYTE,
        cRedBits: 3 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 3 as BYTE,
        cGreenShift: 3 as BYTE,
        cBlueBits: 2 as BYTE,
        cBlueShift: 6 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 24 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 64 as BYTE,
        cAccumRedBits: 16 as BYTE,
        cAccumGreenBits: 16 as BYTE,
        cAccumBlueBits: 16 as BYTE,
        cAccumAlphaBits: 16 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 32 as BYTE,
        cRedBits: 8 as BYTE,
        cRedShift: 16 as BYTE,
        cGreenBits: 8 as BYTE,
        cGreenShift: 8 as BYTE,
        cBlueBits: 8 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 11 as BYTE,
        cAccumGreenBits: 11 as BYTE,
        cAccumBlueBits: 10 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 32 as BYTE,
        cAccumRedBits: 8 as BYTE,
        cAccumGreenBits: 8 as BYTE,
        cAccumBlueBits: 8 as BYTE,
        cAccumAlphaBits: 8 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 16 as BYTE,
        cRedBits: 5 as BYTE,
        cRedShift: 10 as BYTE,
        cGreenBits: 5 as BYTE,
        cGreenShift: 5 as BYTE,
        cBlueBits: 5 as BYTE,
        cBlueShift: 0 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 5 as BYTE,
        cAccumGreenBits: 6 as BYTE,
        cAccumBlueBits: 5 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_RGBA as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 8 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 16 as BYTE,
        cAccumRedBits: 4 as BYTE,
        cAccumGreenBits: 4 as BYTE,
        cAccumBlueBits: 4 as BYTE,
        cAccumAlphaBits: 4 as BYTE,
        cDepthBits: 16 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 32 as BYTE,
    },
    pixel_format {
        dwFlags: SB_FLAGS as DWORD,
        iPixelType: PFD_TYPE_COLORINDEX as BYTE,
        cColorBits: 4 as BYTE,
        cRedBits: 1 as BYTE,
        cRedShift: 0 as BYTE,
        cGreenBits: 1 as BYTE,
        cGreenShift: 1 as BYTE,
        cBlueBits: 1 as BYTE,
        cBlueShift: 2 as BYTE,
        cAlphaBits: 0 as BYTE,
        cAlphaShift: 0 as BYTE,
        cAccumBits: 0 as BYTE,
        cAccumRedBits: 0 as BYTE,
        cAccumGreenBits: 0 as BYTE,
        cAccumBlueBits: 0 as BYTE,
        cAccumAlphaBits: 0 as BYTE,
        cDepthBits: 16 as BYTE,
    },
];

static mut sw_api_table: GLCLTPROCTABLE = unsafe {
    _GLCLTPROCTABLE {
        cEntries: (OPENGL_VERSION_110_ENTRIES as i32),
        glDispatchTable: __GLdispatchTableRec {
            NewList: Some(_mesa_NewList),
            EndList: Some(_mesa_EndList),
            CallList: Some(_mesa_CallList),
            CallLists: Some(
                _mesa_CallLists,
            ),
            DeleteLists: Some(_mesa_DeleteLists),
            GenLists: Some(_mesa_GenLists),
            ListBase: Some(_mesa_ListBase),
            Begin: Some(_mesa_Begin),
            Bitmap: Some(
                _mesa_Bitmap,
            ),
            Color3b: Some(_mesa_Color3b),
            Color3bv: Some(_mesa_Color3bv),
            Color3d: Some(
                _mesa_Color3d,
            ),
            Color3dv: Some(_mesa_Color3dv),
            Color3f: Some(_mesa_Color3f),
            Color3fv: Some(_mesa_Color3fv),
            Color3i: Some(_mesa_Color3i),
            Color3iv: Some(_mesa_Color3iv),
            Color3s: Some(_mesa_Color3s),
            Color3sv: Some(_mesa_Color3sv),
            Color3ub: Some(_mesa_Color3ub),
            Color3ubv: Some(_mesa_Color3ubv),
            Color3ui: Some(_mesa_Color3ui),
            Color3uiv: Some(_mesa_Color3uiv),
            Color3us: Some(
                _mesa_Color3us,
            ),
            Color3usv: Some(_mesa_Color3usv),
            Color4b: Some(
                _mesa_Color4b,
            ),
            Color4bv: Some(_mesa_Color4bv),
            Color4d: Some(
                _mesa_Color4d,
            ),
            Color4dv: Some(_mesa_Color4dv),
            Color4f: Some(
                _mesa_Color4f,
            ),
            Color4fv: Some(_mesa_Color4fv),
            Color4i: Some(_mesa_Color4i),
            Color4iv: Some(_mesa_Color4iv),
            Color4s: Some(
                _mesa_Color4s,
            ),
            Color4sv: Some(_mesa_Color4sv),
            Color4ub: Some(
                _mesa_Color4ub,
            ),
            Color4ubv: Some(_mesa_Color4ubv),
            Color4ui: Some(
                _mesa_Color4ui,
            ),
            Color4uiv: Some(_mesa_Color4uiv),
            Color4us: Some(
                _mesa_Color4us,
            ),
            Color4usv: Some(_mesa_Color4usv),
            EdgeFlag: Some(_mesa_EdgeFlag),
            EdgeFlagv: Some(_mesa_EdgeFlagv),
            End: Some(_mesa_End),
            Indexd: Some(_mesa_Indexd),
            Indexdv: Some(_mesa_Indexdv),
            Indexf: Some(_mesa_Indexf),
            Indexfv: Some(_mesa_Indexfv),
            Indexi: Some(_mesa_Indexi),
            Indexiv: Some(_mesa_Indexiv),
            Indexs: Some(_mesa_Indexs),
            Indexsv: Some(_mesa_Indexsv),
            Normal3b: Some(_mesa_Normal3b),
            Normal3bv: Some(_mesa_Normal3bv),
            Normal3d: Some(
                _mesa_Normal3d,
            ),
            Normal3dv: Some(_mesa_Normal3dv),
            Normal3f: Some(_mesa_Normal3f),
            Normal3fv: Some(_mesa_Normal3fv),
            Normal3i: Some(_mesa_Normal3i),
            Normal3iv: Some(_mesa_Normal3iv),
            Normal3s: Some(_mesa_Normal3s),
            Normal3sv: Some(_mesa_Normal3sv),
            RasterPos2d: Some(_mesa_RasterPos2d),
            RasterPos2dv: Some(_mesa_RasterPos2dv),
            RasterPos2f: Some(_mesa_RasterPos2f),
            RasterPos2fv: Some(_mesa_RasterPos2fv),
            RasterPos2i: Some(_mesa_RasterPos2i),
            RasterPos2iv: Some(_mesa_RasterPos2iv),
            RasterPos2s: Some(_mesa_RasterPos2s),
            RasterPos2sv: Some(_mesa_RasterPos2sv),
            RasterPos3d: Some(
                _mesa_RasterPos3d,
            ),
            RasterPos3dv: Some(_mesa_RasterPos3dv),
            RasterPos3f: Some(
                _mesa_RasterPos3f,
            ),
            RasterPos3fv: Some(_mesa_RasterPos3fv),
            RasterPos3i: Some(_mesa_RasterPos3i),
            RasterPos3iv: Some(_mesa_RasterPos3iv),
            RasterPos3s: Some(
                _mesa_RasterPos3s,
            ),
            RasterPos3sv: Some(_mesa_RasterPos3sv),
            RasterPos4d: Some(
                _mesa_RasterPos4d,
            ),
            RasterPos4dv: Some(_mesa_RasterPos4dv),
            RasterPos4f: Some(
                _mesa_RasterPos4f,
            ),
            RasterPos4fv: Some(_mesa_RasterPos4fv),
            RasterPos4i: Some(
                _mesa_RasterPos4i,
            ),
            RasterPos4iv: Some(_mesa_RasterPos4iv),
            RasterPos4s: Some(
                _mesa_RasterPos4s,
            ),
            RasterPos4sv: Some(_mesa_RasterPos4sv),
            Rectd: Some(
                _mesa_Rectd,
            ),
            Rectdv: Some(
                _mesa_Rectdv,
            ),
            Rectf: Some(
                _mesa_Rectf,
            ),
            Rectfv: Some(
                _mesa_Rectfv,
            ),
            Recti: Some(_mesa_Recti),
            Rectiv: Some(_mesa_Rectiv),
            Rects: Some(
                _mesa_Rects,
            ),
            Rectsv: Some(
                _mesa_Rectsv,
            ),
            TexCoord1d: Some(_mesa_TexCoord1d),
            TexCoord1dv: Some(_mesa_TexCoord1dv),
            TexCoord1f: Some(_mesa_TexCoord1f),
            TexCoord1fv: Some(_mesa_TexCoord1fv),
            TexCoord1i: Some(_mesa_TexCoord1i),
            TexCoord1iv: Some(_mesa_TexCoord1iv),
            TexCoord1s: Some(_mesa_TexCoord1s),
            TexCoord1sv: Some(_mesa_TexCoord1sv),
            TexCoord2d: Some(_mesa_TexCoord2d),
            TexCoord2dv: Some(_mesa_TexCoord2dv),
            TexCoord2f: Some(_mesa_TexCoord2f),
            TexCoord2fv: Some(_mesa_TexCoord2fv),
            TexCoord2i: Some(_mesa_TexCoord2i),
            TexCoord2iv: Some(_mesa_TexCoord2iv),
            TexCoord2s: Some(_mesa_TexCoord2s),
            TexCoord2sv: Some(_mesa_TexCoord2sv),
            TexCoord3d: Some(
                _mesa_TexCoord3d,
            ),
            TexCoord3dv: Some(_mesa_TexCoord3dv),
            TexCoord3f: Some(
                _mesa_TexCoord3f,
            ),
            TexCoord3fv: Some(_mesa_TexCoord3fv),
            TexCoord3i: Some(_mesa_TexCoord3i),
            TexCoord3iv: Some(_mesa_TexCoord3iv),
            TexCoord3s: Some(
                _mesa_TexCoord3s,
            ),
            TexCoord3sv: Some(_mesa_TexCoord3sv),
            TexCoord4d: Some(
                _mesa_TexCoord4d,
            ),
            TexCoord4dv: Some(_mesa_TexCoord4dv),
            TexCoord4f: Some(
                _mesa_TexCoord4f,
            ),
            TexCoord4fv: Some(_mesa_TexCoord4fv),
            TexCoord4i: Some(
                _mesa_TexCoord4i,
            ),
            TexCoord4iv: Some(_mesa_TexCoord4iv),
            TexCoord4s: Some(
                _mesa_TexCoord4s,
            ),
            TexCoord4sv: Some(_mesa_TexCoord4sv),
            Vertex2d: Some(_mesa_Vertex2d),
            Vertex2dv: Some(_mesa_Vertex2dv),
            Vertex2f: Some(_mesa_Vertex2f),
            Vertex2fv: Some(_mesa_Vertex2fv),
            Vertex2i: Some(_mesa_Vertex2i),
            Vertex2iv: Some(_mesa_Vertex2iv),
            Vertex2s: Some(_mesa_Vertex2s),
            Vertex2sv: Some(_mesa_Vertex2sv),
            Vertex3d: Some(
                _mesa_Vertex3d,
            ),
            Vertex3dv: Some(_mesa_Vertex3dv),
            Vertex3f: Some(_mesa_Vertex3f),
            Vertex3fv: Some(_mesa_Vertex3fv),
            Vertex3i: Some(_mesa_Vertex3i),
            Vertex3iv: Some(_mesa_Vertex3iv),
            Vertex3s: Some(_mesa_Vertex3s),
            Vertex3sv: Some(_mesa_Vertex3sv),
            Vertex4d: Some(
                _mesa_Vertex4d,
            ),
            Vertex4dv: Some(_mesa_Vertex4dv),
            Vertex4f: Some(
                _mesa_Vertex4f,
            ),
            Vertex4fv: Some(_mesa_Vertex4fv),
            Vertex4i: Some(
                _mesa_Vertex4i,
            ),
            Vertex4iv: Some(_mesa_Vertex4iv),
            Vertex4s: Some(
                _mesa_Vertex4s,
            ),
            Vertex4sv: Some(_mesa_Vertex4sv),
            ClipPlane: Some(_mesa_ClipPlane),
            ColorMaterial: Some(_mesa_ColorMaterial),
            CullFace: Some(_mesa_CullFace),
            Fogf: Some(_mesa_Fogf),
            Fogfv: Some(_mesa_Fogfv),
            Fogi: Some(_mesa_Fogi),
            Fogiv: Some(_mesa_Fogiv),
            FrontFace: Some(_mesa_FrontFace),
            Hint: Some(_mesa_Hint),
            Lightf: Some(_mesa_Lightf),
            Lightfv: Some(
                _mesa_Lightfv,
            ),
            Lighti: Some(_mesa_Lighti),
            Lightiv: Some(
                _mesa_Lightiv,
            ),
            LightModelf: Some(_mesa_LightModelf),
            LightModelfv: Some(
                _mesa_LightModelfv,
            ),
            LightModeli: Some(_mesa_LightModeli),
            LightModeliv: Some(
                _mesa_LightModeliv,
            ),
            LineStipple: Some(_mesa_LineStipple),
            LineWidth: Some(_mesa_LineWidth),
            Materialf: Some(_mesa_Materialf),
            Materialfv: Some(
                _mesa_Materialfv,
            ),
            Materiali: Some(_mesa_Materiali),
            Materialiv: Some(
                _mesa_Materialiv,
            ),
            PointSize: Some(_mesa_PointSize),
            PolygonMode: Some(_mesa_PolygonMode),
            PolygonStipple: Some(
                _mesa_PolygonStipple,
            ),
            Scissor: Some(
                _mesa_Scissor,
            ),
            ShadeModel: Some(_mesa_ShadeModel),
            TexParameterf: Some(
                _mesa_TexParameterf,
            ),
            TexParameterfv: Some(
                _mesa_TexParameterfv,
            ),
            TexParameteri: Some(
                _mesa_TexParameteri,
            ),
            TexParameteriv: Some(
                _mesa_TexParameteriv,
            ),
            TexImage1D: Some(
                _mesa_TexImage1D,
            ),
            TexImage2D: Some(
                _mesa_TexImage2D,
            ),
            TexEnvf: Some(_mesa_TexEnvf),
            TexEnvfv: Some(
                _mesa_TexEnvfv,
            ),
            TexEnvi: Some(_mesa_TexEnvi),
            TexEnviv: Some(
                _mesa_TexEnviv,
            ),
            TexGend: Some(_mesa_TexGend),
            TexGendv: Some(
                _mesa_TexGendv,
            ),
            TexGenf: Some(_mesa_TexGenf),
            TexGenfv: Some(
                _mesa_TexGenfv,
            ),
            TexGeni: Some(_mesa_TexGeni),
            TexGeniv: Some(
                _mesa_TexGeniv,
            ),
            FeedbackBuffer: Some(
                _mesa_FeedbackBuffer,
            ),
            SelectBuffer: Some(
                _mesa_SelectBuffer,
            ),
            RenderMode: Some(_mesa_RenderMode),
            InitNames: Some(_mesa_InitNames),
            LoadName: Some(_mesa_LoadName),
            PassThrough: Some(_mesa_PassThrough),
            PopName: Some(_mesa_PopName),
            PushName: Some(_mesa_PushName),
            DrawBuffer: Some(_mesa_DrawBuffer),
            Clear: Some(_mesa_Clear),
            ClearAccum: Some(
                _mesa_ClearAccum,
            ),
            ClearIndex: Some(_mesa_ClearIndex),
            ClearColor: Some(
                _mesa_ClearColor,
            ),
            ClearStencil: Some(_mesa_ClearStencil),
            ClearDepth: Some(_mesa_ClearDepth),
            StencilMask: Some(_mesa_StencilMask),
            ColorMask: Some(
                _mesa_ColorMask,
            ),
            DepthMask: Some(_mesa_DepthMask),
            IndexMask: Some(_mesa_IndexMask),
            Accum: Some(_mesa_Accum),
            Disable: Some(_mesa_Disable),
            Enable: Some(_mesa_Enable),
            Finish: Some(_mesa_Finish),
            Flush: Some(_mesa_Flush),
            PopAttrib: Some(_mesa_PopAttrib),
            PushAttrib: Some(_mesa_PushAttrib),
            Map1d: Some(
                _mesa_Map1d,
            ),
            Map1f: Some(
                _mesa_Map1f,
            ),
            Map2d: Some(
                _mesa_Map2d,
            ),
            Map2f: Some(
                _mesa_Map2f,
            ),
            MapGrid1d: Some(
                _mesa_MapGrid1d,
            ),
            MapGrid1f: Some(_mesa_MapGrid1f),
            MapGrid2d: Some(
                _mesa_MapGrid2d,
            ),
            MapGrid2f: Some(
                _mesa_MapGrid2f,
            ),
            EvalCoord1d: Some(_mesa_EvalCoord1d),
            EvalCoord1dv: Some(_mesa_EvalCoord1dv),
            EvalCoord1f: Some(_mesa_EvalCoord1f),
            EvalCoord1fv: Some(_mesa_EvalCoord1fv),
            EvalCoord2d: Some(_mesa_EvalCoord2d),
            EvalCoord2dv: Some(_mesa_EvalCoord2dv),
            EvalCoord2f: Some(_mesa_EvalCoord2f),
            EvalCoord2fv: Some(_mesa_EvalCoord2fv),
            EvalMesh1: Some(_mesa_EvalMesh1),
            EvalPoint1: Some(_mesa_EvalPoint1),
            EvalMesh2: Some(
                _mesa_EvalMesh2,
            ),
            EvalPoint2: Some(_mesa_EvalPoint2),
            AlphaFunc: Some(_mesa_AlphaFunc),
            BlendFunc: Some(_mesa_BlendFunc),
            LogicOp: Some(_mesa_LogicOp),
            StencilFunc: Some(
                _mesa_StencilFunc,
            ),
            StencilOp: Some(_mesa_StencilOp),
            DepthFunc: Some(_mesa_DepthFunc),
            PixelZoom: Some(_mesa_PixelZoom),
            PixelTransferf: Some(
                _mesa_PixelTransferf,
            ),
            PixelTransferi: Some(_mesa_PixelTransferi),
            PixelStoref: Some(_mesa_PixelStoref),
            PixelStorei: Some(_mesa_PixelStorei),
            PixelMapfv: Some(
                _mesa_PixelMapfv,
            ),
            PixelMapuiv: Some(
                _mesa_PixelMapuiv,
            ),
            PixelMapusv: Some(
                _mesa_PixelMapusv,
            ),
            ReadBuffer: Some(_mesa_ReadBuffer),
            CopyPixels: Some(
                _mesa_CopyPixels,
            ),
            ReadPixels: Some(
                _mesa_ReadPixels,
            ),
            DrawPixels: Some(
                _mesa_DrawPixels,
            ),
            GetBooleanv: Some(
                _mesa_GetBooleanv,
            ),
            GetClipPlane: Some(
                _mesa_GetClipPlane,
            ),
            GetDoublev: Some(_mesa_GetDoublev),
            GetError: Some(_mesa_GetError),
            GetFloatv: Some(_mesa_GetFloatv),
            GetIntegerv: Some(_mesa_GetIntegerv),
            GetLightfv: Some(
                _mesa_GetLightfv,
            ),
            GetLightiv: Some(
                _mesa_GetLightiv,
            ),
            GetMapdv: Some(
                _mesa_GetMapdv,
            ),
            GetMapfv: Some(
                _mesa_GetMapfv,
            ),
            GetMapiv: Some(
                _mesa_GetMapiv,
            ),
            GetMaterialfv: Some(
                _mesa_GetMaterialfv,
            ),
            GetMaterialiv: Some(
                _mesa_GetMaterialiv,
            ),
            GetPixelMapfv: Some(
                _mesa_GetPixelMapfv,
            ),
            GetPixelMapuiv: Some(
                _mesa_GetPixelMapuiv,
            ),
            GetPixelMapusv: Some(
                _mesa_GetPixelMapusv,
            ),
            GetPolygonStipple: Some(
                _mesa_GetPolygonStipple,
            ),
            GetString: Some(_mesa_GetString),
            GetTexEnvfv: Some(
                _mesa_GetTexEnvfv,
            ),
            GetTexEnviv: Some(
                _mesa_GetTexEnviv,
            ),
            GetTexGendv: Some(
                _mesa_GetTexGendv,
            ),
            GetTexGenfv: Some(
                _mesa_GetTexGenfv,
            ),
            GetTexGeniv: Some(
                _mesa_GetTexGeniv,
            ),
            GetTexImage: Some(
                _mesa_GetTexImage,
            ),
            GetTexParameterfv: Some(
                _mesa_GetTexParameterfv,
            ),
            GetTexParameteriv: Some(
                _mesa_GetTexParameteriv,
            ),
            GetTexLevelParameterfv: Some(
                _mesa_GetTexLevelParameterfv,
            ),
            GetTexLevelParameteriv: Some(
                _mesa_GetTexLevelParameteriv,
            ),
            IsEnabled: Some(_mesa_IsEnabled),
            IsList: Some(_mesa_IsList),
            DepthRange: Some(_mesa_DepthRange),
            Frustum: Some(
                _mesa_Frustum,
            ),
            LoadIdentity: Some(_mesa_LoadIdentity),
            LoadMatrixf: Some(_mesa_LoadMatrixf),
            LoadMatrixd: Some(_mesa_LoadMatrixd),
            MatrixMode: Some(_mesa_MatrixMode),
            MultMatrixf: Some(_mesa_MultMatrixf),
            MultMatrixd: Some(_mesa_MultMatrixd),
            Ortho: Some(
                _mesa_Ortho,
            ),
            PopMatrix: Some(_mesa_PopMatrix),
            PushMatrix: Some(_mesa_PushMatrix),
            Rotated: Some(
                _mesa_Rotated,
            ),
            Rotatef: Some(
                _mesa_Rotatef,
            ),
            Scaled: Some(_mesa_Scaled),
            Scalef: Some(_mesa_Scalef),
            Translated: Some(
                _mesa_Translated,
            ),
            Translatef: Some(
                _mesa_Translatef,
            ),
            Viewport: Some(
                _mesa_Viewport,
            ),
            ArrayElement: Some(_mesa_ArrayElement),
            BindTexture: Some(_mesa_BindTexture),
            ColorPointer: Some(
                _mesa_ColorPointer,
            ),
            DisableClientState: Some(
                _mesa_DisableClientState,
            ),
            DrawArrays: Some(
                _mesa_DrawArrays,
            ),
            DrawElements: Some(
                _mesa_DrawElements,
            ),
            EdgeFlagPointer: Some(
                _mesa_EdgeFlagPointer,
            ),
            EnableClientState: Some(_mesa_EnableClientState),
            IndexPointer: Some(
                _mesa_IndexPointer,
            ),
            Indexub: Some(_mesa_Indexub),
            Indexubv: Some(_mesa_Indexubv),
            InterleavedArrays: Some(
                _mesa_InterleavedArrays,
            ),
            NormalPointer: Some(
                _mesa_NormalPointer,
            ),
            PolygonOffset: Some(
                _mesa_PolygonOffset,
            ),
            TexCoordPointer: Some(
                _mesa_TexCoordPointer,
            ),
            VertexPointer: Some(
                _mesa_VertexPointer,
            ),
            AreTexturesResident: Some(
                _mesa_AreTexturesResident,
            ),
            CopyTexImage1D: Some(
                _mesa_CopyTexImage1D,
            ),
            CopyTexImage2D: Some(
                _mesa_CopyTexImage2D,
            ),
            CopyTexSubImage1D: Some(
                _mesa_CopyTexSubImage1D,
            ),
            CopyTexSubImage2D: Some(
                _mesa_CopyTexSubImage2D,
            ),
            DeleteTextures: Some(
                _mesa_DeleteTextures,
            ),
            GenTextures: Some(
                _mesa_GenTextures,
            ),
            GetPointerv: Some(
                _mesa_GetPointerv,
            ),
            IsTexture: Some(_mesa_IsTexture),
            PrioritizeTextures: Some(
                _mesa_PrioritizeTextures,
            ),
            TexSubImage1D: Some(
                _mesa_TexSubImage1D,
            ),
            TexSubImage2D: Some(
                _mesa_TexSubImage2D,
            ),
            PopClientAttrib: Some(_mesa_PopClientAttrib),
            PushClientAttrib: Some(
                _mesa_PushClientAttrib,
            ),
        },
    }
};


pub const SB_FLAGS: u32 = PFD_DRAW_TO_BITMAP | PFD_SUPPORT_GDI | PFD_SUPPORT_OPENGL | PFD_GENERIC_FORMAT;
pub const SB_FLAGS_WINDOW: u32 = PFD_DRAW_TO_BITMAP | PFD_SUPPORT_GDI | PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_GENERIC_FORMAT;
pub const SB_FLAGS_PALETTE: u32 = PFD_DRAW_TO_BITMAP | PFD_SUPPORT_GDI | PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_GENERIC_FORMAT | PFD_NEED_PALETTE;
pub const DB_FLAGS: u32 = PFD_DOUBLEBUFFER | PFD_SWAP_COPY | PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_GENERIC_FORMAT;
pub const DB_FLAGS_PALETTE: u32 = PFD_DOUBLEBUFFER | PFD_SWAP_COPY | PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_GENERIC_FORMAT | PFD_NEED_PALETTE;

#[repr(C)]
pub struct pixel_format {
    pub dwFlags: DWORD,
    pub iPixelType: BYTE,
    pub cColorBits: BYTE,
    pub cRedBits: BYTE,
    pub cRedShift: BYTE,
    pub cGreenBits: BYTE,
    pub cGreenShift: BYTE,
    pub cBlueBits: BYTE,
    pub cBlueShift: BYTE,
    pub cAlphaBits: BYTE,
    pub cAlphaShift: BYTE,
    pub cAccumBits: BYTE,
    pub cAccumRedBits: BYTE,
    pub cAccumGreenBits: BYTE,
    pub cAccumBlueBits: BYTE,
    pub cAccumAlphaBits: BYTE,
    pub cDepthBits: BYTE,
}

#[repr(C)]
pub struct sw_framebuffer {
    pub gl_visual: *mut GLvisual,
    pub gl_buffer: *mut GLframebuffer,
    pub pixel_format: *const pixel_format,
    pub Hdc: HDC,
    pub width: GLuint,
    pub height: GLuint,
    pub BackBuffer: *mut BYTE,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct sw_context_u8 {
    pub ClearColor: BYTE,
    pub CurrentColor: BYTE,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct sw_context_u16 {
    pub ClearColor: USHORT,
    pub CurrentColor: USHORT,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct sw_context_u24 {
    pub ClearColor: ULONG,
    pub CurrentColor: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct sw_context_u32 {
    pub ClearColor: ULONG,
    pub CurrentColor: ULONG,
}

// C anonymous union inside struct sw_context; Rust has no anonymous unions,
// so the members are grouped in a named union. Access as sw_ctx.anon_union.u8, etc.
// The members are plain-data Copy structs, so no ManuallyDrop (whose Deref would put a
// reference to a raw-pointer target on every access).
#[repr(C)]
pub union sw_context_union {
    pub u8: sw_context_u8,
    pub u16: sw_context_u16,
    pub u24: sw_context_u24,
    pub u32: sw_context_u32,
}

#[repr(C)]
pub struct sw_context {
    pub gl_ctx: *mut GLcontext,
    pub hook: HHOOK,
    pub fb: *mut sw_framebuffer,
    pub anon_union: sw_context_union,
    pub Mode: GLenum,
}

#[no_mangle]
pub unsafe extern "C" fn sw_DescribePixelFormat(hdc: HDC, format: INT, size: UINT, descr: *mut PIXELFORMATDESCRIPTOR) -> INT {
    let mut ret: INT = 0;
    let mut pixel_format: *const pixel_format = core::ptr::null();

    TRACE!("Describing format %i.\n", format);

    pixel_format = get_format(format, core::ptr::addr_of_mut!(ret));
    if descr.is_null() {
        return ret;
    }
    if (format > ret) || (size as usize != core::mem::size_of::<PIXELFORMATDESCRIPTOR>()) {
        return 0;
    }

    /* Fill the structure */
    (*descr).nSize = core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD;
    (*descr).nVersion = 1 as WORD;
    (*descr).dwFlags = (*pixel_format).dwFlags;
    (*descr).iPixelType = (*pixel_format).iPixelType;
    (*descr).cColorBits = (*pixel_format).cColorBits;
    (*descr).cRedBits = (*pixel_format).cRedBits;
    (*descr).cRedShift = (*pixel_format).cRedShift;
    (*descr).cGreenBits = (*pixel_format).cGreenBits;
    (*descr).cGreenShift = (*pixel_format).cGreenShift;
    (*descr).cBlueBits = (*pixel_format).cBlueBits;
    (*descr).cBlueShift = (*pixel_format).cBlueShift;
    (*descr).cAlphaBits = (*pixel_format).cAlphaBits;
    (*descr).cAlphaShift = (*pixel_format).cAlphaShift;
    (*descr).cAccumBits = (*pixel_format).cAccumBits;
    (*descr).cAccumRedBits = (*pixel_format).cAccumRedBits;
    (*descr).cAccumGreenBits = (*pixel_format).cAccumGreenBits;
    (*descr).cAccumBlueBits = (*pixel_format).cAccumBlueBits;
    (*descr).cAccumAlphaBits = (*pixel_format).cAccumAlphaBits;
    (*descr).cDepthBits = (*pixel_format).cDepthBits;
    (*descr).cStencilBits = STENCIL_BITS as BYTE;
    (*descr).cAuxBuffers = 0 as BYTE;
    (*descr).iLayerType = PFD_MAIN_PLANE as BYTE;
    (*descr).bReserved = 0 as BYTE;
    (*descr).dwLayerMask = 0 as DWORD;
    (*descr).dwVisibleMask = 0 as DWORD;
    (*descr).dwDamageMask = 0 as DWORD;

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn sw_SetPixelFormat(mut hdc: HDC, mut dc_data: *mut wgl_dc_data, mut format: INT) -> BOOL {
    let mut fb: *mut sw_framebuffer = ::core::ptr::null_mut::<sw_framebuffer>();
    let mut pixel_format: *const pixel_format = ::core::ptr::null::<pixel_format>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/opengl/opengl32/swimpl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sw_SetPixelFormat\0" as *const u8 as *const ::core::ffi::c_char,
            365 as ::core::ffi::c_int,
            b"OpenGL software implementation START for hdc %p, format %i!\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            hdc,
            format,
        );
    }

    pixel_format = get_format(format, ::core::ptr::null_mut::<INT>());
    if pixel_format.is_null() {
        return FALSE as BOOL;
    }

    /* allocate our structure */
    fb = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        ::core::mem::size_of::<sw_framebuffer>() as SIZE_T,
    ) as *mut sw_framebuffer;
    if fb.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/opengl/opengl32/swimpl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"sw_SetPixelFormat\0" as *const u8 as *const ::core::ffi::c_char,
                375 as ::core::ffi::c_int,
                b"HeapAlloc FAILED!\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return FALSE as BOOL;
    }
    /* Set the format */
    (*fb).pixel_format = pixel_format;

    (*fb).gl_visual = gl_create_visual(
        ((*pixel_format).iPixelType as INT == (PFD_TYPE_RGBA as i32)) as GLboolean,
        ((*pixel_format).cAlphaBits as INT != 0) as GLboolean,
        (((*pixel_format).dwFlags & PFD_DOUBLEBUFFER) != 0) as GLboolean,
        (*pixel_format).cDepthBits as GLint,
        STENCIL_BITS,
        max(
            max(
                max((*pixel_format).cAccumRedBits as GLint, (*pixel_format).cAccumGreenBits as GLint),
                (*pixel_format).cAccumBlueBits as GLint,
            ),
            (*pixel_format).cAccumAlphaBits as GLint,
        ),
        if (*pixel_format).iPixelType as INT == (PFD_TYPE_COLORINDEX as i32) {
            (*pixel_format).cColorBits as GLint
        } else {
            0 as GLint
        },
        ((((1 as u64) << ((*pixel_format).cRedBits as u32))
            - (1 as u64)) as GLfloat),
        ((((1 as u64) << ((*pixel_format).cGreenBits as u32))
            - (1 as u64)) as GLfloat),
        ((((1 as u64) << ((*pixel_format).cBlueBits as u32))
            - (1 as u64)) as GLfloat),
        if (*pixel_format).cAlphaBits as INT != 0 {
            ((((1 as u64) << ((*pixel_format).cAlphaBits as u32))
                - (1 as u64)) as GLfloat)
        } else {
            (255 as GLfloat)
        },
        (*pixel_format).cRedBits as GLint,
        (*pixel_format).cGreenBits as GLint,
        (*pixel_format).cBlueBits as GLint,
        (*pixel_format).cAlphaBits as GLint,
    );

    if (*fb).gl_visual.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/opengl/opengl32/swimpl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"sw_SetPixelFormat\0" as *const u8 as *const ::core::ffi::c_char,
                400 as ::core::ffi::c_int,
                b"Failed to allocate a GL visual.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            fb as PVOID,
        );
        return FALSE as BOOL;
    }

    /* Allocate the framebuffer structure */
    (*fb).gl_buffer = gl_create_framebuffer((*fb).gl_visual);
    if (*fb).gl_buffer.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/opengl/opengl32/swimpl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"sw_SetPixelFormat\0" as *const u8 as *const ::core::ffi::c_char,
                408 as ::core::ffi::c_int,
                b"Failed to allocate the mesa framebuffer structure.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        gl_destroy_visual((*fb).gl_visual);
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            fb as PVOID,
        );
        return FALSE as BOOL;
    }

    /* Save our DC */
    (*fb).Hdc = hdc;

    /* Everything went fine */
    (*dc_data).sw_data = fb as *mut ::core::ffi::c_void;
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn sw_CreateContext(dc_data: *mut wgl_dc_data) -> DHGLRC {
    let mut sw_ctx: *mut sw_context;
    let fb: *mut sw_framebuffer = (*dc_data).sw_data as *mut sw_framebuffer;

    sw_ctx = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<sw_context>() as SIZE_T,
    ) as *mut sw_context;
    if sw_ctx.is_null() {
        return core::ptr::null_mut();
    }

    /* Initialize the context */
    (*sw_ctx).gl_ctx = gl_create_context(
        (*fb).gl_visual,
        core::ptr::null_mut::<GLcontext>(),
        sw_ctx as *mut core::ffi::c_void,
    );
    if (*sw_ctx).gl_ctx.is_null() {
        ERR!("Failed to initialize the mesa context.\n");
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            sw_ctx as PVOID,
        );
        return core::ptr::null_mut();
    }

    (*sw_ctx).fb = fb;

    /* Choose relevant default */
    (*sw_ctx).Mode = (if (*(*fb).gl_visual).DBflag as core::ffi::c_int != 0 {
        GL_BACK
    } else {
        GL_FRONT
    }) as GLenum;

    return sw_ctx as DHGLRC;
}
#[no_mangle]
pub unsafe extern "C" fn sw_DeleteContext(dhglrc: DHGLRC) -> BOOL {
    let sw_ctx: *mut sw_context = dhglrc as *mut sw_context;
    /* Those get clobbered by _mesa_free_context_data via _glapi_set{context,dispath_table} */
    let icd_save: *mut core::ffi::c_void = IntGetCurrentICDPrivate();
    let table_save: *const GLDISPATCHTABLE = IntGetCurrentDispatchTable();

    /* Destroy everything */
    gl_destroy_context((*sw_ctx).gl_ctx);

    HeapFree(GetProcessHeap(), 0, sw_ctx as PVOID);

    /* Restore this */
    IntSetCurrentDispatchTable(table_save);
    IntSetCurrentICDPrivate(icd_save);

    return (TRUE) as BOOL;
}
unsafe extern "system" fn _swimpl_AddSwapHintRectWIN(
    x: GLint,
    y: GLint,
    width: GLsizei,
    height: GLsizei,
) {
    UNIMPLEMENTED!();
}
#[no_mangle]
pub unsafe extern "C" fn sw_GetProcAddress(name: LPCSTR) -> PROC {
    /* GL_EXT_paletted_texture */
    if strcmp(name as *const ::core::ffi::c_char, b"glColorTableEXT\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLenum, GLenum, GLsizei, GLenum, GLenum, *const ::core::ffi::c_void) -> ()>,
            PROC,
        >(Some(
            _mesa_ColorTableEXT,
        ));
    }
    if strcmp(name as *const ::core::ffi::c_char, b"glColorSubTableEXT\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLenum, GLsizei, GLsizei, GLenum, GLenum, *const ::core::ffi::c_void) -> ()>,
            PROC,
        >(Some(
            _mesa_ColorSubTableEXT,
        ));
    }
    if strcmp(name as *const ::core::ffi::c_char, b"glColorGetTableEXT\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLenum, GLenum, GLenum, *mut ::core::ffi::c_void) -> ()>,
            PROC,
        >(Some(
            _mesa_GetColorTableEXT,
        ));
    }
    if strcmp(name as *const ::core::ffi::c_char, b"glGetColorTableParameterivEXT\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLenum, GLenum, *mut GLfloat) -> ()>,
            PROC,
        >(Some(
            _mesa_GetColorTableParameterivEXT,
        ));
    }
    if strcmp(name as *const ::core::ffi::c_char, b"glGetColorTableParameterfvEXT\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLenum, GLenum, *mut GLint) -> ()>,
            PROC,
        >(Some(
            _mesa_GetColorTableParameterfvEXT,
        ));
    }
    if strcmp(name as *const ::core::ffi::c_char, b"glAddSwapHintRectWIN\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLint, GLint, GLsizei, GLsizei) -> ()>,
            PROC,
        >(Some(
            _swimpl_AddSwapHintRectWIN,
        ));
    }
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/opengl/opengl32/swimpl.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"sw_GetProcAddress\0".as_ptr() as *const ::core::ffi::c_char,
            494 as ::core::ffi::c_int,
            b"Asking for proc address %s, returning NULL.\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            name,
        );
    }
    return None;
}

#[no_mangle]
pub unsafe extern "C" fn sw_CopyContext(
    dhglrcSrc: DHGLRC,
    dhglrcDst: DHGLRC,
    mask: UINT,
) -> BOOL {
    FIXME!("Software wglCopyContext is UNIMPLEMENTED, mask %lx.\n", mask);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn sw_ShareLists(dhglrcSrc: DHGLRC, dhglrcDst: DHGLRC) -> BOOL {
    FIXME!("Unimplemented!\n");
    (TRUE) as i32
}

unsafe extern "system" fn sw_call_window_proc(
   nCode: ::core::ffi::c_int,
   wParam: WPARAM,
   lParam: LPARAM,
) -> LRESULT {
    let dc_data: *mut wgl_dc_data = IntGetCurrentDcData();
    let ctx: *mut sw_context = IntGetCurrentDHGLRC() as *mut sw_context;
    let pParams: PCWPSTRUCT = lParam as PCWPSTRUCT;

    if dc_data.is_null() || ctx.is_null() {
        return 0;
    }

    if ((*dc_data).flags & WGL_DC_OBJ_DC) == 0 {
        return 0;
    }

    if (nCode < 0) || ((*dc_data).owner.hwnd != (*pParams).hwnd) || (*dc_data).sw_data.is_null() {
        return CallNextHookEx((*ctx).hook, nCode, wParam, lParam);
    }

    if (*pParams).message == WM_WINDOWPOSCHANGED {
        /* We handle WM_WINDOWPOSCHANGED instead of WM_SIZE because according to
         * http://blogs.msdn.com/oldnewthing/archive/2008/01/15/7113860.aspx
         * WM_SIZE is generated from WM_WINDOWPOSCHANGED by DefWindowProc so it
         * can be masked out by the application. */
        let lpWindowPos: LPWINDOWPOS = (*pParams).lParam as LPWINDOWPOS;
        if ((*lpWindowPos).flags & SWP_SHOWWINDOW) != 0
            || ((*lpWindowPos).flags & SWP_NOMOVE) == 0
            || ((*lpWindowPos).flags & SWP_NOSIZE) == 0
        {
            /* Size in WINDOWPOS includes the window frame, so get the size
             * of the client area via GetClientRect.  */
            let mut client_rect: RECT = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            let mut width: UINT;
            let mut height: UINT;

            TRACE!("Got WM_WINDOWPOSCHANGED\n");

            GetClientRect((*pParams).hwnd, &raw mut client_rect);
            width = (client_rect.right - client_rect.left) as UINT;
            height = (client_rect.bottom - client_rect.top) as UINT;

            'end: {
                /* Do not reallocate for minimized windows */
                if width <= 0 || height <= 0 {
                    break 'end;
                }
                /* Propagate to mesa */
                gl_ResizeBuffersMESA((*ctx).gl_ctx);
            }
        }
    }

    return CallNextHookEx((*ctx).hook, nCode, wParam, lParam);
}

unsafe extern "C" fn renderer_string() -> *const i8 {
    return b"ReactOS SW Implementation\0".as_ptr() as *const i8;
}

unsafe extern "C" fn PUT_PIXEL_8(Buffer: *mut BYTE, Value: BYTE) {
    *Buffer = Value;
}

unsafe extern "C" fn PUT_PIXEL_16(Buffer: *mut USHORT, Value: USHORT) {
    *Buffer = Value;
}

// A 24bpp pixel sits at `x * 3`, so this ULONG access is UNALIGNED for 3 of every 4 pixels.
// The C dereferences it anyway (fine on x86/arm64 hardware, UB for a Rust `*`), so the
// read-modify-write goes through read_unaligned/write_unaligned. Same bytes touched as the C.
unsafe extern "C" fn PUT_PIXEL_24(Buffer: *mut ULONG, Value: ULONG) {
    let mut v: ULONG = core::ptr::read_unaligned(Buffer);
    v &= 0xFF000000;
    v |= Value & 0x00FFFFFF;
    core::ptr::write_unaligned(Buffer, v);
}

unsafe extern "C" fn PUT_PIXEL_32(Buffer: *mut ULONG, Value: ULONG) {
    *Buffer = Value;
}

unsafe extern "C" fn GET_PIXEL_8(Buffer: *mut BYTE) -> BYTE {
    return *Buffer;
}

unsafe extern "C" fn GET_PIXEL_16(Buffer: *mut USHORT) -> USHORT {
    return *Buffer;
}

// Unaligned like PUT_PIXEL_24.
unsafe extern "C" fn GET_PIXEL_24(Buffer: *mut ULONG) -> ULONG {
    return core::ptr::read_unaligned(Buffer) & 0x00FFFFFF;
}

unsafe extern "C" fn GET_PIXEL_32(Buffer: *mut ULONG) -> ULONG {
    return *Buffer;
}

unsafe extern "C" fn PACK_COLOR_8(mut r: GLubyte, mut g: GLubyte, mut b: GLubyte) -> BYTE {
    return ((r as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int)
        | ((g as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int) << 3)
        | ((b as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int) << 6))
        as BYTE;
}

unsafe extern "C" fn PACK_COLOR_16(mut r: GLubyte, mut g: GLubyte, mut b: GLubyte) -> USHORT {
    return ((((r as ::core::ffi::c_int) & 0x1F) << 11)
        | (((g as ::core::ffi::c_int) & 0x3F) << 5)
        | ((b as ::core::ffi::c_int) & 0x1F)) as USHORT;
}

unsafe extern "C" fn PACK_COLOR_24(mut r: GLubyte, mut g: GLubyte, mut b: GLubyte) -> ULONG {
    return ((r as ULONG) << 16) | ((g as ULONG) << 8) | (b as ULONG);
}

unsafe extern "C" fn PACK_COLOR_32(r: GLubyte, g: GLubyte, b: GLubyte) -> ULONG {
    return ((r as ULONG) << 16) | ((g as ULONG) << 8) | (b as ULONG);
}

unsafe extern "C" fn PACK_COLORREF_8(r: GLubyte, g: GLubyte, b: GLubyte) -> COLORREF {
    return RGB!((r as i32) << 5, (g as i32) << 5, (b as i32) << 6);
}

unsafe extern "C" fn PACK_COLORREF_16(r: GLubyte, g: GLubyte, b: GLubyte) -> COLORREF {
    return RGB!(
        ((r as ::core::ffi::c_int) << 3),
        ((g as ::core::ffi::c_int) << 2),
        ((b as ::core::ffi::c_int) << 3),
    );
}

unsafe extern "C" fn PACK_COLORREF_24(r: GLubyte, g: GLubyte, b: GLubyte) -> COLORREF {
    RGB!(r, g, b)
}

unsafe extern "C" fn PACK_COLORREF_32(r: GLubyte, g: GLubyte, b: GLubyte) -> COLORREF {
    RGB!(r, g, b)
}

unsafe extern "C" fn UNPACK_COLOR_8(Color: BYTE, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = Color & 0x7;
    *g = (Color >> 3) & 0x7;
    *b = (Color >> 6) & 0x3;
}

unsafe extern "C" fn UNPACK_COLOR_16(Color: USHORT, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = (((Color as ::core::ffi::c_int) >> 11) & 0x1F) as GLubyte;
    *g = (((Color as ::core::ffi::c_int) >> 5) & 0x3F) as GLubyte;
    *b = ((Color as ::core::ffi::c_int) & 0x1F) as GLubyte;
}

unsafe extern "C" fn UNPACK_COLOR_24(Color: ULONG, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = ((Color >> 16) & 0xFF) as GLubyte;
    *g = ((Color >> 8) & 0xFF) as GLubyte;
    *b = (Color & 0xFF) as GLubyte;
}

unsafe extern "C" fn UNPACK_COLOR_32(Color: ULONG, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = ((Color >> 16) & 0xFF) as GLubyte;
    *g = ((Color >> 8) & 0xFF) as GLubyte;
    *b = (Color & 0xFF) as GLubyte;
}

unsafe extern "C" fn UNPACK_COLORREF_8(Color: COLORREF, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = (GetRValue!(Color) >> 5) as GLubyte;
    *g = (GetGValue!(Color) >> 5) as GLubyte;
    *b = (GetBValue!(Color) >> 6) as GLubyte;
}

unsafe extern "C" fn UNPACK_COLORREF_16(Color: COLORREF, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = ((GetRValue!(Color) as ::core::ffi::c_int) >> 3) as GLubyte;
    *g = ((GetGValue!(Color) as ::core::ffi::c_int) >> 2) as GLubyte;
    *b = ((GetBValue!(Color) as ::core::ffi::c_int) >> 3) as GLubyte;
}

unsafe extern "C" fn UNPACK_COLORREF_24(Color: COLORREF, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = GetRValue!(Color);
    *g = GetGValue!(Color);
    *b = GetBValue!(Color);
}

unsafe extern "C" fn UNPACK_COLORREF_32(Color: COLORREF, r: *mut GLubyte, g: *mut GLubyte, b: *mut GLubyte) {
    *r = GetRValue!(Color);
    *g = GetGValue!(Color);
    *b = GetBValue!(Color);
}

unsafe extern "C" fn clear_frontbuffer(
    sw_ctx: *mut sw_context,
    fb: *mut sw_framebuffer,
    x: GLint,
    y: GLint,
    width: GLint,
    height: GLint,
    ClearColor: COLORREF,
) {
    let mut Brush: HBRUSH = core::ptr::null_mut::<HBRUSH__>();
    let mut ret: BOOL = 0;

    TRACE!("Clearing front buffer (%u, %u, %u, %u), color 0x%08x.\n", x, y, width, height, ClearColor);

    Brush = CreateSolidBrush(ClearColor) as HBRUSH;
    Brush = SelectObject((*fb).Hdc, Brush as HGDIOBJ) as HBRUSH;

    ret = PatBlt(
        (*fb).Hdc,
        x as i32,
        (*fb).height.wrapping_sub((y + height) as GLuint) as i32,
        width as i32,
        height as i32,
        PATCOPY,
    );
    if ret == 0 {
        ERR!("PatBlt failed. last Error %d.\n", GetLastError());
    }

    Brush = SelectObject((*fb).Hdc, Brush as HGDIOBJ) as HBRUSH;
    DeleteObject(Brush as HGDIOBJ);
}

unsafe extern "C" fn set_buffer(mut ctx: *mut GLcontext, mut mode: GLenum) -> GLboolean {
    let mut sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let mut fb: *mut sw_framebuffer = (*sw_ctx).fb;
    if (*(*fb).gl_visual).DBflag == 0 {
        return (GL_FALSE) as GLboolean;
    }
    if mode != GL_FRONT && mode != GL_BACK {
        return (GL_FALSE) as GLboolean;
    }
    (*sw_ctx).Mode = mode;
    return (GL_TRUE) as GLboolean;
}

unsafe extern "C" fn buffer_size(ctx: *mut GLcontext, width: *mut GLuint, height: *mut GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let Window: HWND = WindowFromDC((*fb).Hdc);

    if !Window.is_null() {
        let mut client_rect: RECT = core::mem::zeroed();
        GetClientRect(Window, core::ptr::addr_of_mut!(client_rect));
        *width = (client_rect.right - client_rect.left) as GLuint;
        *height = (client_rect.bottom - client_rect.top) as GLuint;
    } else {
        /* We are drawing to a bitmap */
        let mut bm: BITMAP = core::mem::zeroed();
        let Hbm: HBITMAP;

        Hbm = GetCurrentObject((*fb).Hdc, OBJ_BITMAP as UINT) as HBITMAP;

        if GetObjectW(
            Hbm as HANDLE,
            core::mem::size_of::<BITMAP>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        ) == 0
        {
            return;
        }

        TRACE!("Framebuffer size : %i, %i\n", bm.bmWidth, bm.bmHeight);

        *width = bm.bmWidth as GLuint;
        *height = bm.bmHeight as GLuint;
    }

    if (*width != (*fb).width) || (*height != (*fb).height) {
        let pixel_format: *const pixel_format = (*fb).pixel_format;

        if ((*pixel_format).dwFlags & PFD_DOUBLEBUFFER) != 0 {
            /* Allocate a new backbuffer */
            let BufferSize: usize = (*height as usize)
                * (WIDTH_BYTES_ALIGN32!(*width, (*pixel_format).cColorBits as GLuint) as usize);
            if (*fb).BackBuffer.is_null() {
                (*fb).BackBuffer = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (BufferSize) as SIZE_T,
                ) as *mut BYTE;
            } else {
                (*fb).BackBuffer = HeapReAlloc(
                    GetProcessHeap(),
                    0,
                    (*fb).BackBuffer as PVOID,
                    (BufferSize) as SIZE_T,
                ) as *mut BYTE;
            }
            if (*fb).BackBuffer.is_null() {
                ERR!("Failed allocating back buffer !.\n");
                return;
            }
        }

        (*fb).width = *width;
        (*fb).height = *height;
    }
}

// DIVERGENCE from the C: its masked branch calls `SetPixel(fb->Hdc, x + n, y, Color)` with GL's
// bottom-up `y`, while the unmasked branch of this same function (PatBlt at `fb->height - y`)
// and every other front-buffer writer/reader flip it. So a masked monocolor span lands
// mirrored vertically. The masked branch flips here like the rest.
unsafe extern "C" fn write_monocolor_span_frontbuffer(fb: *mut sw_framebuffer, mut n: GLuint, x: GLint, y: GLint, mask: *const GLubyte, Color: COLORREF) {
    TRACE!("Writing monocolor span at %u %u (%u), Color 0x%08x\n", x, y, n, Color);

    if !mask.is_null() {
        loop {
            let fresh27 = n;
            n = n.wrapping_sub(1);
            if !(fresh27 != 0) {
                break;
            }
            if *mask.offset(n as isize) != 0 {
                SetPixel(
                    (*fb).Hdc,
                    (x as GLuint).wrapping_add(n) as GLint,
                    (*fb).height.wrapping_sub(y as GLuint) as GLint,
                    Color,
                );
            }
        }
    } else {
        let mut Brush: HBRUSH = CreateSolidBrush(Color);
        Brush = SelectObject((*fb).Hdc, Brush as _) as HBRUSH;

        PatBlt(
            (*fb).Hdc,
            x,
            (*fb).height.wrapping_sub(y as GLuint) as GLint,
            n as GLint,
            1,
            PATCOPY,
        );

        Brush = SelectObject((*fb).Hdc, Brush as _) as HBRUSH;
        DeleteObject(Brush as _);
    }
}

unsafe extern "C" fn write_monocolor_pixels_frontbuffer(
    fb: *mut sw_framebuffer,
    mut n: GLuint,
    x: *const GLint,
    y: *const GLint,
    mask: *const GLubyte,
    Color: COLORREF,
) {
    TRACE!("Writing monocolor pixels to front buffer.\n");

    loop {
        let fresh22 = n;
        n = n.wrapping_sub(1);
        if !(fresh22 != 0) {
            break;
        }
        if *mask.offset(n as isize) != 0 {
            SetPixel(
                (*fb).Hdc,
                *x.offset(n as isize) as i32,
                (*fb).height.wrapping_sub(*y.offset(n as isize) as GLuint) as i32,
                Color,
            );
        }
    }
}

unsafe extern "C" fn write_index_span(
    mut ctx: *mut GLcontext,
    mut n: GLuint,
    mut x: GLint,
    mut y: GLint,
    mut index: *const GLuint,
    mut mask: *const GLubyte,
) {
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_ERR as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/opengl/opengl32/swimpl.c\0"
                as *const u8 as *const i8,
            b"write_index_span\0" as *const u8 as *const i8,
            1197i32,
            b"Not implemented yet !\n\0" as *const u8 as *const i8,
        );
    }
}

unsafe extern "C" fn write_index_pixels(
    mut ctx: *mut GLcontext,
    mut n: GLuint,
    mut x: *const GLint,
    mut y: *const GLint,
    mut index: *const GLuint,
    mut mask: *const GLubyte,
) {
    if (*(*core::ptr::addr_of!(__wine_dbch___default))).flags as i32
        & (1i32) << __WINE_DBCL_ERR
        != 0
    {
        let __dbch: *mut __wine_debug_channel = *core::ptr::addr_of_mut!(__wine_dbch___default);
        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR as __wine_debug_class;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/opengl/opengl32/swimpl.c\0"
                .as_ptr() as *const i8,
            b"write_index_pixels\0".as_ptr() as *const i8,
            1205,
            b"Not implemented yet !\n\0".as_ptr() as *const i8,
        );
    }
}

unsafe extern "C" fn read_index_span(
    mut ctx: *mut GLcontext,
    mut n: GLuint,
    mut x: GLint,
    mut y: GLint,
    mut index: *mut GLuint,
) {
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/opengl/opengl32/swimpl.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"read_index_span\0" as *const u8 as *const core::ffi::c_char,
            1211 as core::ffi::c_int,
            b"Not implemented yet !\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }
}

unsafe extern "C" fn read_index_pixels(
    mut ctx: *mut GLcontext,
    mut n: GLuint,
    mut x: *const GLint,
    mut y: *const GLint,
    mut index: *mut GLuint,
    mut mask: *const GLubyte,
) {
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_ERR as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/opengl/opengl32/swimpl.c\0"
                as *const u8 as *const i8,
            b"read_index_pixels\0" as *const u8 as *const i8,
            1260 as i32,
            b"Not implemented yet !\n\0" as *const u8 as *const i8,
        );
    }
}

// ---- swimpl.c:715-1306 per-bit-depth span/pixel functions ----
// The C stamps each of these out once per BPP with a token-pasting macro (MAKE_COLORREF(__bpp,
// __type), CLEAR(__bpp, __type, __pixel_size), WRITE_COLOR_SPAN(...), ...) instantiated for
// (8, BYTE, 1), (16, USHORT, 2), (24, ULONG, 3), (32, ULONG, 4). Each instantiation is written
// out explicitly below, with that depth's PUT_PIXEL_/GET_PIXEL_/PACK_COLOR_/UNPACK_COLOR_ helpers.
// Pointer arithmetic follows C's integer types: `y * WIDTH_BYTES_ALIGN32(...)` and `(x + n) *
// pixel_size` are UNSIGNED (zero-extended when added to the pointer), `x[n] * pixel_size` is
// signed int.

// MAKE_COLORREF(8, BYTE)
#[inline]
unsafe fn MAKE_COLORREF_8(format: *const pixel_format, Color: BYTE) -> COLORREF {
    let mut r: GLubyte = 0;
    let mut g: GLubyte = 0;
    let mut b: GLubyte = 0;

    if (*format).iPixelType as u32 == PFD_TYPE_COLORINDEX {
        return PALETTEINDEX!(Color);
    }

    UNPACK_COLOR_8(Color, &mut r, &mut g, &mut b);

    return PACK_COLORREF_8(r, g, b);
}

// CLEAR_INDEX(8, BYTE): set the color index used to clear the color buffer.
unsafe extern "C" fn clear_index_8(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u8.ClearColor = index as BYTE;
}

// CLEAR_COLOR(8): set the color used to clear the color buffer.
unsafe extern "C" fn clear_color_8(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u8.ClearColor = PACK_COLOR_8(r, g, b);

    TRACE!("Set Clear color %u, %u, %u.\n", r as c_uint, g as c_uint, b as c_uint);
}

// CLEAR(8, BYTE, 1): clear the specified region of the color buffer using the clear color
// or index set by the two functions above.
unsafe extern "C" fn clear_8(ctx: *mut GLcontext, all: GLboolean, mut x: GLint, mut y: GLint, mut width: GLint, mut height: GLint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut ScanLine: *mut BYTE;

    if all != 0 {
        y = 0;
        x = y;
        width = (*fb).width as GLint;
        height = (*fb).height as GLint;
    }

    if (*sw_ctx).Mode == GL_FRONT {
        clear_frontbuffer(sw_ctx, fb, x, y, width, height,
                MAKE_COLORREF_8((*fb).pixel_format, (*sw_ctx).anon_union.u8.ClearColor));
        return;
    }

    ScanLine = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint)) as usize);
    loop {
        let h = height;
        height = height.wrapping_sub(1);
        if h == 0 {
            break;
        }
        let mut Buffer: *mut BYTE = ScanLine.wrapping_offset(x.wrapping_mul(1) as isize);
        let mut n: UINT = width as UINT;

        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            PUT_PIXEL_8(Buffer as *mut BYTE, (*sw_ctx).anon_union.u8.ClearColor);
            Buffer = Buffer.wrapping_add(1);
        }

        ScanLine = ScanLine.wrapping_add(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint) as usize);
    }
}

// SET_INDEX(8): set the current color index.
unsafe extern "C" fn set_index_8(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u8.CurrentColor = index as BYTE;
}

// SET_COLOR(8): set the current RGBA color.
unsafe extern "C" fn set_color_8(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u8.CurrentColor = PACK_COLOR_8(r, g, b);
}

// WRITE_COLOR_SPAN_FRONTBUFFER(8): write a horizontal span of color pixels with a boolean mask.
unsafe fn write_color_span_frontbuffer_8(fb: *mut sw_framebuffer,
        mut n: GLuint, x: GLint, y: GLint,
        red: *const GLubyte, green: *const GLubyte,
        blue: *const GLubyte, mask: *const GLubyte) {
    TRACE!("Writing color span at %u, %u (%u)\n", x, y, n);

    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                        PACK_COLORREF_8(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                    PACK_COLORREF_8(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_COLOR_SPAN(8, BYTE, 1)
unsafe extern "C" fn write_color_span_8(ctx: *mut GLcontext,
                                     mut n: GLuint, x: GLint, y: GLint,
                                     red: *const GLubyte, green: *const GLubyte,
                                     blue: *const GLubyte, alpha: *const GLubyte,
                                     mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_color_span_frontbuffer_8(fb, n, x, y, red, green, blue, mask);
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(1) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(1);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_8(Buffer as *mut BYTE,
                        PACK_COLOR_8(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(1);
            PUT_PIXEL_8(Buffer as *mut BYTE,
                    PACK_COLOR_8(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_SPAN(8, BYTE, 1)
unsafe extern "C" fn write_monocolor_span_8(ctx: *mut GLcontext,
                                 mut n: GLuint, x: GLint, y: GLint,
                                 mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_span_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_8((*fb).pixel_format, (*sw_ctx).anon_union.u8.CurrentColor));
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(1) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(1);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_8(Buffer as *mut BYTE, (*sw_ctx).anon_union.u8.CurrentColor);
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(1);
            PUT_PIXEL_8(Buffer as *mut BYTE, (*sw_ctx).anon_union.u8.CurrentColor);
        }
    }
}

// WRITE_COLOR_PIXELS(8, BYTE, 1): write an array of pixels with a boolean mask.
unsafe extern "C" fn write_color_pixels_8(ctx: *mut GLcontext,
                               mut n: GLuint, x: *const GLint, y: *const GLint,
                               r: *const GLubyte, g: *const GLubyte,
                               b: *const GLubyte, a: *const GLubyte,
                               mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    TRACE!("Writing color pixels\n");

    if (*sw_ctx).Mode == GL_FRONT {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                TRACE!("Setting pixel %u, %u to 0x%08x.\n", *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint),
                        PACK_COLORREF_8(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
                SetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int,
                        PACK_COLORREF_8(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(1) as isize);
            PUT_PIXEL_8(Buffer as *mut BYTE, PACK_COLOR_8(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_PIXELS(8, BYTE, 1): write an array of pixels with a boolean mask. The
// current color is used for all pixels.
// DIVERGENCE from the C: the C macro computes the row stride as
// `WIDTH_BYTES_ALIGN32(fb->width, 32)` for EVERY depth while stepping x by `__pixel_size`;
// every sibling (WRITE_COLOR_PIXELS, READ_COLOR_PIXELS, the span functions, CLEAR) and
// sw_SwapBuffers' biSizeImage use `__bpp`, and buffer_size allocates `height *
// WIDTH_BYTES_ALIGN32(width, cColorBits)`. So below 32bpp the C writes to the wrong row and,
// for large y, past the end of the back buffer. The stride here uses this depth's bpp.
unsafe extern "C" fn write_monocolor_pixels_8(ctx: *mut GLcontext, mut n: GLuint,
                                   x: *const GLint, y: *const GLint,
                                   mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_pixels_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_8((*fb).pixel_format, (*sw_ctx).anon_union.u8.CurrentColor));

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(1) as isize);
            PUT_PIXEL_8(Buffer as *mut BYTE, (*sw_ctx).anon_union.u8.CurrentColor);
        }
    }
}

// READ_COLOR_SPAN(8, BYTE, 1): read a horizontal span of color pixels.
unsafe extern "C" fn read_color_span_8(ctx: *mut GLcontext,
                            mut n: GLuint, x: GLint, y: GLint,
                            red: *mut GLubyte, green: *mut GLubyte,
                            blue: *mut GLubyte, alpha: *mut GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Color = GetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int);
            UNPACK_COLORREF_8(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }

        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(1) as usize);
    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        Buffer = Buffer.wrapping_sub(1);
        UNPACK_COLOR_8(GET_PIXEL_8(Buffer as *mut BYTE),
                red.add(n as usize), green.add(n as usize), blue.add(n as usize));
        *alpha.add(n as usize) = 0;
    }
}

// READ_COLOR_PIXELS(8, BYTE, 1): read an array of color pixels.
unsafe extern "C" fn read_color_pixels_8(ctx: *mut GLcontext,
                              mut n: GLuint, x: *const GLint, y: *const GLint,
                              red: *mut GLubyte, green: *mut GLubyte,
                              blue: *mut GLubyte, alpha: *mut GLubyte,
                              mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                Color = GetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int);
                UNPACK_COLORREF_8(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
                *alpha.add(n as usize) = 0;
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 8 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(1) as isize);
            UNPACK_COLOR_8(GET_PIXEL_8(Buffer as *mut BYTE),
                    red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }
    }
}

// MAKE_COLORREF(16, USHORT)
#[inline]
unsafe fn MAKE_COLORREF_16(format: *const pixel_format, Color: USHORT) -> COLORREF {
    let mut r: GLubyte = 0;
    let mut g: GLubyte = 0;
    let mut b: GLubyte = 0;

    if (*format).iPixelType as u32 == PFD_TYPE_COLORINDEX {
        return PALETTEINDEX!(Color);
    }

    UNPACK_COLOR_16(Color, &mut r, &mut g, &mut b);

    return PACK_COLORREF_16(r, g, b);
}

// CLEAR_INDEX(16, USHORT): set the color index used to clear the color buffer.
unsafe extern "C" fn clear_index_16(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u16.ClearColor = index as USHORT;
}

// CLEAR_COLOR(16): set the color used to clear the color buffer.
unsafe extern "C" fn clear_color_16(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u16.ClearColor = PACK_COLOR_16(r, g, b);

    TRACE!("Set Clear color %u, %u, %u.\n", r as c_uint, g as c_uint, b as c_uint);
}

// CLEAR(16, USHORT, 2): clear the specified region of the color buffer using the clear color
// or index set by the two functions above.
unsafe extern "C" fn clear_16(ctx: *mut GLcontext, all: GLboolean, mut x: GLint, mut y: GLint, mut width: GLint, mut height: GLint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut ScanLine: *mut BYTE;

    if all != 0 {
        y = 0;
        x = y;
        width = (*fb).width as GLint;
        height = (*fb).height as GLint;
    }

    if (*sw_ctx).Mode == GL_FRONT {
        clear_frontbuffer(sw_ctx, fb, x, y, width, height,
                MAKE_COLORREF_16((*fb).pixel_format, (*sw_ctx).anon_union.u16.ClearColor));
        return;
    }

    ScanLine = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint)) as usize);
    loop {
        let h = height;
        height = height.wrapping_sub(1);
        if h == 0 {
            break;
        }
        let mut Buffer: *mut BYTE = ScanLine.wrapping_offset(x.wrapping_mul(2) as isize);
        let mut n: UINT = width as UINT;

        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            PUT_PIXEL_16(Buffer as *mut USHORT, (*sw_ctx).anon_union.u16.ClearColor);
            Buffer = Buffer.wrapping_add(2);
        }

        ScanLine = ScanLine.wrapping_add(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint) as usize);
    }
}

// SET_INDEX(16): set the current color index.
unsafe extern "C" fn set_index_16(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u16.CurrentColor = index as USHORT;
}

// SET_COLOR(16): set the current RGBA color.
unsafe extern "C" fn set_color_16(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u16.CurrentColor = PACK_COLOR_16(r, g, b);
}

// WRITE_COLOR_SPAN_FRONTBUFFER(16): write a horizontal span of color pixels with a boolean mask.
unsafe fn write_color_span_frontbuffer_16(fb: *mut sw_framebuffer,
        mut n: GLuint, x: GLint, y: GLint,
        red: *const GLubyte, green: *const GLubyte,
        blue: *const GLubyte, mask: *const GLubyte) {
    TRACE!("Writing color span at %u, %u (%u)\n", x, y, n);

    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                        PACK_COLORREF_16(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                    PACK_COLORREF_16(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_COLOR_SPAN(16, USHORT, 2)
unsafe extern "C" fn write_color_span_16(ctx: *mut GLcontext,
                                     mut n: GLuint, x: GLint, y: GLint,
                                     red: *const GLubyte, green: *const GLubyte,
                                     blue: *const GLubyte, alpha: *const GLubyte,
                                     mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_color_span_frontbuffer_16(fb, n, x, y, red, green, blue, mask);
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(2) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(2);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_16(Buffer as *mut USHORT,
                        PACK_COLOR_16(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(2);
            PUT_PIXEL_16(Buffer as *mut USHORT,
                    PACK_COLOR_16(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_SPAN(16, USHORT, 2)
unsafe extern "C" fn write_monocolor_span_16(ctx: *mut GLcontext,
                                 mut n: GLuint, x: GLint, y: GLint,
                                 mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_span_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_16((*fb).pixel_format, (*sw_ctx).anon_union.u16.CurrentColor));
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(2) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(2);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_16(Buffer as *mut USHORT, (*sw_ctx).anon_union.u16.CurrentColor);
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(2);
            PUT_PIXEL_16(Buffer as *mut USHORT, (*sw_ctx).anon_union.u16.CurrentColor);
        }
    }
}

// WRITE_COLOR_PIXELS(16, USHORT, 2): write an array of pixels with a boolean mask.
unsafe extern "C" fn write_color_pixels_16(ctx: *mut GLcontext,
                               mut n: GLuint, x: *const GLint, y: *const GLint,
                               r: *const GLubyte, g: *const GLubyte,
                               b: *const GLubyte, a: *const GLubyte,
                               mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    TRACE!("Writing color pixels\n");

    if (*sw_ctx).Mode == GL_FRONT {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                TRACE!("Setting pixel %u, %u to 0x%08x.\n", *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint),
                        PACK_COLORREF_16(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
                SetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int,
                        PACK_COLORREF_16(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(2) as isize);
            PUT_PIXEL_16(Buffer as *mut USHORT, PACK_COLOR_16(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_PIXELS(16, USHORT, 2): write an array of pixels with a boolean mask. The
// current color is used for all pixels.
// DIVERGENCE from the C: the C macro computes the row stride as
// `WIDTH_BYTES_ALIGN32(fb->width, 32)` for EVERY depth while stepping x by `__pixel_size`;
// every sibling (WRITE_COLOR_PIXELS, READ_COLOR_PIXELS, the span functions, CLEAR) and
// sw_SwapBuffers' biSizeImage use `__bpp`, and buffer_size allocates `height *
// WIDTH_BYTES_ALIGN32(width, cColorBits)`. So below 32bpp the C writes to the wrong row and,
// for large y, past the end of the back buffer. The stride here uses this depth's bpp.
unsafe extern "C" fn write_monocolor_pixels_16(ctx: *mut GLcontext, mut n: GLuint,
                                   x: *const GLint, y: *const GLint,
                                   mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_pixels_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_16((*fb).pixel_format, (*sw_ctx).anon_union.u16.CurrentColor));

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(2) as isize);
            PUT_PIXEL_16(Buffer as *mut USHORT, (*sw_ctx).anon_union.u16.CurrentColor);
        }
    }
}

// READ_COLOR_SPAN(16, USHORT, 2): read a horizontal span of color pixels.
unsafe extern "C" fn read_color_span_16(ctx: *mut GLcontext,
                            mut n: GLuint, x: GLint, y: GLint,
                            red: *mut GLubyte, green: *mut GLubyte,
                            blue: *mut GLubyte, alpha: *mut GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Color = GetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int);
            UNPACK_COLORREF_16(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }

        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(2) as usize);
    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        Buffer = Buffer.wrapping_sub(2);
        UNPACK_COLOR_16(GET_PIXEL_16(Buffer as *mut USHORT),
                red.add(n as usize), green.add(n as usize), blue.add(n as usize));
        *alpha.add(n as usize) = 0;
    }
}

// READ_COLOR_PIXELS(16, USHORT, 2): read an array of color pixels.
unsafe extern "C" fn read_color_pixels_16(ctx: *mut GLcontext,
                              mut n: GLuint, x: *const GLint, y: *const GLint,
                              red: *mut GLubyte, green: *mut GLubyte,
                              blue: *mut GLubyte, alpha: *mut GLubyte,
                              mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                Color = GetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int);
                UNPACK_COLORREF_16(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
                *alpha.add(n as usize) = 0;
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 16 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(2) as isize);
            UNPACK_COLOR_16(GET_PIXEL_16(Buffer as *mut USHORT),
                    red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }
    }
}

// MAKE_COLORREF(24, ULONG)
#[inline]
unsafe fn MAKE_COLORREF_24(format: *const pixel_format, Color: ULONG) -> COLORREF {
    let mut r: GLubyte = 0;
    let mut g: GLubyte = 0;
    let mut b: GLubyte = 0;

    if (*format).iPixelType as u32 == PFD_TYPE_COLORINDEX {
        return PALETTEINDEX!(Color);
    }

    UNPACK_COLOR_24(Color, &mut r, &mut g, &mut b);

    return PACK_COLORREF_24(r, g, b);
}

// CLEAR_INDEX(24, ULONG): set the color index used to clear the color buffer.
unsafe extern "C" fn clear_index_24(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u24.ClearColor = index as ULONG;
}

// CLEAR_COLOR(24): set the color used to clear the color buffer.
unsafe extern "C" fn clear_color_24(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u24.ClearColor = PACK_COLOR_24(r, g, b);

    TRACE!("Set Clear color %u, %u, %u.\n", r as c_uint, g as c_uint, b as c_uint);
}

// CLEAR(24, ULONG, 3): clear the specified region of the color buffer using the clear color
// or index set by the two functions above.
unsafe extern "C" fn clear_24(ctx: *mut GLcontext, all: GLboolean, mut x: GLint, mut y: GLint, mut width: GLint, mut height: GLint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut ScanLine: *mut BYTE;

    if all != 0 {
        y = 0;
        x = y;
        width = (*fb).width as GLint;
        height = (*fb).height as GLint;
    }

    if (*sw_ctx).Mode == GL_FRONT {
        clear_frontbuffer(sw_ctx, fb, x, y, width, height,
                MAKE_COLORREF_24((*fb).pixel_format, (*sw_ctx).anon_union.u24.ClearColor));
        return;
    }

    ScanLine = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint)) as usize);
    loop {
        let h = height;
        height = height.wrapping_sub(1);
        if h == 0 {
            break;
        }
        let mut Buffer: *mut BYTE = ScanLine.wrapping_offset(x.wrapping_mul(3) as isize);
        let mut n: UINT = width as UINT;

        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            PUT_PIXEL_24(Buffer as *mut ULONG, (*sw_ctx).anon_union.u24.ClearColor);
            Buffer = Buffer.wrapping_add(3);
        }

        ScanLine = ScanLine.wrapping_add(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint) as usize);
    }
}

// SET_INDEX(24): set the current color index.
unsafe extern "C" fn set_index_24(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u24.CurrentColor = index as ULONG;
}

// SET_COLOR(24): set the current RGBA color.
unsafe extern "C" fn set_color_24(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u24.CurrentColor = PACK_COLOR_24(r, g, b);
}

// WRITE_COLOR_SPAN_FRONTBUFFER(24): write a horizontal span of color pixels with a boolean mask.
unsafe fn write_color_span_frontbuffer_24(fb: *mut sw_framebuffer,
        mut n: GLuint, x: GLint, y: GLint,
        red: *const GLubyte, green: *const GLubyte,
        blue: *const GLubyte, mask: *const GLubyte) {
    TRACE!("Writing color span at %u, %u (%u)\n", x, y, n);

    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                        PACK_COLORREF_24(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                    PACK_COLORREF_24(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_COLOR_SPAN(24, ULONG, 3)
unsafe extern "C" fn write_color_span_24(ctx: *mut GLcontext,
                                     mut n: GLuint, x: GLint, y: GLint,
                                     red: *const GLubyte, green: *const GLubyte,
                                     blue: *const GLubyte, alpha: *const GLubyte,
                                     mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_color_span_frontbuffer_24(fb, n, x, y, red, green, blue, mask);
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(3) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(3);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_24(Buffer as *mut ULONG,
                        PACK_COLOR_24(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(3);
            PUT_PIXEL_24(Buffer as *mut ULONG,
                    PACK_COLOR_24(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_SPAN(24, ULONG, 3)
unsafe extern "C" fn write_monocolor_span_24(ctx: *mut GLcontext,
                                 mut n: GLuint, x: GLint, y: GLint,
                                 mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_span_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_24((*fb).pixel_format, (*sw_ctx).anon_union.u24.CurrentColor));
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(3) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(3);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_24(Buffer as *mut ULONG, (*sw_ctx).anon_union.u24.CurrentColor);
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(3);
            PUT_PIXEL_24(Buffer as *mut ULONG, (*sw_ctx).anon_union.u24.CurrentColor);
        }
    }
}

// WRITE_COLOR_PIXELS(24, ULONG, 3): write an array of pixels with a boolean mask.
unsafe extern "C" fn write_color_pixels_24(ctx: *mut GLcontext,
                               mut n: GLuint, x: *const GLint, y: *const GLint,
                               r: *const GLubyte, g: *const GLubyte,
                               b: *const GLubyte, a: *const GLubyte,
                               mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    TRACE!("Writing color pixels\n");

    if (*sw_ctx).Mode == GL_FRONT {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                TRACE!("Setting pixel %u, %u to 0x%08x.\n", *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint),
                        PACK_COLORREF_24(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
                SetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int,
                        PACK_COLORREF_24(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(3) as isize);
            PUT_PIXEL_24(Buffer as *mut ULONG, PACK_COLOR_24(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_PIXELS(24, ULONG, 3): write an array of pixels with a boolean mask. The
// current color is used for all pixels.
// DIVERGENCE from the C: the C macro computes the row stride as
// `WIDTH_BYTES_ALIGN32(fb->width, 32)` for EVERY depth while stepping x by `__pixel_size`;
// every sibling (WRITE_COLOR_PIXELS, READ_COLOR_PIXELS, the span functions, CLEAR) and
// sw_SwapBuffers' biSizeImage use `__bpp`, and buffer_size allocates `height *
// WIDTH_BYTES_ALIGN32(width, cColorBits)`. So below 32bpp the C writes to the wrong row and,
// for large y, past the end of the back buffer. The stride here uses this depth's bpp.
unsafe extern "C" fn write_monocolor_pixels_24(ctx: *mut GLcontext, mut n: GLuint,
                                   x: *const GLint, y: *const GLint,
                                   mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_pixels_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_24((*fb).pixel_format, (*sw_ctx).anon_union.u24.CurrentColor));

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(3) as isize);
            PUT_PIXEL_24(Buffer as *mut ULONG, (*sw_ctx).anon_union.u24.CurrentColor);
        }
    }
}

// READ_COLOR_SPAN(24, ULONG, 3): read a horizontal span of color pixels.
unsafe extern "C" fn read_color_span_24(ctx: *mut GLcontext,
                            mut n: GLuint, x: GLint, y: GLint,
                            red: *mut GLubyte, green: *mut GLubyte,
                            blue: *mut GLubyte, alpha: *mut GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Color = GetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int);
            UNPACK_COLORREF_24(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }

        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(3) as usize);
    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        Buffer = Buffer.wrapping_sub(3);
        UNPACK_COLOR_24(GET_PIXEL_24(Buffer as *mut ULONG),
                red.add(n as usize), green.add(n as usize), blue.add(n as usize));
        *alpha.add(n as usize) = 0;
    }
}

// READ_COLOR_PIXELS(24, ULONG, 3): read an array of color pixels.
unsafe extern "C" fn read_color_pixels_24(ctx: *mut GLcontext,
                              mut n: GLuint, x: *const GLint, y: *const GLint,
                              red: *mut GLubyte, green: *mut GLubyte,
                              blue: *mut GLubyte, alpha: *mut GLubyte,
                              mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                Color = GetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int);
                UNPACK_COLORREF_24(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
                *alpha.add(n as usize) = 0;
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 24 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(3) as isize);
            UNPACK_COLOR_24(GET_PIXEL_24(Buffer as *mut ULONG),
                    red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }
    }
}

// MAKE_COLORREF(32, ULONG)
#[inline]
unsafe fn MAKE_COLORREF_32(format: *const pixel_format, Color: ULONG) -> COLORREF {
    let mut r: GLubyte = 0;
    let mut g: GLubyte = 0;
    let mut b: GLubyte = 0;

    if (*format).iPixelType as u32 == PFD_TYPE_COLORINDEX {
        return PALETTEINDEX!(Color);
    }

    UNPACK_COLOR_32(Color, &mut r, &mut g, &mut b);

    return PACK_COLORREF_32(r, g, b);
}

// CLEAR_INDEX(32, ULONG): set the color index used to clear the color buffer.
unsafe extern "C" fn clear_index_32(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u32.ClearColor = index as ULONG;
}

// CLEAR_COLOR(32): set the color used to clear the color buffer.
unsafe extern "C" fn clear_color_32(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u32.ClearColor = PACK_COLOR_32(r, g, b);

    TRACE!("Set Clear color %u, %u, %u.\n", r as c_uint, g as c_uint, b as c_uint);
}

// CLEAR(32, ULONG, 4): clear the specified region of the color buffer using the clear color
// or index set by the two functions above.
unsafe extern "C" fn clear_32(ctx: *mut GLcontext, all: GLboolean, mut x: GLint, mut y: GLint, mut width: GLint, mut height: GLint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut ScanLine: *mut BYTE;

    if all != 0 {
        y = 0;
        x = y;
        width = (*fb).width as GLint;
        height = (*fb).height as GLint;
    }

    if (*sw_ctx).Mode == GL_FRONT {
        clear_frontbuffer(sw_ctx, fb, x, y, width, height,
                MAKE_COLORREF_32((*fb).pixel_format, (*sw_ctx).anon_union.u32.ClearColor));
        return;
    }

    ScanLine = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint)) as usize);
    loop {
        let h = height;
        height = height.wrapping_sub(1);
        if h == 0 {
            break;
        }
        let mut Buffer: *mut BYTE = ScanLine.wrapping_offset(x.wrapping_mul(4) as isize);
        let mut n: UINT = width as UINT;

        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            PUT_PIXEL_32(Buffer as *mut ULONG, (*sw_ctx).anon_union.u32.ClearColor);
            Buffer = Buffer.wrapping_add(4);
        }

        ScanLine = ScanLine.wrapping_add(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint) as usize);
    }
}

// SET_INDEX(32): set the current color index.
unsafe extern "C" fn set_index_32(ctx: *mut GLcontext, index: GLuint) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u32.CurrentColor = index as ULONG;
}

// SET_COLOR(32): set the current RGBA color.
unsafe extern "C" fn set_color_32(ctx: *mut GLcontext, r: GLubyte, g: GLubyte, b: GLubyte, a: GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*sw_ctx).anon_union.u32.CurrentColor = PACK_COLOR_32(r, g, b);
}

// WRITE_COLOR_SPAN_FRONTBUFFER(32): write a horizontal span of color pixels with a boolean mask.
unsafe fn write_color_span_frontbuffer_32(fb: *mut sw_framebuffer,
        mut n: GLuint, x: GLint, y: GLint,
        red: *const GLubyte, green: *const GLubyte,
        blue: *const GLubyte, mask: *const GLubyte) {
    TRACE!("Writing color span at %u, %u (%u)\n", x, y, n);

    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                        PACK_COLORREF_32(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            SetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int,
                    PACK_COLORREF_32(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_COLOR_SPAN(32, ULONG, 4)
unsafe extern "C" fn write_color_span_32(ctx: *mut GLcontext,
                                     mut n: GLuint, x: GLint, y: GLint,
                                     red: *const GLubyte, green: *const GLubyte,
                                     blue: *const GLubyte, alpha: *const GLubyte,
                                     mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_color_span_frontbuffer_32(fb, n, x, y, red, green, blue, mask);
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(4) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(4);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_32(Buffer as *mut ULONG,
                        PACK_COLOR_32(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(4);
            PUT_PIXEL_32(Buffer as *mut ULONG,
                    PACK_COLOR_32(*red.add(n as usize), *green.add(n as usize), *blue.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_SPAN(32, ULONG, 4)
unsafe extern "C" fn write_monocolor_span_32(ctx: *mut GLcontext,
                                 mut n: GLuint, x: GLint, y: GLint,
                                 mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_span_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_32((*fb).pixel_format, (*sw_ctx).anon_union.u32.CurrentColor));
        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(4) as usize);
    if !mask.is_null() {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(4);
            if *mask.add(n as usize) != 0 {
                PUT_PIXEL_32(Buffer as *mut ULONG, (*sw_ctx).anon_union.u32.CurrentColor);
            }
        }
    } else {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Buffer = Buffer.wrapping_sub(4);
            PUT_PIXEL_32(Buffer as *mut ULONG, (*sw_ctx).anon_union.u32.CurrentColor);
        }
    }
}

// WRITE_COLOR_PIXELS(32, ULONG, 4): write an array of pixels with a boolean mask.
unsafe extern "C" fn write_color_pixels_32(ctx: *mut GLcontext,
                               mut n: GLuint, x: *const GLint, y: *const GLint,
                               r: *const GLubyte, g: *const GLubyte,
                               b: *const GLubyte, a: *const GLubyte,
                               mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    TRACE!("Writing color pixels\n");

    if (*sw_ctx).Mode == GL_FRONT {
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                TRACE!("Setting pixel %u, %u to 0x%08x.\n", *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint),
                        PACK_COLORREF_32(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
                SetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int,
                        PACK_COLORREF_32(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(4) as isize);
            PUT_PIXEL_32(Buffer as *mut ULONG, PACK_COLOR_32(*r.add(n as usize), *g.add(n as usize), *b.add(n as usize)));
        }
    }
}

// WRITE_MONOCOLOR_PIXELS(32, ULONG, 4): write an array of pixels with a boolean mask. The
// current color is used for all pixels.
// DIVERGENCE from the C: the C macro computes the row stride as
// `WIDTH_BYTES_ALIGN32(fb->width, 32)` for EVERY depth while stepping x by `__pixel_size`;
// every sibling (WRITE_COLOR_PIXELS, READ_COLOR_PIXELS, the span functions, CLEAR) and
// sw_SwapBuffers' biSizeImage use `__bpp`, and buffer_size allocates `height *
// WIDTH_BYTES_ALIGN32(width, cColorBits)`. So below 32bpp the C writes to the wrong row and,
// for large y, past the end of the back buffer. The stride here uses this depth's bpp.
unsafe extern "C" fn write_monocolor_pixels_32(ctx: *mut GLcontext, mut n: GLuint,
                                   x: *const GLint, y: *const GLint,
                                   mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        write_monocolor_pixels_frontbuffer(fb, n, x, y, mask,
                MAKE_COLORREF_32((*fb).pixel_format, (*sw_ctx).anon_union.u32.CurrentColor));

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(4) as isize);
            PUT_PIXEL_32(Buffer as *mut ULONG, (*sw_ctx).anon_union.u32.CurrentColor);
        }
    }
}

// READ_COLOR_SPAN(32, ULONG, 4): read a horizontal span of color pixels.
unsafe extern "C" fn read_color_span_32(ctx: *mut GLcontext,
                            mut n: GLuint, x: GLint, y: GLint,
                            red: *mut GLubyte, green: *mut GLubyte,
                            blue: *mut GLubyte, alpha: *mut GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;
    let mut Buffer: *mut BYTE;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            Color = GetPixel((*fb).Hdc, (x as GLuint).wrapping_add(n) as c_int, (*fb).height.wrapping_sub(y as GLuint) as c_int);
            UNPACK_COLORREF_32(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }

        return;
    }

    Buffer = (*fb).BackBuffer
        .wrapping_add((y as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint)) as usize)
        .wrapping_add((x as GLuint).wrapping_add(n).wrapping_mul(4) as usize);
    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        Buffer = Buffer.wrapping_sub(4);
        UNPACK_COLOR_32(GET_PIXEL_32(Buffer as *mut ULONG),
                red.add(n as usize), green.add(n as usize), blue.add(n as usize));
        *alpha.add(n as usize) = 0;
    }
}

// READ_COLOR_PIXELS(32, ULONG, 4): read an array of color pixels.
unsafe extern "C" fn read_color_pixels_32(ctx: *mut GLcontext,
                              mut n: GLuint, x: *const GLint, y: *const GLint,
                              red: *mut GLubyte, green: *mut GLubyte,
                              blue: *mut GLubyte, alpha: *mut GLubyte,
                              mask: *const GLubyte) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;
    let fb: *mut sw_framebuffer = (*sw_ctx).fb;

    if (*sw_ctx).Mode == GL_FRONT {
        let mut Color: COLORREF;
        loop {
            let c = n;
            n = n.wrapping_sub(1);
            if c == 0 {
                break;
            }
            if *mask.add(n as usize) != 0 {
                Color = GetPixel((*fb).Hdc, *x.add(n as usize), (*fb).height.wrapping_sub(*y.add(n as usize) as GLuint) as c_int);
                UNPACK_COLORREF_32(Color, red.add(n as usize), green.add(n as usize), blue.add(n as usize));
                *alpha.add(n as usize) = 0;
            }
        }

        return;
    }

    loop {
        let c = n;
        n = n.wrapping_sub(1);
        if c == 0 {
            break;
        }
        if *mask.add(n as usize) != 0 {
            let Buffer: *mut BYTE = (*fb).BackBuffer
                .wrapping_add((*y.add(n as usize) as GLuint).wrapping_mul(WIDTH_BYTES_ALIGN32!((*fb).width, 32 as GLuint)) as usize)
                .wrapping_offset((*x.add(n as usize)).wrapping_mul(4) as isize);
            UNPACK_COLOR_32(GET_PIXEL_32(Buffer as *mut ULONG),
                    red.add(n as usize), green.add(n as usize), blue.add(n as usize));
            *alpha.add(n as usize) = 0;
        }
    }
}

unsafe extern "C" fn setup_DD_pointers(ctx: *mut GLcontext) {
    let sw_ctx: *mut sw_context = (*ctx).DriverCtx as *mut sw_context;

    (*ctx).Driver.RendererString = Some(renderer_string);
    (*ctx).Driver.UpdateState = Some(setup_DD_pointers);

    // ClearIndex / ClearColor / Clear are cdecl in dd.h like every other member, but on
    // amd64/arm64 the island spells those three `extern "system"` -- the same ABI on both
    // Windows 64-bit targets, a different Rust TYPE. The transmute bridges the spelling only
    // (it is the identity on i386, where the island says "C").
    match (*(*(*sw_ctx).fb).pixel_format).cColorBits {
        8 => {
            (*ctx).Driver.ClearIndex = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLuint)>, _>(Some(clear_index_8 as unsafe extern "C" fn(*mut GLcontext, GLuint)));
            (*ctx).Driver.ClearColor = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)>, _>(Some(clear_color_8 as unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)));
            (*ctx).Driver.Clear = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)>, _>(Some(clear_8 as unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)));
            (*ctx).Driver.Index = Some(set_index_8);
            (*ctx).Driver.Color = Some(set_color_8);
            (*ctx).Driver.WriteColorSpan       = Some(write_color_span_8);
            (*ctx).Driver.WriteMonocolorSpan   = Some(write_monocolor_span_8);
            (*ctx).Driver.WriteMonoindexSpan   = Some(write_monocolor_span_8);
            (*ctx).Driver.WriteColorPixels     = Some(write_color_pixels_8);
            (*ctx).Driver.WriteMonocolorPixels = Some(write_monocolor_pixels_8);
            (*ctx).Driver.WriteMonoindexPixels = Some(write_monocolor_pixels_8);
            (*ctx).Driver.ReadColorSpan = Some(read_color_span_8);
            (*ctx).Driver.ReadColorPixels = Some(read_color_pixels_8);
        }
        16 => {
            (*ctx).Driver.ClearIndex = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLuint)>, _>(Some(clear_index_16 as unsafe extern "C" fn(*mut GLcontext, GLuint)));
            (*ctx).Driver.ClearColor = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)>, _>(Some(clear_color_16 as unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)));
            (*ctx).Driver.Clear = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)>, _>(Some(clear_16 as unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)));
            (*ctx).Driver.Index = Some(set_index_16);
            (*ctx).Driver.Color = Some(set_color_16);
            (*ctx).Driver.WriteColorSpan       = Some(write_color_span_16);
            (*ctx).Driver.WriteMonocolorSpan   = Some(write_monocolor_span_16);
            (*ctx).Driver.WriteMonoindexSpan   = Some(write_monocolor_span_16);
            (*ctx).Driver.WriteColorPixels     = Some(write_color_pixels_16);
            (*ctx).Driver.WriteMonocolorPixels = Some(write_monocolor_pixels_16);
            (*ctx).Driver.WriteMonoindexPixels = Some(write_monocolor_pixels_16);
            (*ctx).Driver.ReadColorSpan = Some(read_color_span_16);
            (*ctx).Driver.ReadColorPixels = Some(read_color_pixels_16);
        }
        24 => {
            (*ctx).Driver.ClearIndex = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLuint)>, _>(Some(clear_index_24 as unsafe extern "C" fn(*mut GLcontext, GLuint)));
            (*ctx).Driver.ClearColor = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)>, _>(Some(clear_color_24 as unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)));
            (*ctx).Driver.Clear = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)>, _>(Some(clear_24 as unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)));
            (*ctx).Driver.Index = Some(set_index_24);
            (*ctx).Driver.Color = Some(set_color_24);
            (*ctx).Driver.WriteColorSpan       = Some(write_color_span_24);
            (*ctx).Driver.WriteMonocolorSpan   = Some(write_monocolor_span_24);
            (*ctx).Driver.WriteMonoindexSpan   = Some(write_monocolor_span_24);
            (*ctx).Driver.WriteColorPixels     = Some(write_color_pixels_24);
            (*ctx).Driver.WriteMonocolorPixels = Some(write_monocolor_pixels_24);
            (*ctx).Driver.WriteMonoindexPixels = Some(write_monocolor_pixels_24);
            (*ctx).Driver.ReadColorSpan = Some(read_color_span_24);
            (*ctx).Driver.ReadColorPixels = Some(read_color_pixels_24);
        }
        32 => {
            (*ctx).Driver.ClearIndex = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLuint)>, _>(Some(clear_index_32 as unsafe extern "C" fn(*mut GLcontext, GLuint)));
            (*ctx).Driver.ClearColor = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)>, _>(Some(clear_color_32 as unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)));
            (*ctx).Driver.Clear = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)>, _>(Some(clear_32 as unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)));
            (*ctx).Driver.Index = Some(set_index_32);
            (*ctx).Driver.Color = Some(set_color_32);
            (*ctx).Driver.WriteColorSpan       = Some(write_color_span_32);
            (*ctx).Driver.WriteMonocolorSpan   = Some(write_monocolor_span_32);
            (*ctx).Driver.WriteMonoindexSpan   = Some(write_monocolor_span_32);
            (*ctx).Driver.WriteColorPixels     = Some(write_color_pixels_32);
            (*ctx).Driver.WriteMonocolorPixels = Some(write_monocolor_pixels_32);
            (*ctx).Driver.WriteMonoindexPixels = Some(write_monocolor_pixels_32);
            (*ctx).Driver.ReadColorSpan = Some(read_color_span_32);
            (*ctx).Driver.ReadColorPixels = Some(read_color_pixels_32);
        }
        _ => {
            ERR!("Unhandled bit depth %u, defaulting to 32bpp.\n", (*(*(*sw_ctx).fb).pixel_format).cColorBits as c_uint);
            (*ctx).Driver.ClearIndex = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLuint)>, _>(Some(clear_index_32 as unsafe extern "C" fn(*mut GLcontext, GLuint)));
            (*ctx).Driver.ClearColor = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)>, _>(Some(clear_color_32 as unsafe extern "C" fn(*mut GLcontext, GLubyte, GLubyte, GLubyte, GLubyte)));
            (*ctx).Driver.Clear = core::mem::transmute::<Option<unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)>, _>(Some(clear_32 as unsafe extern "C" fn(*mut GLcontext, GLboolean, GLint, GLint, GLint, GLint)));
            (*ctx).Driver.Index = Some(set_index_32);
            (*ctx).Driver.Color = Some(set_color_32);
            (*ctx).Driver.WriteColorSpan       = Some(write_color_span_32);
            (*ctx).Driver.WriteMonocolorSpan   = Some(write_monocolor_span_32);
            (*ctx).Driver.WriteMonoindexSpan   = Some(write_monocolor_span_32);
            (*ctx).Driver.WriteColorPixels     = Some(write_color_pixels_32);
            (*ctx).Driver.WriteMonocolorPixels = Some(write_monocolor_pixels_32);
            (*ctx).Driver.WriteMonoindexPixels = Some(write_monocolor_pixels_32);
            (*ctx).Driver.ReadColorSpan = Some(read_color_span_32);
            (*ctx).Driver.ReadColorPixels = Some(read_color_pixels_32);
        }
    }

    (*ctx).Driver.SetBuffer = Some(set_buffer);
    (*ctx).Driver.GetBufferSize = Some(buffer_size);

    /* Pixel/span writing functions: */
    (*ctx).Driver.WriteIndexSpan       = Some(write_index_span);
    (*ctx).Driver.WriteIndexPixels     = Some(write_index_pixels);

    /* Pixel/span reading functions: */
    (*ctx).Driver.ReadIndexSpan = Some(read_index_span);
    (*ctx).Driver.ReadIndexPixels = Some(read_index_pixels);
}
#[no_mangle]
pub unsafe extern "C" fn gl_get_thread_context() -> *mut GLcontext {
    let sw_ctx: *mut sw_context = IntGetCurrentDHGLRC() as *mut sw_context;
    return (*sw_ctx).gl_ctx;
}
#[no_mangle]
pub unsafe extern "C" fn sw_SetContext(dc_data: *mut wgl_dc_data, dhglrc: DHGLRC) -> BOOL {
    let sw_ctx: *mut sw_context = dhglrc as *mut sw_context;
    let fb: *mut sw_framebuffer = (*dc_data).sw_data as *mut sw_framebuffer;
    let mut width: UINT = 0;
    let mut height: UINT = 0;

    /* Get framebuffer size */
    if (*dc_data).flags & WGL_DC_OBJ_DC != 0 {
        let hwnd: HWND = (*dc_data).owner.hwnd;
        let mut client_rect: RECT = core::mem::zeroed::<RECT>();
        if hwnd.is_null() {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/opengl/opengl32/swimpl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"sw_SetContext\0" as *const u8 as *const ::core::ffi::c_char,
                    1403 as ::core::ffi::c_int,
                    b"Physical DC without a window!\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
        if GetClientRect(hwnd, core::ptr::addr_of_mut!(client_rect)) == 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/opengl/opengl32/swimpl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"sw_SetContext\0" as *const u8 as *const ::core::ffi::c_char,
                    1408 as ::core::ffi::c_int,
                    b"GetClientRect failed!\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }

        /* This is a physical DC. Setup the hook */
        (*sw_ctx).hook = SetWindowsHookEx(
            (WH_CALLWNDPROC) as i32,
            Some(
                sw_call_window_proc,
            ),
            core::ptr::null_mut(),
            GetCurrentThreadId(),
        );

        /* Calculate width & height */
        width = (client_rect.right - client_rect.left) as UINT;
        height = (client_rect.bottom - client_rect.top) as UINT;
    } else {
        /* OBJ_MEMDC */
        let mut bm: BITMAP = core::mem::zeroed::<BITMAP>();
        let mut hbmp: HBITMAP = core::ptr::null_mut();
        let hdc: HDC = (*dc_data).owner.hdc;

        if (*(*fb).gl_visual).DBflag != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/opengl/opengl32/swimpl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"sw_SetContext\0" as *const u8 as *const ::core::ffi::c_char,
                    1430 as ::core::ffi::c_int,
                    b"Memory DC called with a double buffered format.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }

        hbmp = GetCurrentObject(hdc, OBJ_BITMAP) as HBITMAP;
        if hbmp.is_null() {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/opengl/opengl32/swimpl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"sw_SetContext\0" as *const u8 as *const ::core::ffi::c_char,
                    1437 as ::core::ffi::c_int,
                    b"No Bitmap!\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
        if GetObject(
            (hbmp).cast(),
            core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        ) == 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"dll/opengl/opengl32/swimpl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"sw_SetContext\0" as *const u8 as *const ::core::ffi::c_char,
                    1442 as ::core::ffi::c_int,
                    b"GetObject failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
        width = bm.bmWidth as UINT;
        height = bm.bmHeight as UINT;
    }

    if width == 0 {
        width = 1;
    }
    if height == 0 {
        height = 1;
    }

    /* Also make the mesa context current to mesa */
    gl_make_current((*sw_ctx).gl_ctx, (*fb).gl_buffer);

    /* Setup our functions */
    setup_DD_pointers((*sw_ctx).gl_ctx);

    /* Set the viewport if this is the first time we initialize this context */
    if (*(*sw_ctx).gl_ctx).Viewport.X == 0
        && (*(*sw_ctx).gl_ctx).Viewport.Y == 0
        && (*(*sw_ctx).gl_ctx).Viewport.Width == 0
        && (*(*sw_ctx).gl_ctx).Viewport.Height == 0
    {
        gl_Viewport(
            (*sw_ctx).gl_ctx,
            0,
            0,
            width as GLsizei,
            height as GLsizei,
        );
    }

    /* update the framebuffer size */
    gl_ResizeBuffersMESA((*sw_ctx).gl_ctx);

    /* Use our API table */
    IntSetCurrentDispatchTable(core::ptr::addr_of_mut!(sw_api_table.glDispatchTable));

    /* We're good */
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn sw_ReleaseContext(dhglrc: DHGLRC) {
    let sw_ctx: *mut sw_context = dhglrc as *mut sw_context;

    /* Forward to mesa */
    gl_make_current(core::ptr::null_mut(), core::ptr::null_mut());

    /* Unhook */
    if !(*sw_ctx).hook.is_null() {
        UnhookWindowsHookEx((*sw_ctx).hook);
        (*sw_ctx).hook = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn sw_SwapBuffers(hdc: HDC, dc_data: *mut wgl_dc_data) -> BOOL {
    let mut fb: *mut sw_framebuffer = (*dc_data).sw_data as *mut sw_framebuffer;
    let mut Buffer: [::core::ffi::c_char; ::core::mem::size_of::<BITMAPINFOHEADER>() + 3 * ::core::mem::size_of::<DWORD>()] = [0; ::core::mem::size_of::<BITMAPINFOHEADER>() + 3 * ::core::mem::size_of::<DWORD>()];
    let mut bmi: *mut BITMAPINFO = core::ptr::addr_of_mut!(Buffer) as *mut ::core::ffi::c_char as *mut BITMAPINFO;
    let mut Bpp: BYTE = (*(*fb).pixel_format).cColorBits;

    if (*(*fb).gl_visual).DBflag == 0 {
        return TRUE as BOOL;
    }

    if (*fb).BackBuffer.is_null() {
        return FALSE as BOOL;
    }

    (*bmi).bmiHeader.biSize = ::core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    (*bmi).bmiHeader.biBitCount = Bpp as WORD;
    (*bmi).bmiHeader.biClrImportant = 0 as DWORD;
    (*bmi).bmiHeader.biClrUsed = 0 as DWORD;
    (*bmi).bmiHeader.biPlanes = 1 as WORD;
    (*bmi).bmiHeader.biSizeImage = WIDTH_BYTES_ALIGN32!((*fb).width, Bpp as GLuint).wrapping_mul((*fb).height) as DWORD;
    (*bmi).bmiHeader.biXPelsPerMeter = 0 as LONG;
    (*bmi).bmiHeader.biYPelsPerMeter = 0 as LONG;
    (*bmi).bmiHeader.biHeight = (*fb).height as LONG;
    (*bmi).bmiHeader.biWidth = (*fb).width as LONG;
    (*bmi).bmiHeader.biCompression = (if Bpp as ::core::ffi::c_int == 16 {
        BI_BITFIELDS
    } else {
        BI_RGB
    }) as DWORD;

    if Bpp as ::core::ffi::c_int == 16 {
        let mut BitMasks: *mut DWORD = core::ptr::addr_of_mut!((*bmi).bmiColors[0 as usize]) as *mut DWORD;
        *BitMasks.offset(0 as isize) = 0x0000F800 as DWORD;
        *BitMasks.offset(1 as isize) = 0x000007E0 as DWORD;
        *BitMasks.offset(2 as isize) = 0x0000001F as DWORD;
    }

    return (SetDIBitsToDevice(
        (*fb).Hdc,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        (*fb).width as DWORD,
        (*fb).height as DWORD,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as UINT,
        (*fb).height as UINT,
        (*fb).BackBuffer as *const ::core::ffi::c_void,
        bmi,
        (if (*(*fb).pixel_format).iPixelType as ::core::ffi::c_int == (PFD_TYPE_COLORINDEX as i32) {
            DIB_PAL_COLORS
        } else {
            DIB_RGB_COLORS
        }) as UINT,
    ) != 0 as ::core::ffi::c_int) as BOOL;
}

// get_format — hand-port of dll/opengl/opengl32/swimpl.c:274-313.
//
// Hand-written because the C's `HANDLE_BPP(__x__)` token-pastes `pixel_formats_##__x__`, which
// the fact seal cannot carry (the banked body was rejected: the pasted tokens became
// `pixel_formats___RUSTCONV_FACT_0002__`). The four HANDLE_BPP cases (32/24/16/8) are written
// out explicitly below, in the C's order.
//
// Assumed from the module's c2rust file-scope preamble: `struct pixel_format` and the four
// tables `pixel_formats_32` / `_24` / `_16` / `_8`. GetDC / GetDeviceCaps / ReleaseDC and the
// debug-channel items come from the prelude. No helper items needed.

/* WGL <-> mesa glue */
unsafe fn get_format(mut pf_index: INT, mut pf_count: *mut INT) -> *const pixel_format {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut bpp: INT = 0;
    let mut nb_format: INT = 0;
    let mut ret: *const pixel_format = core::ptr::null();

    hdc = GetDC(core::ptr::null_mut());
    bpp = GetDeviceCaps(hdc, BITSPIXEL as ::core::ffi::c_int) as INT;
    ReleaseDC(core::ptr::null_mut(), hdc);

    match bpp {
        // HANDLE_BPP(32);
        32 => {
            nb_format = ARRAYSIZE!(pixel_formats_32) as INT;
            if (pf_index > nb_format) || (pf_index <= 0) {
                ret = core::ptr::null();
            } else {
                ret = (core::ptr::addr_of!(pixel_formats_32) as *const pixel_format)
                    .offset((pf_index - 1) as isize);
            }
        }
        // HANDLE_BPP(24);
        24 => {
            nb_format = ARRAYSIZE!(pixel_formats_24) as INT;
            if (pf_index > nb_format) || (pf_index <= 0) {
                ret = core::ptr::null();
            } else {
                ret = (core::ptr::addr_of!(pixel_formats_24) as *const pixel_format)
                    .offset((pf_index - 1) as isize);
            }
        }
        // HANDLE_BPP(16);
        16 => {
            nb_format = ARRAYSIZE!(pixel_formats_16) as INT;
            if (pf_index > nb_format) || (pf_index <= 0) {
                ret = core::ptr::null();
            } else {
                ret = (core::ptr::addr_of!(pixel_formats_16) as *const pixel_format)
                    .offset((pf_index - 1) as isize);
            }
        }
        // HANDLE_BPP(8);
        8 => {
            nb_format = ARRAYSIZE!(pixel_formats_8) as INT;
            if (pf_index > nb_format) || (pf_index <= 0) {
                ret = core::ptr::null();
            } else {
                ret = (core::ptr::addr_of!(pixel_formats_8) as *const pixel_format)
                    .offset((pf_index - 1) as isize);
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_FIXME as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/opengl/opengl32/swimpl.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"get_format\0" as *const u8 as *const ::core::ffi::c_char,
                    301 as ::core::ffi::c_int,
                    b"Unhandled bit depth %u, defaulting to 32bpp\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    bpp,
                );
            }
            nb_format = ARRAYSIZE!(pixel_formats_32) as INT;
            // Divergence: the C tests `pf_index == 0` here but `<= 0` in every HANDLE_BPP
            // case, so a negative index reads before the table. Use the same bound as above.
            if (pf_index > nb_format) || (pf_index <= 0) {
                ret = core::ptr::null();
            } else {
                ret = (core::ptr::addr_of!(pixel_formats_32) as *const pixel_format)
                    .offset((pf_index - 1) as isize);
            }
        }
    }

    if !pf_count.is_null() {
        *pf_count = nb_format;
    }

    return ret;
}
