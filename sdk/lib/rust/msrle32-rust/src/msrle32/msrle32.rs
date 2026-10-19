use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut MSRLE32_hModule: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// WINE_DEFAULT_DEBUG_CHANNEL(msrle32) expands to a debug-channel static; not translated here.

unsafe extern "C" fn ColorCmp(clr1: WORD, clr2: WORD) -> WORD {
    let a: UINT = (clr1 as core::ffi::c_int - clr2 as core::ffi::c_int) as UINT;
    return a.wrapping_mul(a) as WORD;
}

unsafe extern "C" fn Intensity(clr: RGBQUAD) -> WORD {
    return ((30 as ::core::ffi::c_int * clr.rgbRed as ::core::ffi::c_int
        + 59 as ::core::ffi::c_int * clr.rgbGreen as ::core::ffi::c_int
        + 11 as ::core::ffi::c_int * clr.rgbBlue as ::core::ffi::c_int)
        / 4 as ::core::ffi::c_int) as WORD;
}

unsafe extern "C" fn isSupportedMRLE(lpbi: LPCBITMAPINFOHEADER) -> BOOL {
    assert!(!lpbi.is_null());

    if ((*lpbi).biSize as usize) < ::core::mem::size_of::<BITMAPINFOHEADER>() as usize
        || (*lpbi).biPlanes as ::core::ffi::c_int != 1
    {
        return (FALSE) as BOOL;
    }

    if (*lpbi).biCompression == BI_RLE4 {
        if (*lpbi).biBitCount != 4
            || (*lpbi).biWidth % 2 != 0
        {
            return FALSE as BOOL;
        }
    } else if (*lpbi).biCompression == BI_RLE8 {
        if (*lpbi).biBitCount != 8 {
            return FALSE as BOOL;
        }
    } else {
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn isSupportedDIB(lpbi: LPCBITMAPINFOHEADER) -> BOOL {
    /* pre-conditions */
    assert!(!lpbi.is_null());

    /* check structure version/planes/compression */
    if ((*lpbi).biSize as usize) < core::mem::size_of::<BITMAPINFOHEADER>()
        || (*lpbi).biPlanes as ::core::ffi::c_int != 1
    {
        return (FALSE) as BOOL;
    }
    if (*lpbi).biCompression != BI_RGB
        && (*lpbi).biCompression != BI_BITFIELDS
    {
        return FALSE as BOOL;
    }

    /* check bit-depth */
    if (*lpbi).biBitCount as ::core::ffi::c_int != 1
        && (*lpbi).biBitCount as ::core::ffi::c_int != 4
        && (*lpbi).biBitCount as ::core::ffi::c_int != 8
        && (*lpbi).biBitCount as ::core::ffi::c_int != 15
        && (*lpbi).biBitCount as ::core::ffi::c_int != 16
        && (*lpbi).biBitCount as ::core::ffi::c_int != 24
        && (*lpbi).biBitCount as ::core::ffi::c_int != 32
    {
        return FALSE as BOOL;
    }

    /* check for size(s) */
    if (*lpbi).biWidth == 0 || (*lpbi).biHeight == 0 {
        return FALSE as BOOL; /* image with zero size, makes no sense so error ! */
    }
    if DIBWIDTHBYTES!(*lpbi).wrapping_mul((*lpbi).biHeight as DWORD)
        >= (1 << 31) - 1
    {
        return FALSE as BOOL; /* image too big ! */
    }

    /* check for nonexistent colortable for hi- and true-color DIB's */
    if (*lpbi).biBitCount as ::core::ffi::c_int >= 15
        && (*lpbi).biClrUsed > 0
    {
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn MSRLE32_GetNearestPaletteIndex(
    count: UINT,
    clrs: *const RGBQUAD,
    clr: RGBQUAD,
) -> BYTE {
    let mut diff: INT = 0x00FFFFFF;
    let mut i: UINT;
    let mut idx: UINT = 0;

    /* pre-conditions */
    assert!(!clrs.is_null());

    i = 0;
    while i < count {
        let mut r: INT = (*clrs.offset(i as isize)).rgbRed as INT - clr.rgbRed as INT;
        let mut g: INT = (*clrs.offset(i as isize)).rgbGreen as INT - clr.rgbGreen as INT;
        let mut b: INT = (*clrs.offset(i as isize)).rgbBlue as INT - clr.rgbBlue as INT;

        r = r * r + g * g + b * b;

        if r < diff {
            idx = i;
            diff = r;
            if diff == 0 {
                break;
            }
        }
        i = i.wrapping_add(1);
    }

    return idx as BYTE;
}
#[no_mangle]
pub unsafe extern "C" fn computeInternalFrame(
    mut pi: *mut CodecInfo,
    mut lpbiIn: LPCBITMAPINFOHEADER,
    mut lpIn: *const BYTE,
) {
    let mut wIntensityTbl: [WORD; 256] = [0; 256];
    let mut lInLine: DWORD = 0;
    let mut lOutLine: DWORD = 0;
    let mut lpOut: LPWORD = ::core::ptr::null_mut::<WORD>();
    let mut i: UINT = 0;
    let mut y: LONG = 0;

    assert!((!pi.is_null() && !lpbiIn.is_null() && !lpIn.is_null()) != false);
    assert!((!(*pi).pCurFrame.is_null()) != false);

    lInLine = DIBWIDTHBYTES!(*lpbiIn);
    lOutLine = (WIDTHBYTES!(
        ((*lpbiIn).biWidth as WORD as usize)
            * 8
            * core::mem::size_of::<WORD>(),
    ) as DWORD)
        / 2;
    lpOut = (*pi).pCurFrame;

    assert!(((*lpbiIn).biClrUsed != 0) != false);

    {
        let lp: *const RGBQUAD =
            (lpbiIn as *const BYTE).offset((*lpbiIn).biSize as isize) as *const RGBQUAD;

        i = 0;
        while (i as DWORD) < (*lpbiIn).biClrUsed {
            wIntensityTbl[i as usize] = Intensity(*lp.offset(i as isize));
            i = i.wrapping_add(1);
        }
    }

    y = 0;
    while y < (*lpbiIn).biHeight {
        let mut x: LONG = 0;

        match (*lpbiIn).biBitCount as ::core::ffi::c_int {
            1 => {
                x = 0;
                while x < (*lpbiIn).biWidth / 8 {
                    i = 0;
                    while i < 7 {
                        *lpOut.offset(
                            ((8 as LONG) * x + (i as LONG)) as isize,
                        ) = wIntensityTbl[((((*lpIn.offset(x as isize)) as ::core::ffi::c_int)
                            >> ((7 as UINT).wrapping_sub(i)))
                            & 1) as usize];
                        i = i.wrapping_add(1);
                    }
                    x += 1;
                }
            }
            4 => {
                x = 0;
                while x < (*lpbiIn).biWidth / 2 {
                    *lpOut.offset(
                        ((2 as LONG) * x + (0 as LONG))
                            as isize,
                    ) = wIntensityTbl[((((*lpIn.offset(x as isize)) as ::core::ffi::c_int)
                        >> 4)) as usize];
                    *lpOut.offset(
                        ((2 as LONG) * x + (1 as LONG))
                            as isize,
                    ) = wIntensityTbl[((((*lpIn.offset(x as isize)) as ::core::ffi::c_int)
                        & 0x0F)) as usize];
                    x += 1;
                }
            }
            8 => {
                x = 0;
                while x < (*lpbiIn).biWidth {
                    *lpOut.offset(x as isize) =
                        wIntensityTbl[*lpIn.offset(x as isize) as usize];
                    x += 1;
                }
            }
            _ => {}
        }

        lpIn = lpIn.offset(lInLine as isize);
        lpOut = lpOut.offset(lOutLine as isize);
        y += 1;
    }
}

unsafe extern "C" fn MSRLE32_GetMaxCompressedSize(lpbi: LPCBITMAPINFOHEADER) -> LONG {
    let mut a: LONG;
    let mut b: LONG;
    let mut size: LONG;

    /* pre-condition */
    assert!((!lpbi.is_null()) != false);

    a = (*lpbi).biWidth / 255;
    b = (*lpbi).biWidth % 255;
    if ((*lpbi).biBitCount as LONG) <= 4 {
        a /= 2;
        b /= 2;
    }

    size = (2
        + a * (2 + ((a + 2) & !2))
        + b * (2 + ((b + 2) & !2)));
    return size * (*lpbi).biHeight + 2;
}

unsafe extern "C" fn countDiffRLE4(lpP: *const WORD, lpA: *const WORD, lpB: *const WORD, mut pos: INT, lDist: LONG, width: LONG) -> INT {
    let mut count: INT = 0;
    let mut clr1: WORD = 0;
    let mut clr2: WORD = 0;

    /* pre-conditions */
    assert!(!lpA.is_null() && !lpB.is_null() && lDist >= (0 as LONG) && width > (0 as LONG));

    if (pos as LONG) >= width {
        return (0 as INT);
    }
    if ((pos as LONG) + (1 as LONG)) == width {
        return (1 as INT);
    }

    let fresh_pos = pos;
    pos = pos + 1;
    clr1 = *lpB.offset(fresh_pos as isize);
    clr2 = *lpB.offset(pos as isize);

    count = (2 as INT);
    while ((pos as LONG) + (1 as LONG)) < width {
        let mut clr3: WORD = 0;
        let mut clr4: WORD = 0;

        pos += 1;
        clr3 = *lpB.offset(pos as isize);
        if ((pos as LONG) + (1 as LONG)) >= width {
            return count + (1 as INT);
        }

        pos += 1;
        clr4 = *lpB.offset(pos as isize);
        if ((ColorCmp(clr1, clr3) as LONG) <= lDist)
            && ((ColorCmp(clr2, clr4) as LONG) <= lDist)
        {
            /* diff at end? -- look-ahead for at least ?? more encodable pixels */
            if ((pos as LONG) + (2 as LONG)) < width
                && ((ColorCmp(clr1, *lpB.offset(((pos as LONG) + (1 as LONG)) as isize))) as LONG) <= lDist
                && ((ColorCmp(clr2, *lpB.offset(((pos as LONG) + (2 as LONG)) as isize))) as LONG) <= lDist
            {
                if ((pos as LONG) + (4 as LONG)) < width
                    && ((ColorCmp(
                        *lpB.offset(((pos as LONG) + (1 as LONG)) as isize),
                        *lpB.offset(((pos as LONG) + (3 as LONG)) as isize),
                    )) as LONG) <= lDist
                    && ((ColorCmp(
                        *lpB.offset(((pos as LONG) + (2 as LONG)) as isize),
                        *lpB.offset(((pos as LONG) + (4 as LONG)) as isize),
                    )) as LONG) <= lDist
                {
                    return count - (3 as INT); /* followed by at least 4 encodable pixels */
                }
                return count - (2 as INT);
            }
        } else if !lpP.is_null()
            && ((ColorCmp(*lpP.offset(pos as isize), *lpB.offset(pos as isize))) as LONG) <= lDist
        {
            /* 'compare' with previous frame for end of diff */
            let mut count2: INT = (0 as INT);

            /* FIXME */

            if count2 >= (8 as INT) {
                return count;
            }

            pos -= count2;
        }

        count += (2 as INT);
        clr1 = clr3;
        clr2 = clr4;
    }

    return count;
}

unsafe extern "C" fn countDiffRLE8(lpP: *const WORD, lpA: *const WORD, lpB: *const WORD, mut pos: INT, lDist: LONG, width: LONG) -> INT {
    let mut count: INT = 0;
    while (pos as LONG) < width {
        if (ColorCmp(*lpA.offset(pos as isize), *lpB.offset(pos as isize)) as LONG) <= lDist {
            /* diff at end? -- look-ahead for some more encodable pixel */
            if ((pos as LONG) + 1) < width
                && (ColorCmp(*lpB.offset(pos as isize), *lpB.offset((pos + 1) as isize)) as LONG) <= lDist
            {
                return count - 1;
            }
            if ((pos as LONG) + 2) < width
                && (ColorCmp(*lpB.offset((pos + 1) as isize), *lpB.offset((pos + 2) as isize)) as LONG) <= lDist
            {
                return count - 1;
            }
        } else if !lpP.is_null()
            && (ColorCmp(*lpP.offset(pos as isize), *lpB.offset(pos as isize)) as LONG) <= lDist
        {
            /* 'compare' with previous frame for end of diff */
            let mut count2: INT = 0;

            count2 = 0;
            pos += 1;
            while (pos as LONG) < width && count2 <= 5 {
                if (ColorCmp(*lpP.offset(pos as isize), *lpB.offset(pos as isize)) as LONG) > lDist {
                    break;
                }
                pos += 1;
                count2 += 1;
            }
            if count2 > 4 {
                return count;
            }

            pos -= count2;
        }
        pos += 1;
        count += 1;
    }

    return count;
}

unsafe extern "C" fn MSRLE32_CompressRLE4Line(
    pi: *const CodecInfo,
    lpP: *const WORD,
    lpC: *const WORD,
    lpbi: LPCBITMAPINFOHEADER,
    lpIn: *const BYTE,
    lDist: LONG,
    mut x: INT,
    ppOut: *mut LPBYTE,
    lpSizeImage: *mut DWORD,
) -> INT {
    let mut lpOut: LPBYTE = *ppOut;
    let mut count: INT = 0;
    let mut pos: INT = 0;
    let mut clr1: WORD = 0;
    let mut clr2: WORD = 0;

    /* try to encode as many pixel as possible */
    count = 1;
    pos = x;
    let fresh0 = pos;
    pos += 1;
    clr1 = *lpC.offset(fresh0 as isize);
    if (pos as LONG) < (*lpbi).biWidth {
        clr2 = *lpC.offset(pos as isize);
        count += 1;
        while ((pos + 1) as LONG) < (*lpbi).biWidth {
            pos += 1;
            if ColorCmp(clr1, *lpC.offset(pos as isize)) as LONG > lDist {
                break;
            }
            count += 1;
            if ((pos + 1) as LONG) >= (*lpbi).biWidth {
                break;
            }
            pos += 1;
            if ColorCmp(clr2, *lpC.offset(pos as isize)) as LONG > lDist {
                break;
            }
            count += 1;
        }
    }

    if count < 4 {
        /* add some pixel for absoluting if possible */
        count += countDiffRLE4(
            lpP,
            lpC.offset(-((1) as isize)),
            lpC,
            pos - 1,
            lDist,
            (*lpbi).biWidth,
        );

        assert!(count > 0);

        /* check for near end of line */
        if ((x + count) as LONG) > (*lpbi).biWidth {
            count = ((*lpbi).biWidth - x as LONG) as INT;
        }

        /* absolute pixel(s) in groups of at least 3 and at most 254 pixels */
        while count > 2 {
            let mut i: INT = 0;
            let mut size: INT = min!(count, 254);
            let mut bytes: INT =
                ((size + 1) & (!(1))) / 2;
            let mut extra_byte: INT = bytes & 0x01;

            *lpSizeImage = (*lpSizeImage)
                .wrapping_add((2 + bytes + extra_byte) as DWORD);
            assert!(((*lpSizeImage) % 2) == 0);
            count -= size;
            *lpOut = 0 as BYTE;
            lpOut = lpOut.offset(1);
            *lpOut = size as BYTE;
            lpOut = lpOut.offset(1);
            i = 0;
            while i < size {
                clr1 = *(*pi)
                    .palette_map
                    .offset(GetRawPixel!(lpbi, lpIn, x) as isize) as WORD;
                x += 1;
                if i + 1 < size {
                    clr2 = *(*pi)
                        .palette_map
                        .offset(GetRawPixel!(lpbi, lpIn, x) as isize) as WORD;
                    x += 1;
                } else {
                    clr2 = 0 as WORD;
                }

                *lpOut = (((clr1 as INT) << 4) | (clr2 as INT)) as BYTE;
                lpOut = lpOut.offset(1);
                i += 2;
            }
            if extra_byte != 0 {
                *lpOut = 0 as BYTE;
                lpOut = lpOut.offset(1);
            }
        }

        if count > 0 {
            /* too little for absoluting so we must encode them */
            assert!(count <= 2);

            *lpSizeImage = (*lpSizeImage).wrapping_add(2 as DWORD);
            clr1 = *(*pi)
                .palette_map
                .offset(GetRawPixel!(lpbi, lpIn, x) as isize) as WORD;
            x += 1;
            if count == 2 {
                clr2 = *(*pi)
                    .palette_map
                    .offset(GetRawPixel!(lpbi, lpIn, x) as isize) as WORD;
                x += 1;
            } else {
                clr2 = 0 as WORD;
            }
            *lpOut = count as BYTE;
            lpOut = lpOut.offset(1);
            *lpOut = (((clr1 as INT) << 4) | (clr2 as INT)) as BYTE;
            lpOut = lpOut.offset(1);
        }
    } else {
        /* encode count pixel(s) */
        clr1 = (((*(*pi)
            .palette_map
            .offset(GetRawPixel!(lpbi, lpIn, x) as isize) as INT)
            << 4)
            | (*(*pi)
                .palette_map
                .offset(GetRawPixel!(lpbi, lpIn, x + 1) as isize) as INT))
            as WORD;

        x += count;
        while count > 0 {
            let mut size_0: INT = min!(count, 254);

            *lpSizeImage = (*lpSizeImage).wrapping_add(2 as DWORD);
            count -= size_0;
            *lpOut = size_0 as BYTE;
            lpOut = lpOut.offset(1);
            *lpOut = clr1 as BYTE;
            lpOut = lpOut.offset(1);
        }
    }

    *ppOut = lpOut;

    return x;
}

unsafe extern "C" fn MSRLE32_CompressRLE8Line(pi: *const CodecInfo, lpP: *const WORD, lpC: *const WORD, lpbi: LPCBITMAPINFOHEADER, lpIn: *const BYTE, mut x: INT, ppOut: *mut LPBYTE, lpSizeImage: *mut DWORD) -> INT {
    let mut lpOut: LPBYTE = *ppOut;
    let mut count: INT = 0;
    let mut pos: INT = 0;
    let mut clr: WORD = 0;

    assert!((((*lpbi).biBitCount as INT) <= 8));
    assert!(((*lpbi).biCompression == BI_RGB));

    /* try to encode as much as possible */
    pos = x;
    clr = *lpC.offset(pos as isize);
    pos += 1;
    count = 1;
    while (pos as LONG) < (*lpbi).biWidth {
        let arg: WORD = *lpC.offset(pos as isize);
        pos += 1;
        if (ColorCmp(clr, arg) as INT) > 0 {
            break;
        }
        count += 1;
    }

    if count < 2 {
        /* add some more pixels for absoluting if possible */
        count += countDiffRLE8(lpP, lpC.offset(-(1 as isize)), lpC, pos - 1, 0 as LONG, (*lpbi).biWidth);

        assert!((count > 0));

        /* check for over end of line */
        if ((x + count) as LONG) > (*lpbi).biWidth {
            count = ((*lpbi).biWidth - x as LONG) as INT;
        }

        /* absolute pixel(s) in groups of at least 3 and at most 255 pixels */
        while count > 2 {
            let mut i: INT = 0;
            let mut size: INT = min!(count, 255);
            let mut extra_byte: INT = size % 2;

            *lpSizeImage = (*lpSizeImage).wrapping_add((2 + size + extra_byte) as DWORD);
            count -= size;
            *lpOut = 0 as BYTE;
            lpOut = lpOut.offset(1);
            *lpOut = size as BYTE;
            lpOut = lpOut.offset(1);
            i = 0;
            while i < size {
                *lpOut = *(*pi).palette_map.offset(GetRawPixel!(lpbi, lpIn, x) as isize);
                lpOut = lpOut.offset(1);
                x += 1;
                i += 1;
            }
            if extra_byte != 0 {
                *lpOut = 0 as BYTE;
                lpOut = lpOut.offset(1);
            }
        }
        if count > 0 {
            /* too little for absoluting so we must encode them even if it's expensive! */
            assert!((count <= 2));

            *lpSizeImage = (*lpSizeImage).wrapping_add((2 * count) as DWORD);
            *lpOut = 1 as BYTE;
            lpOut = lpOut.offset(1);
            *lpOut = *(*pi).palette_map.offset(GetRawPixel!(lpbi, lpIn, x) as isize);
            lpOut = lpOut.offset(1);
            x += 1;

            if count == 2 {
                *lpOut = 1 as BYTE;
                lpOut = lpOut.offset(1);
                *lpOut = *(*pi).palette_map.offset(GetRawPixel!(lpbi, lpIn, x) as isize);
                lpOut = lpOut.offset(1);
                x += 1;
            }
        }
    } else {
        /* encode count pixel(s) */
        clr = *(*pi).palette_map.offset(GetRawPixel!(lpbi, lpIn, x) as isize) as WORD;

        /* optimize end of line */
        if ((x + count + 1) as LONG) == (*lpbi).biWidth {
            count += 1;
        }

        x += count;
        while count > 0 {
            let size: INT = min!(count, 255);

            *lpSizeImage = (*lpSizeImage).wrapping_add(2 as DWORD);
            count -= size;
            *lpOut = size as BYTE;
            lpOut = lpOut.offset(1);
            *lpOut = clr as BYTE;
            lpOut = lpOut.offset(1);
        }
    }

    *ppOut = lpOut;

    return x;
}
#[no_mangle]
pub unsafe extern "C" fn MSRLE32_CompressRLE4(
    pi: *const CodecInfo,
    lpbiIn: LPCBITMAPINFOHEADER,
    mut lpIn: *const BYTE,
    lpbiOut: LPBITMAPINFOHEADER,
    mut lpOut: LPBYTE,
    isKey: BOOL,
) -> LRESULT {
    let mut lpC: LPWORD = core::ptr::null_mut();
    let mut lLine: LONG;
    let mut lInLine: LONG;
    let lpOutStart: LPBYTE = lpOut;

    /* pre-conditions */
    assert!(!pi.is_null() && !lpbiOut.is_null());
    assert!(!lpIn.is_null() && !lpOut.is_null());
    assert!(!(*pi).pCurFrame.is_null());

    lpC = (*pi).pCurFrame;
    lInLine = DIBWIDTHBYTES!(*lpbiIn) as LONG;
    lLine = (WIDTHBYTES!((*lpbiOut).biWidth * 16) as LONG) / 2;

    (*lpbiOut).biSizeImage = 0 as DWORD;
    if isKey != 0 {
        /* keyframe -- convert internal frame to output format */
        let mut x: INT;
        let mut y: INT;

        y = 0 as INT;
        while (y as LONG) < (*lpbiOut).biHeight {
            x = 0 as INT;

            loop {
                x = MSRLE32_CompressRLE4Line(
                    pi,
                    core::ptr::null::<WORD>(),
                    lpC as *const WORD,
                    lpbiIn,
                    lpIn,
                    0 as LONG,
                    x,
                    core::ptr::addr_of_mut!(lpOut),
                    core::ptr::addr_of_mut!((*lpbiOut).biSizeImage),
                );
                if !((x as LONG) < (*lpbiOut).biWidth) {
                    break;
                }
            }

            lpC = lpC.offset(lLine as isize);
            lpIn = lpIn.offset(lInLine as isize);

            /* add EOL -- end of line */
            (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
            *(lpOut as LPWORD) = 0 as WORD;
            lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
            assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));
            y += 1;
        }
    } else {
        /* delta-frame -- compute delta between last and this internal frame */
        let mut lpP: LPWORD = core::ptr::null_mut();
        let mut x: INT;
        let mut y: INT;
        let mut jumpx: INT;
        let mut jumpy: INT;

        assert!(!(*pi).pPrevFrame.is_null());

        lpP = (*pi).pPrevFrame;
        jumpy = 0 as INT;
        jumpx = -(1 as INT);

        y = 0 as INT;
        while (y as LONG) < (*lpbiOut).biHeight {
            x = 0 as INT;

            loop {
                let mut count: INT;
                let mut pos: INT;

                if jumpx == -(1 as INT) {
                    jumpx = x;
                }
                count = 0 as INT;
                pos = x;
                while (pos as LONG) < (*lpbiOut).biWidth {
                    if (ColorCmp(*lpP.offset(pos as isize), *lpC.offset(pos as isize)) as INT)
                        > (0 as INT)
                    {
                        break;
                    }
                    pos += 1;
                    count += 1;
                }

                if ((pos as LONG) == (*lpbiOut).biWidth) && (count > (8 as INT)) {
                    /* (count > 8) secures that we will save space */
                    jumpy += 1;
                    break;
                } else if jumpy != 0 || jumpx != pos {
                    /* time to jump */
                    assert!(jumpx != -(1 as INT));

                    if pos < jumpx {
                        /* can only jump in positive direction -- jump until EOL, EOL */
                        let mut w: INT = ((*lpbiOut).biWidth - jumpx as LONG) as INT;

                        assert!(jumpy > (0 as INT));
                        assert!(w >= (4 as INT));

                        jumpx = 0 as INT;
                        jumpy -= 1;
                        /* if (w % 255 == 2) then equal costs
                         * else if (w % 255 < 4 && we could encode all) then 2 bytes too expensive
                         * else it will be cheaper
                         */
                        while w > (0 as INT) {
                            (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(4 as DWORD);
                            *lpOut = 0 as BYTE;
                            lpOut = lpOut.offset(1);
                            *lpOut = 2 as BYTE;
                            lpOut = lpOut.offset(1);
                            *lpOut = min!(w, 255 as INT) as BYTE;
                            w -= *lpOut as INT;
                            lpOut = lpOut.offset(1);
                            *lpOut = 0 as BYTE;
                            lpOut = lpOut.offset(1);
                        }
                        /* add EOL -- end of line */
                        (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
                        *(lpOut as LPWORD) = 0 as WORD;
                        lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
                    }

                    /* FIXME: if (jumpy == 0 && could encode all) then jump too expensive */

                    /* write out real jump(s) */
                    while jumpy != 0 || pos != jumpx {
                        (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(4 as DWORD);
                        *lpOut = 0 as BYTE;
                        lpOut = lpOut.offset(1);
                        *lpOut = 2 as BYTE;
                        lpOut = lpOut.offset(1);
                        *lpOut = min!(pos - jumpx, 255 as INT) as BYTE;
                        x += *lpOut as INT;
                        jumpx += *lpOut as INT;
                        lpOut = lpOut.offset(1);
                        *lpOut = min!(jumpy, 255 as INT) as BYTE;
                        jumpy -= *lpOut as INT;
                        lpOut = lpOut.offset(1);
                    }

                    jumpy = 0 as INT;
                }

                jumpx = -(1 as INT);

                if (x as LONG) < (*lpbiOut).biWidth {
                    /* skipped the 'same' things corresponding to previous frame */
                    x = MSRLE32_CompressRLE4Line(
                        pi,
                        lpP as *const WORD,
                        lpC as *const WORD,
                        lpbiIn,
                        lpIn,
                        0 as LONG,
                        x,
                        core::ptr::addr_of_mut!(lpOut),
                        core::ptr::addr_of_mut!((*lpbiOut).biSizeImage),
                    );
                }
                if !((x as LONG) < (*lpbiOut).biWidth) {
                    break;
                }
            }

            lpP = lpP.offset(lLine as isize);
            lpC = lpC.offset(lLine as isize);
            lpIn = lpIn.offset(lInLine as isize);

            if jumpy == (0 as INT) {
                assert!(jumpx == -(1 as INT));

                /* add EOL -- end of line */
                (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
                *(lpOut as LPWORD) = 0 as WORD;
                lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
                assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));
            }
            y += 1;
        }

        /* add EOL -- will be changed to EOI */
        (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
        *(lpOut as LPWORD) = 0 as WORD;
        lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
    }

    /* change EOL to EOI -- end of image */
    *lpOut.offset(-(1 as isize)) = 1 as BYTE;
    assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));

    return ICERR_OK as LRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn MSRLE32_CompressRLE8(pi: *const CodecInfo, lpbiIn: LPCBITMAPINFOHEADER, mut lpIn: *const BYTE, lpbiOut: LPBITMAPINFOHEADER, mut lpOut: LPBYTE, isKey: BOOL) -> LRESULT {
    let mut lpC: LPWORD = core::ptr::null_mut::<WORD>();
    let mut lInLine: LONG = 0;
    let mut lLine: LONG = 0;
    let mut lpOutStart: LPBYTE = lpOut;

    assert!(!pi.is_null() && !lpbiOut.is_null());
    assert!(!lpIn.is_null() && !lpOut.is_null());
    assert!(!(*pi).pCurFrame.is_null());

    lpC = (*pi).pCurFrame;
    lInLine = DIBWIDTHBYTES!(*lpbiIn) as LONG;
    lLine = (WIDTHBYTES!((*lpbiOut).biWidth * (16 as LONG)) as LONG) / (2 as LONG);

    (*lpbiOut).biSizeImage = 0 as DWORD;
    if isKey != 0 {
        /* keyframe -- convert internal frame to output format */
        let mut x: INT = 0;
        let mut y: INT = 0;

        y = 0 as INT;
        while (y as LONG) < (*lpbiOut).biHeight {
            x = 0 as INT;
            loop {
                x = MSRLE32_CompressRLE8Line(pi, core::ptr::null::<WORD>(), lpC as *const WORD, lpbiIn, lpIn, x, &raw mut lpOut, &raw mut (*lpbiOut).biSizeImage);
                assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));
                if !((x as LONG) < (*lpbiOut).biWidth) {
                    break;
                }
            }

            lpC = lpC.offset(lLine as isize);
            lpIn = lpIn.offset(lInLine as isize);

            /* add EOL -- end of line */
            (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
            *(lpOut as LPWORD) = 0 as WORD;
            lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
            assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));
            y += 1;
        }
    } else {
        /* delta-frame -- compute delta between last and this internal frame */
        let mut lpP: LPWORD = core::ptr::null_mut::<WORD>();
        let mut x: INT = 0;
        let mut y: INT = 0;
        let mut jumpx: INT = 0;
        let mut jumpy: INT = 0;

        assert!(!(*pi).pPrevFrame.is_null());

        lpP = (*pi).pPrevFrame;
        jumpx = -(1 as INT);
        jumpy = 0 as INT;

        y = 0 as INT;
        while (y as LONG) < (*lpbiOut).biHeight {
            x = 0 as INT;
            loop {
                let mut count: INT = 0;
                let mut pos: INT = 0;

                if jumpx == -(1 as INT) {
                    jumpx = x;
                }
                count = 0 as INT;
                pos = x;
                while (pos as LONG) < (*lpbiOut).biWidth {
                    if (ColorCmp(*lpP.offset(pos as isize), *lpC.offset(pos as isize)) as INT) > (0 as INT) {
                        break;
                    }
                    pos += 1;
                    count += 1;
                }

                if (pos as LONG) == (*lpbiOut).biWidth && count > (4 as INT) {
                    /* (count > 4) secures that we will save space */
                    jumpy += 1;
                    break;
                } else {
                    if jumpy != 0 || jumpx != pos {
                        /* time to jump */
                        assert!(jumpx != -(1 as INT));

                        if pos < jumpx {
                            /* can only jump in positive direction -- do an EOL then jump */
                            assert!(jumpy > (0 as INT));

                            jumpx = 0 as INT;
                            jumpy -= 1;

                            /* add EOL -- end of line */
                            (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
                            *(lpOut as LPWORD) = 0 as WORD;
                            lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
                            assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));
                        }

                        /* FIXME: if (jumpy == 0 && could encode all) then jump too expensive */

                        /* write out real jump(s) */
                        while jumpy != 0 || pos != jumpx {
                            (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(4 as DWORD);
                            *lpOut = 0 as BYTE;
                            lpOut = lpOut.offset(1);
                            *lpOut = 2 as BYTE;
                            lpOut = lpOut.offset(1);
                            *lpOut = min!(pos - jumpx, 255 as INT) as BYTE;
                            jumpx += *lpOut as INT;
                            lpOut = lpOut.offset(1);
                            *lpOut = min!(jumpy, 255 as INT) as BYTE;
                            jumpy -= *lpOut as INT;
                            lpOut = lpOut.offset(1);
                        }
                        x = pos;

                        jumpy = 0 as INT;
                    }

                    jumpx = -(1 as INT);

                    if (x as LONG) < (*lpbiOut).biWidth {
                        /* skip the 'same' things corresponding to previous frame */
                        x = MSRLE32_CompressRLE8Line(pi, lpP as *const WORD, lpC as *const WORD, lpbiIn, lpIn, x, &raw mut lpOut, &raw mut (*lpbiOut).biSizeImage);
                        assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));
                    }
                    if !((x as LONG) < (*lpbiOut).biWidth) {
                        break;
                    }
                }
            }

            lpP = lpP.offset(lLine as isize);
            lpC = lpC.offset(lLine as isize);
            lpIn = lpIn.offset(lInLine as isize);

            if jumpy == (0 as INT) {
                /* add EOL -- end of line */
                (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
                *(lpOut as LPWORD) = 0 as WORD;
                lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
                assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));
            }
            y += 1;
        }

        /* add EOL */
        (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
        *(lpOut as LPWORD) = 0 as WORD;
        lpOut = lpOut.offset(core::mem::size_of::<WORD>() as isize);
    }

    /* add EOI -- end of image */
    (*lpbiOut).biSizeImage = (*lpbiOut).biSizeImage.wrapping_add(2 as DWORD);
    *lpOut = 0 as BYTE;
    lpOut = lpOut.offset(1);
    *lpOut = 1 as BYTE;
    lpOut = lpOut.offset(1);
    assert!(lpOut == lpOutStart.offset((*lpbiOut).biSizeImage as isize));

    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn MSRLE32_DecompressRLE4(
    pi: *const CodecInfo,
    lpbi: LPCBITMAPINFOHEADER,
    mut lpIn: *const BYTE,
    mut lpOut: LPBYTE,
) -> LRESULT {
    let mut bytes_per_pixel: ::core::ffi::c_int = 0;
    let mut line_size: ::core::ffi::c_int = 0;
    let mut pixel_ptr: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut bEndFlag: BOOL = (FALSE as i32);

    assert!(!pi.is_null());
    assert!(!lpbi.is_null() && (*lpbi).biCompression == BI_RGB);
    assert!(!lpIn.is_null() && !lpOut.is_null());

    bytes_per_pixel = ((*lpbi).biBitCount as ::core::ffi::c_int + 1) / 8;
    line_size = DIBWIDTHBYTES!(*lpbi) as ::core::ffi::c_int;

    loop {
        let mut code0: BYTE = 0;
        let mut code1: BYTE = 0;

        code0 = *lpIn;
        lpIn = lpIn.offset(1);
        code1 = *lpIn;
        lpIn = lpIn.offset(1);

        if code0 as ::core::ffi::c_int == 0 {
            let mut extra_byte: ::core::ffi::c_int = 0;

            match code1 as ::core::ffi::c_int {
                0 => {
                    /* EOL - end of line  */
                    pixel_ptr = 0;
                    lpOut = lpOut.offset(line_size as isize);
                }
                1 => {
                    /* EOI - end of image */
                    bEndFlag = (TRUE as i32);
                }
                2 => {
                    /* skip */
                    let v1 = *lpIn as ::core::ffi::c_int;
                    lpIn = lpIn.offset(1);
                    pixel_ptr += v1 * bytes_per_pixel;
                    let v2 = *lpIn as ::core::ffi::c_int;
                    lpIn = lpIn.offset(1);
                    lpOut = lpOut.offset((v2 * line_size) as isize);
                    if pixel_ptr as LONG >= (*lpbi).biWidth * bytes_per_pixel as LONG {
                        pixel_ptr = 0;
                        lpOut = lpOut.offset(line_size as isize);
                    }
                }
                _ => {
                    /* absolute mode */
                    extra_byte = (((code1 as ::core::ffi::c_int + 1) & (!1)) / 2) & 0x01;

                    if (pixel_ptr / bytes_per_pixel + code1 as ::core::ffi::c_int) as LONG > (*lpbi).biWidth {
                        return ICERR_ERROR as LRESULT;
                    }

                    code0 = code1;
                    i = 0;
                    while i < code0 as ::core::ffi::c_int / 2 {
                        if bytes_per_pixel == 1 {
                            code1 = *lpIn.offset(i as isize);
                            let fresh55 = pixel_ptr;
                            pixel_ptr += 1;
                            *lpOut.offset(fresh55 as isize) = *(*pi).palette_map.offset(
                                ((code1 as ::core::ffi::c_int) >> 4) as isize,
                            );
                            if 2 * i + 1 <= code0 as ::core::ffi::c_int {
                                let fresh56 = pixel_ptr;
                                pixel_ptr += 1;
                                *lpOut.offset(fresh56 as isize) = *(*pi).palette_map.offset(
                                    ((code1 as ::core::ffi::c_int) & 0x0F) as isize,
                                );
                            }
                        } else if bytes_per_pixel == 2 {
                            code1 = ((*lpIn.offset(i as isize) as ::core::ffi::c_int) >> 4) as BYTE;
                            let fresh57 = pixel_ptr;
                            pixel_ptr += 1;
                            *lpOut.offset(fresh57 as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 2 + 0) as isize,
                            );
                            let fresh58 = pixel_ptr;
                            pixel_ptr += 1;
                            *lpOut.offset(fresh58 as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 2 + 1) as isize,
                            );

                            if 2 * i + 1 <= code0 as ::core::ffi::c_int {
                                code1 = ((*lpIn.offset(i as isize) as ::core::ffi::c_int) & 0x0F) as BYTE;
                                let fresh59 = pixel_ptr;
                                pixel_ptr += 1;
                                *lpOut.offset(fresh59 as isize) = *(*pi).palette_map.offset(
                                    (code1 as ::core::ffi::c_int * 2 + 0) as isize,
                                );
                                let fresh60 = pixel_ptr;
                                pixel_ptr += 1;
                                *lpOut.offset(fresh60 as isize) = *(*pi).palette_map.offset(
                                    (code1 as ::core::ffi::c_int * 2 + 1) as isize,
                                );
                            }
                        } else {
                            code1 = ((*lpIn.offset(i as isize) as ::core::ffi::c_int) >> 4) as BYTE;
                            *lpOut.offset((pixel_ptr + 0) as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 4 + 0) as isize,
                            );
                            *lpOut.offset((pixel_ptr + 1) as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 4 + 1) as isize,
                            );
                            *lpOut.offset((pixel_ptr + 2) as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 4 + 2) as isize,
                            );
                            pixel_ptr += bytes_per_pixel;

                            if 2 * i + 1 <= code0 as ::core::ffi::c_int {
                                code1 = ((*lpIn.offset(i as isize) as ::core::ffi::c_int) & 0x0F) as BYTE;
                                *lpOut.offset((pixel_ptr + 0) as isize) = *(*pi).palette_map.offset(
                                    (code1 as ::core::ffi::c_int * 4 + 0) as isize,
                                );
                                *lpOut.offset((pixel_ptr + 1) as isize) = *(*pi).palette_map.offset(
                                    (code1 as ::core::ffi::c_int * 4 + 1) as isize,
                                );
                                *lpOut.offset((pixel_ptr + 2) as isize) = *(*pi).palette_map.offset(
                                    (code1 as ::core::ffi::c_int * 4 + 2) as isize,
                                );
                                pixel_ptr += bytes_per_pixel;
                            }
                        }
                        i += 1;
                    }

                    if (code0 as ::core::ffi::c_int & 0x01) != 0 {
                        if bytes_per_pixel == 1 {
                            code1 = *lpIn.offset(i as isize);
                            let fresh61 = pixel_ptr;
                            pixel_ptr += 1;
                            *lpOut.offset(fresh61 as isize) = *(*pi).palette_map.offset(
                                ((code1 as ::core::ffi::c_int) >> 4) as isize,
                            );
                        } else if bytes_per_pixel == 2 {
                            code1 = ((*lpIn.offset(i as isize) as ::core::ffi::c_int) >> 4) as BYTE;
                            let fresh62 = pixel_ptr;
                            pixel_ptr += 1;
                            *lpOut.offset(fresh62 as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 2 + 0) as isize,
                            );
                            let fresh63 = pixel_ptr;
                            pixel_ptr += 1;
                            *lpOut.offset(fresh63 as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 2 + 1) as isize,
                            );
                        } else {
                            code1 = ((*lpIn.offset(i as isize) as ::core::ffi::c_int) >> 4) as BYTE;
                            *lpOut.offset((pixel_ptr + 0) as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 4 + 0) as isize,
                            );
                            *lpOut.offset((pixel_ptr + 1) as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 4 + 1) as isize,
                            );
                            *lpOut.offset((pixel_ptr + 2) as isize) = *(*pi).palette_map.offset(
                                (code1 as ::core::ffi::c_int * 4 + 2) as isize,
                            );
                            pixel_ptr += bytes_per_pixel;
                        }
                        lpIn = lpIn.offset(1);
                    }
                    lpIn = lpIn.offset((code0 as ::core::ffi::c_int / 2) as isize);

                    /* if the RLE code is odd, skip a byte in the stream */
                    if extra_byte != 0 {
                        lpIn = lpIn.offset(1);
                    }
                }
            }
        } else {
            /* coded mode */
            if (pixel_ptr / bytes_per_pixel + code0 as ::core::ffi::c_int) as LONG > (*lpbi).biWidth {
                return ICERR_ERROR as LRESULT;
            }

            if bytes_per_pixel == 1 {
                let mut c1: BYTE = *(*pi).palette_map.offset(((code1 as ::core::ffi::c_int) >> 4) as isize);
                let mut c2: BYTE = *(*pi).palette_map.offset(((code1 as ::core::ffi::c_int) & 0x0F) as isize);

                i = 0;
                while i < code0 as ::core::ffi::c_int {
                    if (i & 1) == 0 {
                        let fresh64 = pixel_ptr;
                        pixel_ptr += 1;
                        *lpOut.offset(fresh64 as isize) = c1;
                    } else {
                        let fresh65 = pixel_ptr;
                        pixel_ptr += 1;
                        *lpOut.offset(fresh65 as isize) = c2;
                    }
                    i += 1;
                }
            } else if bytes_per_pixel == 2 {
                let mut hi1: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) >> 4) * 2 + 0) as isize,
                );
                let mut lo1: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) >> 4) * 2 + 1) as isize,
                );

                let mut hi2: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) & 0x0F) * 2 + 0) as isize,
                );
                let mut lo2: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) & 0x0F) * 2 + 1) as isize,
                );

                i = 0;
                while i < code0 as ::core::ffi::c_int {
                    if (i & 1) == 0 {
                        let fresh66 = pixel_ptr;
                        pixel_ptr += 1;
                        *lpOut.offset(fresh66 as isize) = hi1;
                        let fresh67 = pixel_ptr;
                        pixel_ptr += 1;
                        *lpOut.offset(fresh67 as isize) = lo1;
                    } else {
                        let fresh68 = pixel_ptr;
                        pixel_ptr += 1;
                        *lpOut.offset(fresh68 as isize) = hi2;
                        let fresh69 = pixel_ptr;
                        pixel_ptr += 1;
                        *lpOut.offset(fresh69 as isize) = lo2;
                    }
                    i += 1;
                }
            } else {
                let mut b1: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) >> 4) * 4 + 0) as isize,
                );
                let mut g1: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) >> 4) * 4 + 1) as isize,
                );
                let mut r1: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) >> 4) * 4 + 2) as isize,
                );

                let mut b2: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) & 0x0F) * 4 + 0) as isize,
                );
                let mut g2: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) & 0x0F) * 4 + 1) as isize,
                );
                let mut r2: BYTE = *(*pi).palette_map.offset(
                    (((code1 as ::core::ffi::c_int) & 0x0F) * 4 + 2) as isize,
                );

                i = 0;
                while i < code0 as ::core::ffi::c_int {
                    if (i & 1) == 0 {
                        *lpOut.offset((pixel_ptr + 0) as isize) = b1;
                        *lpOut.offset((pixel_ptr + 1) as isize) = g1;
                        *lpOut.offset((pixel_ptr + 2) as isize) = r1;
                    } else {
                        *lpOut.offset((pixel_ptr + 0) as isize) = b2;
                        *lpOut.offset((pixel_ptr + 1) as isize) = g2;
                        *lpOut.offset((pixel_ptr + 2) as isize) = r2;
                    }
                    pixel_ptr += bytes_per_pixel;
                    i += 1;
                }
            }
        }

        if !(bEndFlag == 0) {
            break;
        }
    }

    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn MSRLE32_DecompressRLE8(
    mut pi: *const CodecInfo,
    mut lpbi: LPCBITMAPINFOHEADER,
    mut lpIn: *const BYTE,
    mut lpOut: LPBYTE,
) -> LRESULT {
    let mut bytes_per_pixel: c_int;
    let mut line_size: c_int;
    let mut pixel_ptr: c_int = 0;
    let mut bEndFlag: BOOL = (FALSE as i32);

    assert!(!pi.is_null());
    assert!(!lpbi.is_null() && (*lpbi).biCompression == BI_RGB);
    assert!(!lpIn.is_null() && !lpOut.is_null());

    bytes_per_pixel = ((*lpbi).biBitCount as c_int + 1) / 8;
    line_size = DIBWIDTHBYTES!(*lpbi) as c_int;

    loop {
        let mut code0: BYTE;
        let mut code1: BYTE;

        let fresh_in0 = lpIn;
        lpIn = lpIn.offset(1);
        code0 = *fresh_in0;
        let fresh_in1 = lpIn;
        lpIn = lpIn.offset(1);
        code1 = *fresh_in1;

        if code0 as c_int == 0 {
            let mut extra_byte: c_int;

            match code1 as c_int {
                0 => {
                    /* EOL - end of line  */
                    pixel_ptr = 0;
                    lpOut = lpOut.offset(line_size as isize);
                }
                1 => {
                    /* EOI - end of image */
                    bEndFlag = (TRUE as i32);
                }
                2 => {
                    /* skip */
                    let fresh_skip0 = lpIn;
                    lpIn = lpIn.offset(1);
                    pixel_ptr += *fresh_skip0 as c_int * bytes_per_pixel;
                    let fresh_skip1 = lpIn;
                    lpIn = lpIn.offset(1);
                    lpOut = lpOut.offset((*fresh_skip1 as c_int * line_size) as isize);
                    if pixel_ptr as LONG >= (*lpbi).biWidth * bytes_per_pixel as LONG {
                        pixel_ptr = 0;
                        lpOut = lpOut.offset(line_size as isize);
                    }
                }
                _ => {
                    /* absolute mode */
                    if (pixel_ptr / bytes_per_pixel + code1 as c_int) as LONG > (*lpbi).biWidth {
                        WARN!(
                            "aborted absolute: (%d=%d/%d+%d) > %ld\n",
                            pixel_ptr / bytes_per_pixel + code1 as c_int,
                            pixel_ptr,
                            bytes_per_pixel,
                            code1 as c_int,
                            (*lpbi).biWidth
                        );
                        return (ICERR_ERROR) as LRESULT;
                    }
                    extra_byte = code1 as c_int & 0x01;

                    code0 = code1;
                    loop {
                        let fresh_abs0 = code0;
                        code0 = code0.wrapping_sub(1);
                        if !(fresh_abs0 != 0) {
                            break;
                        }
                        let fresh_abs1 = lpIn;
                        lpIn = lpIn.offset(1);
                        code1 = *fresh_abs1;
                        if bytes_per_pixel == 1 {
                            *lpOut.offset(pixel_ptr as isize) =
                                *(*pi).palette_map.offset(code1 as isize);
                        } else if bytes_per_pixel == 2 {
                            *lpOut.offset((pixel_ptr + 0) as isize) =
                                *(*pi).palette_map.offset(
                                    (code1 as c_int * 2 + 0) as isize,
                                );
                            *lpOut.offset((pixel_ptr + 1) as isize) =
                                *(*pi).palette_map.offset(
                                    (code1 as c_int * 2 + 1) as isize,
                                );
                        } else {
                            *lpOut.offset((pixel_ptr + 0) as isize) =
                                *(*pi).palette_map.offset(
                                    (code1 as c_int * 4 + 0) as isize,
                                );
                            *lpOut.offset((pixel_ptr + 1) as isize) =
                                *(*pi).palette_map.offset(
                                    (code1 as c_int * 4 + 1) as isize,
                                );
                            *lpOut.offset((pixel_ptr + 2) as isize) =
                                *(*pi).palette_map.offset(
                                    (code1 as c_int * 4 + 2) as isize,
                                );
                        }
                        pixel_ptr += bytes_per_pixel;
                    }

                    /* if the RLE code is odd, skip a byte in the stream */
                    if extra_byte != 0 {
                        lpIn = lpIn.offset(1);
                    }
                }
            }
        } else {
            /* coded mode */
            if (pixel_ptr / bytes_per_pixel + code0 as c_int) as LONG > (*lpbi).biWidth {
                WARN!(
                    "aborted coded: (%d=%d/%d+%d) > %ld\n",
                    pixel_ptr / bytes_per_pixel + code1 as c_int,
                    pixel_ptr,
                    bytes_per_pixel,
                    code1 as c_int,
                    (*lpbi).biWidth
                );
                return (ICERR_ERROR) as LRESULT;
            }

            if bytes_per_pixel == 1 {
                code1 = *(*pi).palette_map.offset(code1 as isize);
                loop {
                    let fresh_c1 = code0;
                    code0 = code0.wrapping_sub(1);
                    if !(fresh_c1 != 0) {
                        break;
                    }
                    let fresh_p1 = pixel_ptr;
                    pixel_ptr = pixel_ptr + 1;
                    *lpOut.offset(fresh_p1 as isize) = code1;
                }
            } else if bytes_per_pixel == 2 {
                let mut hi: BYTE = *(*pi).palette_map.offset(
                    (code1 as c_int * 2 + 0) as isize,
                );
                let mut lo: BYTE = *(*pi).palette_map.offset(
                    (code1 as c_int * 2 + 1) as isize,
                );

                loop {
                    let fresh_c2 = code0;
                    code0 = code0.wrapping_sub(1);
                    if !(fresh_c2 != 0) {
                        break;
                    }
                    *lpOut.offset((pixel_ptr + 0) as isize) = hi;
                    *lpOut.offset((pixel_ptr + 1) as isize) = lo;
                    pixel_ptr += bytes_per_pixel;
                }
            } else {
                let mut r: BYTE = *(*pi).palette_map.offset(
                    (code1 as c_int * 4 + 2) as isize,
                );
                let mut g: BYTE = *(*pi).palette_map.offset(
                    (code1 as c_int * 4 + 1) as isize,
                );
                let mut b: BYTE = *(*pi).palette_map.offset(
                    (code1 as c_int * 4 + 0) as isize,
                );

                loop {
                    let fresh_c3 = code0;
                    code0 = code0.wrapping_sub(1);
                    if !(fresh_c3 != 0) {
                        break;
                    }
                    *lpOut.offset((pixel_ptr + 0) as isize) = b;
                    *lpOut.offset((pixel_ptr + 1) as isize) = g;
                    *lpOut.offset((pixel_ptr + 2) as isize) = r;
                    pixel_ptr += bytes_per_pixel;
                }
            }
        }

        if !(bEndFlag == 0) {
            break;
        }
    }

    return (ICERR_OK) as LRESULT;
}

unsafe extern "C" fn Open(icinfo: LPICOPEN) -> *mut CodecInfo {
    let mut pi: *mut CodecInfo = core::ptr::null_mut::<CodecInfo>();

    if icinfo.is_null() {
        return 0xFFFF0000 as ::core::ffi::c_uint as LPVOID as *mut CodecInfo;
    }

    if compare_fourcc!((*icinfo).fccType, ICTYPE_VIDEO) != 0 {
        return core::ptr::null_mut::<CodecInfo>();
    }

    if (*icinfo).fccHandler == FOURCC_RLE
        || (*icinfo).fccHandler == FOURCC_RLE4
        || (*icinfo).fccHandler == FOURCC_RLE8
        || (*icinfo).fccHandler == FOURCC_MRLE
    {
        // break
    } else if (*icinfo).fccHandler == mmioFOURCC!('m', 'r', 'l', 'e') {
        (*icinfo).fccHandler = FOURCC_MRLE;
    } else {
        WARN!(
            "unknown FOURCC = 0x%08lX(%4.4s) !\n",
            (*icinfo).fccHandler,
            &raw mut (*icinfo).fccHandler as *mut ::core::ffi::c_char
        );
        return core::ptr::null_mut::<CodecInfo>();
    }

    pi = LocalAlloc(
        LPTR as UINT,
        core::mem::size_of::<CodecInfo>() as SIZE_T,
    ) as *mut CodecInfo;

    if !pi.is_null() {
        (*pi).fccHandler = (*icinfo).fccHandler;

        (*pi).bCompress = FALSE as BOOL;
        (*pi).nPrevFrame = -(1 as LONG);
        (*pi).pCurFrame = core::ptr::null_mut::<WORD>();
        (*pi).pPrevFrame = (*pi).pCurFrame;

        (*pi).bDecompress = FALSE as BOOL;
        (*pi).palette_map = core::ptr::null_mut::<BYTE>();
    }

    (*icinfo).dwError = (if !pi.is_null() {
        ICERR_OK
    } else {
        (ICERR_MEMORY) as u32
    }) as LRESULT;

    return pi;
}

unsafe extern "C" fn Close(pi: *mut CodecInfo) -> LRESULT {
    assert!(!pi.is_null());

    if !(*pi).pPrevFrame.is_null() || !(*pi).pCurFrame.is_null() {
        CompressEnd(pi);
    }

    LocalFree(pi as HLOCAL);
    return 1;
}

unsafe extern "C" fn GetInfo(pi: *const CodecInfo, icinfo: *mut ICINFO, dwSize: DWORD) -> LRESULT {
    /* pre-condition */
    assert!(!pi.is_null());

    /* check parameters */
    if icinfo.is_null() {
        return core::mem::size_of::<ICINFO>() as LRESULT;
    }
    if (dwSize as usize) < core::mem::size_of::<ICINFO>() {
        return 0;
    }

    (*icinfo).dwSize = core::mem::size_of::<ICINFO>() as DWORD;
    (*icinfo).fccType = ICTYPE_VIDEO;
    (*icinfo).fccHandler = if !pi.is_null() {
        (*pi).fccHandler
    } else {
        FOURCC_MRLE
    };
    (*icinfo).dwFlags = VIDCF_QUALITY | VIDCF_TEMPORAL | VIDCF_CRUNCH;
    (*icinfo).dwVersion = ICVERSION;
    (*icinfo).dwVersionICM = ICVERSION;

    LoadStringW(
        MSRLE32_hModule,
        IDS_NAME,
        (*icinfo).szName.as_mut_ptr(),
        (ARRAY_SIZE!((*icinfo).szName)) as i32,
    );
    LoadStringW(
        MSRLE32_hModule,
        IDS_DESCRIPTION,
        (*icinfo).szDescription.as_mut_ptr(),
        (ARRAY_SIZE!((*icinfo).szDescription)) as i32,
    );

    return core::mem::size_of::<ICINFO>() as LRESULT;
}

unsafe extern "C" fn Configure(pi: *const CodecInfo, hWnd: HWND) -> LRESULT {
    /* pre-condition */
    assert!(!pi.is_null());

    /* FIXME */
    return (ICERR_OK) as LRESULT;
}

unsafe extern "C" fn About(pi: *mut CodecInfo, hWnd: HWND) -> LRESULT {
    let mut szTitle: [WCHAR; 20] = [0; 20];
    let mut szAbout: [WCHAR; 128] = [0; 128];

    /* pre-condition */
    assert!(MSRLE32_hModule != 0 as HINSTANCE);

    LoadStringW(
        MSRLE32_hModule,
        IDS_NAME as UINT,
        core::ptr::addr_of_mut!(szTitle) as LPWSTR,
        ARRAY_SIZE!(szTitle) as ::core::ffi::c_int,
    );
    LoadStringW(
        MSRLE32_hModule,
        IDS_ABOUT as UINT,
        core::ptr::addr_of_mut!(szAbout) as LPWSTR,
        ARRAY_SIZE!(szAbout) as ::core::ffi::c_int,
    );

    MessageBoxW(
        hWnd,
        core::ptr::addr_of_mut!(szAbout) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
        (MB_OK | MB_ICONINFORMATION) as UINT,
    );

    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn CompressGetFormat(
    pi: *mut CodecInfo,
    lpbiIn: LPCBITMAPINFOHEADER,
    lpbiOut: LPBITMAPINFOHEADER,
) -> LRESULT {
    let mut size: LRESULT = 0;

    /* pre-condition */
    assert!(!pi.is_null());

    /* check parameters -- need at least input format */
    if lpbiIn.is_null() {
        if !lpbiOut.is_null() {
            return (ICERR_BADPARAM) as LRESULT;
        }
        return 0;
    }

    /* handle unsupported input format */
    if CompressQuery(pi, lpbiIn, core::ptr::null_mut()) != (ICERR_OK) as LRESULT {
        return (if lpbiOut.is_null() { ICERR_BADFORMAT } else { 0 }) as LRESULT;
    }

    assert!(0 < (*lpbiIn).biBitCount && (*lpbiIn).biBitCount <= 8);

    match (*pi).fccHandler {
        FOURCC_RLE4 => {
            size = (1 << 4) as LRESULT;
        }
        FOURCC_RLE8 => {
            size = (1 << 8) as LRESULT;
        }
        FOURCC_RLE | FOURCC_MRLE => {
            size = (if (*lpbiIn).biBitCount <= 4 {
                (1 << 4)
            } else {
                (1 << 8)
            }) as LRESULT;
        }
        _ => return (ICERR_ERROR) as LRESULT,
    }

    if (*lpbiIn).biClrUsed != 0 {
        size = (*lpbiIn).biClrUsed as LRESULT;
    }

    size = (core::mem::size_of::<BITMAPINFOHEADER>())
        .wrapping_add((size as usize).wrapping_mul(core::mem::size_of::<RGBQUAD>()))
        as LRESULT;

    if !lpbiOut.is_null() {
        (*lpbiOut).biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
        (*lpbiOut).biWidth = (*lpbiIn).biWidth;
        (*lpbiOut).biHeight = (*lpbiIn).biHeight;
        (*lpbiOut).biPlanes = 1;
        if (*pi).fccHandler == FOURCC_RLE4 || (*lpbiIn).biBitCount <= 4 {
            (*lpbiOut).biCompression = BI_RLE4;
            (*lpbiOut).biBitCount = 4;
        } else {
            (*lpbiOut).biCompression = BI_RLE8;
            (*lpbiOut).biBitCount = 8;
        }
        (*lpbiOut).biSizeImage = MSRLE32_GetMaxCompressedSize(lpbiOut as LPCBITMAPINFOHEADER) as DWORD;
        (*lpbiOut).biXPelsPerMeter = (*lpbiIn).biXPelsPerMeter;
        (*lpbiOut).biYPelsPerMeter = (*lpbiIn).biYPelsPerMeter;
        if (*lpbiIn).biClrUsed == 0 {
            size = (1 << (*lpbiIn).biBitCount) as LRESULT;
        } else {
            size = (*lpbiIn).biClrUsed as LRESULT;
        }
        (*lpbiOut).biClrUsed = min!(size, (1 << (*lpbiOut).biBitCount) as LRESULT) as DWORD;
        (*lpbiOut).biClrImportant = 0;

        memcpy(
            (lpbiOut as LPBYTE).offset((*lpbiOut).biSize as isize) as *mut core::ffi::c_void,
            (lpbiIn as *const BYTE).offset((*lpbiIn).biSize as isize) as *const core::ffi::c_void,
            ((*lpbiOut).biClrUsed as usize).wrapping_mul(core::mem::size_of::<RGBQUAD>()),
        );

        return (ICERR_OK) as LRESULT;
    } else {
        return size;
    }
}

unsafe extern "C" fn CompressGetSize(pi: *mut CodecInfo, lpbiIn: LPCBITMAPINFOHEADER, lpbiOut: LPCBITMAPINFOHEADER) -> LRESULT {
    assert!(!pi.is_null());

    if lpbiIn.is_null() && lpbiOut.is_null() {
        return 0 as LRESULT;
    }
    if CompressQuery(pi, lpbiIn, lpbiOut) != ICERR_OK as LRESULT {
        return 0 as LRESULT;
    }
    if !lpbiIn.is_null() {
        return MSRLE32_GetMaxCompressedSize(lpbiIn) as LRESULT;
    } else {
        return MSRLE32_GetMaxCompressedSize(lpbiOut) as LRESULT;
    }
}

unsafe extern "C" fn CompressQuery(pi: *const CodecInfo, lpbiIn: LPCBITMAPINFOHEADER, lpbiOut: LPCBITMAPINFOHEADER) -> LRESULT {
    /* pre-condition */
    assert!(!pi.is_null());

    /* need at least one format */
    if lpbiIn.is_null() && lpbiOut.is_null() {
        return ICERR_BADPARAM as LRESULT;
    }

    /* check input format if given */
    if !lpbiIn.is_null() {
        if isSupportedDIB(lpbiIn) == 0 {
            return ICERR_BADFORMAT as LRESULT;
        }

        /* for 4-bit need an even width */
        if ((*lpbiIn).biBitCount as ::core::ffi::c_int) <= 4
            && ((*lpbiIn).biWidth % 2) != 0
        {
            return ICERR_BADFORMAT as LRESULT;
        }

        if (*pi).fccHandler == FOURCC_RLE4
            && ((*lpbiIn).biBitCount as ::core::ffi::c_int) > 4
        {
            return ICERR_UNSUPPORTED as LRESULT;
        } else if ((*lpbiIn).biBitCount as ::core::ffi::c_int) > 8 {
            return ICERR_UNSUPPORTED as LRESULT;
        }
    }

    /* check output format if given */
    if !lpbiOut.is_null() {
        if isSupportedMRLE(lpbiOut) == 0 {
            return ICERR_BADFORMAT as LRESULT;
        }

        if !lpbiIn.is_null() {
            if (*lpbiIn).biWidth != (*lpbiOut).biWidth {
                return ICERR_UNSUPPORTED as LRESULT;
            }
            if (*lpbiIn).biHeight != (*lpbiOut).biHeight {
                return ICERR_UNSUPPORTED as LRESULT;
            }
            if ((*lpbiIn).biBitCount as ::core::ffi::c_int)
                > ((*lpbiOut).biBitCount as ::core::ffi::c_int)
            {
                return ICERR_UNSUPPORTED as LRESULT;
            }
        }
    }

    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn CompressBegin(pi: *mut CodecInfo, lpbiIn: LPCBITMAPINFOHEADER, lpbiOut: LPCBITMAPINFOHEADER) -> LRESULT {
    let mut rgbIn: *const RGBQUAD = core::ptr::null();
    let mut rgbOut: *const RGBQUAD = core::ptr::null();
    let mut i: UINT = 0;
    let mut size: size_t = 0;

    /* pre-condition */
    assert!(!pi.is_null());

    /* check parameters -- need both formats */
    if lpbiIn.is_null() || lpbiOut.is_null() {
        return ICERR_BADPARAM as LRESULT;
    }
    /* And both must be supported */
    if CompressQuery(pi, lpbiIn, lpbiOut) != ICERR_OK as LRESULT {
        return ICERR_BADFORMAT as LRESULT;
    }

    /* FIXME: cannot compress and decompress at same time! */
    if (*pi).bDecompress != 0 {
        return ICERR_ERROR as LRESULT;
    }

    if (*pi).bCompress != 0 {
        CompressEnd(pi);
    }

    size = (WIDTHBYTES!((*lpbiOut).biWidth * 16) / 2 * (*lpbiOut).biHeight as DWORD) as size_t;
    (*pi).pPrevFrame = GlobalLock(GlobalAlloc(GPTR as UINT, (size * core::mem::size_of::<WORD>()) as SIZE_T)) as LPWORD;
    if (*pi).pPrevFrame.is_null() {
        return ICERR_MEMORY as LRESULT;
    }
    (*pi).pCurFrame = GlobalLock(GlobalAlloc(GPTR as UINT, (size * core::mem::size_of::<WORD>()) as SIZE_T)) as LPWORD;
    if (*pi).pCurFrame.is_null() {
        CompressEnd(pi);
        return ICERR_MEMORY as LRESULT;
    }
    (*pi).nPrevFrame = -(1 as LONG);
    (*pi).bCompress = TRUE as BOOL;

    rgbIn = (lpbiIn as *const BYTE).add((*lpbiIn).biSize as usize) as *const RGBQUAD;
    rgbOut = (lpbiOut as *const BYTE).add((*lpbiOut).biSize as usize) as *const RGBQUAD;

    match (*lpbiOut).biBitCount as core::ffi::c_int {
        4 | 8 => {
            (*pi).palette_map = LocalAlloc(LPTR as UINT, (*lpbiIn).biClrUsed as SIZE_T) as LPBYTE;
            if (*pi).palette_map.is_null() {
                CompressEnd(pi);
                return ICERR_MEMORY as LRESULT;
            }

            i = 0 as UINT;
            while (i as DWORD) < (*lpbiIn).biClrUsed {
                *(*pi).palette_map.add(i as usize) = MSRLE32_GetNearestPaletteIndex(
                    (*lpbiOut).biClrUsed as UINT,
                    rgbOut,
                    *rgbIn.add(i as usize),
                );
                i = i.wrapping_add(1);
            }
        }
        _ => {}
    }

    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn Compress(pi: *mut CodecInfo, lpic: *mut ICCOMPRESS, dwSize: DWORD) -> LRESULT {
    let mut is_key: BOOL;
    let mut i: i32;

    /* pre-condition */
    assert!(!pi.is_null());

    /* check parameters */
    if lpic.is_null() || (dwSize as usize) < core::mem::size_of::<ICCOMPRESS>() as usize {
        return ICERR_BADPARAM as LRESULT;
    }
    if (*lpic).lpbiOutput.is_null()
        || (*lpic).lpOutput.is_null()
        || (*lpic).lpbiInput.is_null()
        || (*lpic).lpInput.is_null()
    {
        return ICERR_BADPARAM as LRESULT;
    }

    if (*pi).bCompress == 0 {
        let hr: LRESULT = CompressBegin(
            pi,
            (*lpic).lpbiInput as LPCBITMAPINFOHEADER,
            (*lpic).lpbiOutput as LPCBITMAPINFOHEADER,
        );
        if hr != ICERR_OK as LRESULT {
            return hr;
        }
    } else if CompressQuery(
        pi,
        (*lpic).lpbiInput as LPCBITMAPINFOHEADER,
        (*lpic).lpbiOutput as LPCBITMAPINFOHEADER,
    ) != ICERR_OK as LRESULT
    {
        return ICERR_BADFORMAT as LRESULT;
    }

    if (*lpic).lFrameNum >= (*pi).nPrevFrame + 1 as LONG {
        /* we continue in the sequence so we need to initialize
         * our internal framedata */
        computeInternalFrame(
            pi,
            (*lpic).lpbiInput as LPCBITMAPINFOHEADER,
            (*lpic).lpInput as *const BYTE,
        );
    } else if (*lpic).lFrameNum == (*pi).nPrevFrame {
        /* Oops, compress same frame again ? Okay, as you wish.
         * No need to recompute internal framedata, because we only swapped buffers */
        let pTmp: LPWORD = (*pi).pPrevFrame;

        (*pi).pPrevFrame = (*pi).pCurFrame;
        (*pi).pCurFrame = pTmp;
    } else if ((*lpic).dwFlags & ICCOMPRESS_KEYFRAME) == 0 {
        let mut pTmp: LPWORD;

        WARN!(": prev=%ld cur=%ld gone back? -- untested\n", (*pi).nPrevFrame, (*lpic).lFrameNum);
        if (*lpic).lpbiPrev.is_null() || (*lpic).lpPrev.is_null() {
            return ICERR_GOTOKEYFRAME as LRESULT; /* Need a keyframe if you go back */
        }
        if CompressQuery(
            pi,
            (*lpic).lpbiPrev as LPCBITMAPINFOHEADER,
            (*lpic).lpbiOutput as LPCBITMAPINFOHEADER,
        ) != ICERR_OK as LRESULT
        {
            return ICERR_BADFORMAT as LRESULT;
        }

        WARN!(": prev=%ld cur=%ld compute swapped -- untested\n", (*pi).nPrevFrame, (*lpic).lFrameNum);
        computeInternalFrame(
            pi,
            (*lpic).lpbiPrev as LPCBITMAPINFOHEADER,
            (*lpic).lpPrev as *const BYTE,
        );

        /* swap buffers for current and previous frame */
        /* Don't free and alloc new -- costs too much time and they are of equal size ! */
        pTmp = (*pi).pPrevFrame;
        (*pi).pPrevFrame = (*pi).pCurFrame;
        (*pi).pCurFrame = pTmp;
        (*pi).nPrevFrame = (*lpic).lFrameNum;
    }

    is_key = (((*lpic).dwFlags & ICCOMPRESS_KEYFRAME) != 0) as BOOL;

    i = 0 as i32;
    while i < 3 as i32 {
        (*(*lpic).lpbiOutput).biSizeImage = 0 as DWORD;

        if (*(*lpic).lpbiOutput).biBitCount == 4 as WORD {
            MSRLE32_CompressRLE4(
                pi,
                (*lpic).lpbiInput as LPCBITMAPINFOHEADER,
                (*lpic).lpInput as *const BYTE,
                (*lpic).lpbiOutput,
                (*lpic).lpOutput as LPBYTE,
                is_key,
            );
        } else {
            MSRLE32_CompressRLE8(
                pi,
                (*lpic).lpbiInput as LPCBITMAPINFOHEADER,
                (*lpic).lpInput as *const BYTE,
                (*lpic).lpbiOutput,
                (*lpic).lpOutput as LPBYTE,
                is_key,
            );
        }

        if (*lpic).dwFrameSize == 0 as DWORD
            || (*(*lpic).lpbiOutput).biSizeImage < (*lpic).dwFrameSize
        {
            break;
        }

        if ((*lpic).lpdwFlags.read() & ICCOMPRESS_KEYFRAME as DWORD) == 0 as DWORD {
            if (*(*lpic).lpbiOutput).biBitCount == 4 as WORD {
                MSRLE32_CompressRLE4(
                    pi,
                    (*lpic).lpbiInput as LPCBITMAPINFOHEADER,
                    (*lpic).lpInput as *const BYTE,
                    (*lpic).lpbiOutput,
                    (*lpic).lpOutput as LPBYTE,
                    (TRUE != 0) as i32,
                );
            } else {
                MSRLE32_CompressRLE8(
                    pi,
                    (*lpic).lpbiInput as LPCBITMAPINFOHEADER,
                    (*lpic).lpInput as *const BYTE,
                    (*lpic).lpbiOutput,
                    (*lpic).lpOutput as LPBYTE,
                    (TRUE != 0) as i32,
                );
            }

            if (*lpic).dwFrameSize == 0 as DWORD
                || (*(*lpic).lpbiOutput).biSizeImage < (*lpic).dwFrameSize
            {
                WARN!("switched to keyframe, was small enough!\n");
                is_key = TRUE as BOOL;
                *(*lpic).lpckid = MAKEAVICKID!(
                    cktypeDIBbits,
                    StreamFromFOURCC!(*(*lpic).lpckid),
                ) as DWORD;
                break;
            }
        }

        if (*lpic).dwQuality < 1000 as DWORD {
            break;
        }
        (*lpic).dwQuality = (*lpic).dwQuality.wrapping_sub(1000 as DWORD); /* reduce quality by 10% */
        i += 1;
    }

    { /* swap buffer for current and previous frame */
        /* Don't free and alloc new -- costs too much time and they are of equal size ! */
        let pTmp: LPWORD = (*pi).pPrevFrame;

        (*pi).pPrevFrame = (*pi).pCurFrame;
        (*pi).pCurFrame = pTmp;
        (*pi).nPrevFrame = (*lpic).lFrameNum;
    }

    /* FIXME: What is AVIIF_TWOCC? */
    *(*lpic).lpdwFlags |= (AVIIF_TWOCC
        | (if is_key != 0 {
            AVIIF_KEYFRAME
        } else {
            0
        })) as DWORD;
    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn CompressEnd(pi: *mut CodecInfo) -> LRESULT {
    if !pi.is_null() {
        if !(*pi).pPrevFrame.is_null() {
            GlobalUnlock(GlobalHandle((*pi).pPrevFrame as LPCVOID));
            GlobalFree(GlobalHandle((*pi).pPrevFrame as LPCVOID));
        }
        if !(*pi).pCurFrame.is_null() {
            GlobalUnlock(GlobalHandle((*pi).pCurFrame as LPCVOID));
            GlobalFree(GlobalHandle((*pi).pCurFrame as LPCVOID));
        }
        (*pi).pPrevFrame = core::ptr::null_mut();
        (*pi).pCurFrame = core::ptr::null_mut();
        (*pi).nPrevFrame = -(1 as LONG);
        (*pi).bCompress = FALSE as BOOL;

        if !(*pi).palette_map.is_null() {
            LocalFree((*pi).palette_map as HLOCAL);
            (*pi).palette_map = core::ptr::null_mut();
        }
    }

    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn DecompressGetFormat(
    mut pi: *mut CodecInfo,
    mut lpbiIn: LPCBITMAPINFOHEADER,
    mut lpbiOut: LPBITMAPINFOHEADER,
) -> LRESULT {
    let mut size: DWORD = 0;

    /* pre-condition */
    assert!(!pi.is_null());

    if lpbiIn.is_null() {
        return (if !lpbiOut.is_null() {
            ICERR_BADPARAM
        } else {
            0
        }) as LRESULT;
    }

    if DecompressQuery(pi, lpbiIn, core::ptr::null()) != ICERR_OK as LRESULT {
        return (if !lpbiOut.is_null() {
            ICERR_BADFORMAT
        } else {
            0
        }) as LRESULT;
    }

    size = (*lpbiIn).biSize;

    if ((*lpbiIn).biBitCount as c_int) <= (8 as c_int) {
        let mut colors: c_int = 0;

        if (*lpbiIn).biClrUsed == 0 {
            colors = (1 as c_int) << ((*lpbiIn).biBitCount as c_int);
        } else {
            colors = (*lpbiIn).biClrUsed as c_int;
        }

        size = size.wrapping_add(
            (colors as usize).wrapping_mul(core::mem::size_of::<RGBQUAD>()) as DWORD,
        );
    }

    if !lpbiOut.is_null() {
        memcpy(
            lpbiOut as *mut c_void,
            lpbiIn as *const c_void,
            size as usize,
        );
        (*lpbiOut).biCompression = BI_RGB as DWORD;
        (*lpbiOut).biSizeImage = DIBWIDTHBYTES!(*lpbiOut) * ((*lpbiOut).biHeight as DWORD);

        return ICERR_OK as LRESULT;
    } else {
        return size as LRESULT;
    }
}

unsafe extern "C" fn DecompressQuery(
    pi: *mut CodecInfo,
    lpbiIn: LPCBITMAPINFOHEADER,
    lpbiOut: LPCBITMAPINFOHEADER,
) -> LRESULT {
    let mut hr: LRESULT = ICERR_OK as LRESULT;

    /* pre-condition */
    assert!(!pi.is_null());

    /* need at least one format */
    if lpbiIn.is_null() && lpbiOut.is_null() {
        return ICERR_BADPARAM as LRESULT;
    }

    /* check input format if given */
    if !lpbiIn.is_null() {
        if isSupportedMRLE(lpbiIn) == 0 && isSupportedDIB(lpbiIn) == 0 {
            return ICERR_BADFORMAT as LRESULT;
        }
    }

    /* check output format if given */
    if !lpbiOut.is_null() {
        if isSupportedDIB(lpbiOut) == 0 {
            hr = ICERR_BADFORMAT as LRESULT;
        }

        if !lpbiIn.is_null() {
            if (*lpbiIn).biWidth != (*lpbiOut).biWidth {
                hr = ICERR_UNSUPPORTED as LRESULT;
            }
            if (*lpbiIn).biHeight != (*lpbiOut).biHeight {
                hr = ICERR_UNSUPPORTED as LRESULT;
            }
            if (*lpbiIn).biBitCount > (*lpbiOut).biBitCount {
                hr = ICERR_UNSUPPORTED as LRESULT;
            }
        }
    }

    return hr;
}

unsafe extern "C" fn DecompressBegin(mut pi: *mut CodecInfo, mut lpbiIn: LPCBITMAPINFOHEADER, mut lpbiOut: LPCBITMAPINFOHEADER) -> LRESULT {
    let mut rgbIn: *const RGBQUAD = core::ptr::null();
    let mut rgbOut: *const RGBQUAD = core::ptr::null();
    let mut i: UINT = 0;

    /* pre-condition */
    assert!(!pi.is_null());

    /* check parameters */
    if lpbiIn.is_null() || lpbiOut.is_null() {
        return ICERR_BADPARAM as LRESULT;
    }
    if DecompressQuery(pi, lpbiIn, lpbiOut) != ICERR_OK as LRESULT {
        return ICERR_BADFORMAT as LRESULT;
    }

    /* FIXME: cannot compress and decompress at a time! */
    if (*pi).bCompress != 0 {
        return ICERR_ERROR as LRESULT;
    }

    if (*pi).bDecompress != 0 {
        DecompressEnd(pi);
    }

    if (*lpbiIn).biCompression != BI_RGB as DWORD {
        let mut colors: i32 = 0;

        if (*lpbiIn).biBitCount <= 8 as WORD && (*lpbiIn).biClrUsed == 0 as DWORD {
            colors = 1 << (*lpbiIn).biBitCount;
        } else {
            colors = (*lpbiIn).biClrUsed as i32;
        }

        rgbIn = ((lpbiIn as *const BYTE).offset((*lpbiIn).biSize as isize)) as *const RGBQUAD;
        rgbOut = ((lpbiOut as *const BYTE).offset((*lpbiOut).biSize as isize)) as *const RGBQUAD;

        match (*lpbiOut).biBitCount {
            4 | 8 => {
                (*pi).palette_map = LocalAlloc(LPTR as UINT, colors as SIZE_T) as LPBYTE;
                if (*pi).palette_map.is_null() {
                    return ICERR_MEMORY as LRESULT;
                }

                i = 0 as UINT;
                while i < colors as UINT {
                    *(*pi).palette_map.offset(i as isize) = MSRLE32_GetNearestPaletteIndex(
                        colors as UINT,
                        rgbOut,
                        *rgbIn.offset(i as isize),
                    );
                    i = i.wrapping_add(1);
                }
            }
            15 | 16 => {
                (*pi).palette_map = LocalAlloc(
                    LPTR as UINT,
                    (colors * 2) as SIZE_T,
                ) as LPBYTE;
                if (*pi).palette_map.is_null() {
                    return ICERR_MEMORY as LRESULT;
                }

                i = 0 as UINT;
                while i < colors as UINT {
                    let mut color: WORD = 0;

                    if (*lpbiOut).biBitCount == 15 {
                        color = ((((*rgbIn.offset(i as isize)).rgbRed as i32 >> 3) << 10)
                            | (((*rgbIn.offset(i as isize)).rgbGreen as i32 >> 3) << 5)
                            | ((*rgbIn.offset(i as isize)).rgbBlue as i32 >> 3)) as WORD;
                    } else {
                        color = ((((*rgbIn.offset(i as isize)).rgbRed as i32 >> 3) << 11)
                            | (((*rgbIn.offset(i as isize)).rgbGreen as i32 >> 3) << 5)
                            | ((*rgbIn.offset(i as isize)).rgbBlue as i32 >> 3)) as WORD;
                    }

                    *(*pi).palette_map.offset(
                        i.wrapping_mul(2 as UINT).wrapping_add(1 as UINT) as isize,
                    ) = ((color as i32) >> 8) as BYTE;
                    *(*pi).palette_map.offset(
                        i.wrapping_mul(2 as UINT).wrapping_add(0 as UINT) as isize,
                    ) = ((color as i32) & 0xFF) as BYTE;
                    i = i.wrapping_add(1);
                }
            }
            24 | 32 => {
                (*pi).palette_map = LocalAlloc(
                    LPTR as UINT,
                    (colors as SIZE_T).wrapping_mul(core::mem::size_of::<RGBQUAD>() as SIZE_T),
                ) as LPBYTE;
                if (*pi).palette_map.is_null() {
                    return ICERR_MEMORY as LRESULT;
                }
                memcpy(
                    (*pi).palette_map as *mut core::ffi::c_void,
                    rgbIn as *const core::ffi::c_void,
                    (colors as size_t).wrapping_mul(core::mem::size_of::<RGBQUAD>() as size_t),
                );
            }
            _ => {}
        }
    }
    (*pi).bDecompress = TRUE as BOOL;

    return ICERR_OK as LRESULT;
}

unsafe extern "C" fn Decompress(pi: *mut CodecInfo, pic: *mut ICDECOMPRESS, dwSize: DWORD) -> LRESULT {
    /* pre-condition */
    assert!(!pi.is_null());

    /* check parameters */
    if pic.is_null() {
        return (ICERR_BADPARAM) as LRESULT;
    }
    if (*pic).lpbiInput.is_null()
        || (*pic).lpInput.is_null()
        || (*pic).lpbiOutput.is_null()
        || (*pic).lpOutput.is_null()
    {
        return ICERR_BADPARAM as LRESULT;
    }

    /* check formats */
    if (*pi).bDecompress == 0 {
        let mut hr: LRESULT = DecompressBegin(
            pi,
            (*pic).lpbiInput as LPCBITMAPINFOHEADER,
            (*pic).lpbiOutput as LPCBITMAPINFOHEADER,
        );
        if hr != (ICERR_OK as LRESULT) {
            return hr;
        }
    } else if DecompressQuery(
        pi,
        (*pic).lpbiInput as LPCBITMAPINFOHEADER,
        (*pic).lpbiOutput as LPCBITMAPINFOHEADER,
    ) != (ICERR_OK as LRESULT)
    {
        return ICERR_BADFORMAT as LRESULT;
    }

    assert!((*(*pic).lpbiInput).biWidth == (*(*pic).lpbiOutput).biWidth);
    assert!((*(*pic).lpbiInput).biHeight == (*(*pic).lpbiOutput).biHeight);

    /* Uncompressed frame? */
    if (*(*pic).lpbiInput).biCompression == BI_RGB {
        (*(*pic).lpbiOutput).biSizeImage = (*(*pic).lpbiInput).biSizeImage;
        memcpy(
            (*pic).lpOutput as *mut core::ffi::c_void,
            (*pic).lpInput as *const core::ffi::c_void,
            (*(*pic).lpbiOutput).biSizeImage as usize,
        );
        return (ICERR_OK) as LRESULT;
    }

    (*(*pic).lpbiOutput).biSizeImage =
        DIBWIDTHBYTES!(*(*pic).lpbiOutput) * ((*(*pic).lpbiOutput).biHeight) as u32;
    if (*(*pic).lpbiInput).biBitCount == 4 {
        return MSRLE32_DecompressRLE4(
            pi,
            (*pic).lpbiOutput as LPCBITMAPINFOHEADER,
            (*pic).lpInput as *const BYTE,
            (*pic).lpOutput as LPBYTE,
        );
    } else {
        return MSRLE32_DecompressRLE8(
            pi,
            (*pic).lpbiOutput as LPCBITMAPINFOHEADER,
            (*pic).lpInput as *const BYTE,
            (*pic).lpOutput as LPBYTE,
        );
    }
}

unsafe extern "C" fn DecompressEnd(pi: *mut CodecInfo) -> LRESULT {
    assert!(!pi.is_null());

    (*pi).bDecompress = (FALSE as i32);

    if !(*pi).palette_map.is_null() {
        LocalFree((*pi).palette_map as HLOCAL);
        (*pi).palette_map = core::ptr::null_mut();
    }

    return (ICERR_OK) as LRESULT;
}

unsafe extern "C" fn DecompressGetPalette(pi: *mut CodecInfo, lpbiIn: LPCBITMAPINFOHEADER, lpbiOut: LPBITMAPINFOHEADER) -> LRESULT {
    let mut size: ::core::ffi::c_int;

    /* pre-condition */
    assert!(!pi.is_null());

    /* check parameters */
    if lpbiIn.is_null() || lpbiOut.is_null() {
        return ICERR_BADPARAM as LRESULT;
    }

    if DecompressQuery(pi, lpbiIn, lpbiOut as LPCBITMAPINFOHEADER) != ICERR_OK as LRESULT {
        return ICERR_BADFORMAT as LRESULT;
    }

    if (*lpbiOut).biBitCount as ::core::ffi::c_int > 8 as ::core::ffi::c_int {
        return ICERR_ERROR as LRESULT;
    }

    if (*lpbiIn).biBitCount as ::core::ffi::c_int <= 8 as ::core::ffi::c_int {
        if (*lpbiIn).biClrUsed > 0 as DWORD {
            size = (*lpbiIn).biClrUsed as ::core::ffi::c_int;
        } else {
            size = (1 as ::core::ffi::c_int) << ((*lpbiIn).biBitCount as ::core::ffi::c_int);
        }

        (*lpbiOut).biClrUsed = size as DWORD;

        memcpy(
            (lpbiOut as LPBYTE).offset((*lpbiOut).biSize as isize) as *mut ::core::ffi::c_void,
            (lpbiIn as *const BYTE).offset((*lpbiIn).biSize as isize) as *const ::core::ffi::c_void,
            (size as usize) * core::mem::size_of::<RGBQUAD>(),
        );
    } /* else could never occur ! */

    return ICERR_OK as LRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn MSRLE32_DriverProc(
    mut dwDrvID: DWORD_PTR,
    mut hDrv: HDRVR,
    mut uMsg: UINT,
    mut lParam1: LPARAM,
    mut lParam2: LPARAM,
) -> LRESULT {
    let mut pi: *mut CodecInfo = dwDrvID as *mut CodecInfo;

    TRACE!(
        "(%Ix,%p,0x%04X,0x%08IX,0x%08IX)\n",
        dwDrvID,
        hDrv,
        uMsg,
        lParam1,
        lParam2
    );

    match uMsg {
        /* standard driver messages */
        DRV_LOAD => {
            return DRVCNF_OK as LRESULT;
        }
        DRV_OPEN => {
            return Open(lParam2 as LPICOPEN) as LRESULT;
        }
        DRV_CLOSE => {
            if dwDrvID != 0xFFFF0000 as DWORD_PTR && !(dwDrvID as LPVOID).is_null() {
                Close(pi);
            }
            return DRVCNF_OK as LRESULT;
        }
        DRV_ENABLE | DRV_DISABLE => {
            return DRVCNF_OK as LRESULT;
        }
        DRV_FREE => {
            return DRVCNF_OK as LRESULT;
        }
        DRV_QUERYCONFIGURE => {
            return DRVCNF_CANCEL as LRESULT;
        }
        DRV_CONFIGURE => {
            return DRVCNF_OK as LRESULT;
        }
        DRV_INSTALL | DRV_REMOVE => {
            return DRVCNF_OK as LRESULT;
        }

        /* installable compression manager messages */
        ICM_CONFIGURE => {
            FIXME!("ICM_CONFIGURE (%Id)\n", lParam1);
            if lParam1 == -(1 as LPARAM) {
                return ICERR_UNSUPPORTED as LRESULT;
            } else {
                return Configure(pi, lParam1 as HWND);
            }
        }
        ICM_ABOUT => {
            if lParam1 == -(1 as LPARAM) {
                return ICERR_OK as LRESULT;
            } else {
                return About(pi, lParam1 as HWND);
            }
        }
        ICM_GETSTATE | ICM_SETSTATE => {
            return 0 as LRESULT;
        }
        ICM_GETINFO => {
            return GetInfo(pi, lParam1 as *mut ICINFO, lParam2 as DWORD);
        }
        ICM_GETDEFAULTQUALITY => {
            if (lParam1 as LPVOID) != core::ptr::null_mut() {
                *(lParam1 as LPDWORD) = MSRLE32_DEFAULTQUALITY as DWORD;
                return ICERR_OK as LRESULT;
            }
        }
        ICM_COMPRESS_GET_FORMAT => {
            return CompressGetFormat(
                pi,
                lParam1 as LPCBITMAPINFOHEADER,
                lParam2 as LPBITMAPINFOHEADER,
            );
        }
        ICM_COMPRESS_GET_SIZE => {
            return CompressGetSize(
                pi,
                lParam1 as LPCBITMAPINFOHEADER,
                lParam2 as LPCBITMAPINFOHEADER,
            );
        }
        ICM_COMPRESS_QUERY => {
            return CompressQuery(
                pi,
                lParam1 as LPCBITMAPINFOHEADER,
                lParam2 as LPCBITMAPINFOHEADER,
            );
        }
        ICM_COMPRESS_BEGIN => {
            return CompressBegin(
                pi,
                lParam1 as LPCBITMAPINFOHEADER,
                lParam2 as LPCBITMAPINFOHEADER,
            );
        }
        ICM_COMPRESS => {
            return Compress(pi, lParam1 as *mut ICCOMPRESS, lParam2 as DWORD);
        }
        ICM_COMPRESS_END => {
            return CompressEnd(pi);
        }
        ICM_DECOMPRESS_GET_FORMAT => {
            return DecompressGetFormat(
                pi,
                lParam1 as LPCBITMAPINFOHEADER,
                lParam2 as LPBITMAPINFOHEADER,
            );
        }
        ICM_DECOMPRESS_QUERY => {
            return DecompressQuery(
                pi,
                lParam1 as LPCBITMAPINFOHEADER,
                lParam2 as LPCBITMAPINFOHEADER,
            );
        }
        ICM_DECOMPRESS_BEGIN => {
            return DecompressBegin(
                pi,
                lParam1 as LPCBITMAPINFOHEADER,
                lParam2 as LPCBITMAPINFOHEADER,
            );
        }
        ICM_DECOMPRESS => {
            return Decompress(pi, lParam1 as *mut ICDECOMPRESS, lParam2 as DWORD);
        }
        ICM_DECOMPRESS_END => {
            return DecompressEnd(pi);
        }
        ICM_DECOMPRESS_SET_PALETTE => {
            FIXME!(
                "(...) -> SetPalette(%p,%p,%p): stub!\n",
                pi,
                lParam1 as LPVOID,
                lParam2 as LPVOID
            );
            return ICERR_UNSUPPORTED as LRESULT;
        }
        ICM_DECOMPRESS_GET_PALETTE => {
            return DecompressGetPalette(
                pi,
                lParam1 as LPBITMAPINFOHEADER,
                lParam2 as LPBITMAPINFOHEADER,
            );
        }
        ICM_GETDEFAULTKEYFRAMERATE => {
            if (lParam1 as LPVOID) != core::ptr::null_mut() {
                *(lParam1 as LPDWORD) = 15 as DWORD;
            }
            return ICERR_OK as LRESULT;
        }
        _ => {
            if uMsg < DRV_USER as UINT {
                return DefDriverProc(dwDrvID, hDrv, uMsg, lParam1, lParam2);
            } else {
                FIXME!(
                    "Unknown message uMsg=0x%08X lParam1=0x%08IX lParam2=0x%08IX\n",
                    uMsg,
                    lParam1,
                    lParam2
                );
            }
        }
    }

    return ICERR_UNSUPPORTED as LRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hModule: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hModule as HMODULE);
            MSRLE32_hModule = hModule;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
