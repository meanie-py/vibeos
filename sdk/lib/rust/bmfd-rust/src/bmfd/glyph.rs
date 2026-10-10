use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _ReadPixel(
    pjBits: *mut CHAR,
    x: ULONG,
    y: ULONG,
    ulHeight: ULONG,
) -> ULONG {
    let mut j: CHAR = 0;
    j = *pjBits.offset(
        (x / 8).wrapping_mul(ulHeight).wrapping_add(y) as isize,
    );
    return (((j as i32) >> (!x & 0x7)) & (1 as i32)) as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn _WritePixel(
    mut pjBits: *mut CHAR,
    mut x: ULONG,
    mut y: ULONG,
    mut cjRow: ULONG,
    mut color: ULONG,
) {
    pjBits = pjBits.offset(y.wrapping_mul(cjRow) as isize);
    pjBits = pjBits.offset((x / 8) as isize);
    *pjBits = (*pjBits as ULONG | (color << (!x & 0x7))) as CHAR;
}
#[no_mangle]
pub unsafe extern "C" fn BmfdGetFontInstance(
    pfo: *mut FONTOBJ,
    pface: PBMFD_FACE,
) -> PBMFD_FONT {
    let mut pfont: PBMFD_FONT = (*pfo).pvProducer as PBMFD_FONT;
    let mut pxo: *mut XFORMOBJ = core::ptr::null_mut();
    let mut xfo: FLOATOBJ_XFORM = core::mem::zeroed();

    if pfont.is_null() {
        /* Allocate realization info */
        pfont = EngAllocMem(0, core::mem::size_of::<BMFD_FONT>() as ULONG, 0) as PBMFD_FONT;
        if pfont.is_null() {
            return core::ptr::null_mut();
        }

        pxo = FONTOBJ_pxoGetXform(pfo);
        XFORMOBJ_iGetFloatObjXform(pxo, core::ptr::addr_of_mut!(xfo));

        (*pfont).pfo = pfo;
        (*pfont).pface = pface;
        (*pfont).xScale = FLOATOBJ_GetLong!(core::ptr::addr_of_mut!(xfo.eM11)) as LONG;
        (*pfont).yScale = FLOATOBJ_GetLong!(core::ptr::addr_of_mut!(xfo.eM22)) as LONG;
        (*pfont).ulAngle = 0;

        /* Set the pvProducer member of the fontobj */
        (*pfo).pvProducer = pfont as PVOID;
    }

    return pfont;
}

#[no_mangle]
pub unsafe extern "C" fn BmfdQueryGlyphAndBitmap(
    pfont: PBMFD_FONT,
    hg: HGLYPH,
    pgd: *mut GLYPHDATA,
    pgb: *mut GLYPHBITS,
    cjSize: ULONG,
) -> ULONG {
    let pface: PBMFD_FACE = (*pfont).pface;
    let mut pge: PGLYPHENTRY = core::ptr::null_mut();
    let mut xSrc: ULONG = 0;
    let mut ySrc: ULONG = 0;
    let mut cxSrc: ULONG = 0;
    let mut cySrc: ULONG = 0;
    let mut xDst: ULONG = 0;
    let mut yDst: ULONG = 0;
    let mut cxDst: ULONG = 0;
    let mut cyDst: ULONG = 0;
    let mut xScale: LONG = 0;
    let mut yScale: LONG = 0;
    let mut ulGlyphOffset: ULONG = 0;
    let mut cjDstRow: ULONG = 0;
    let mut color: ULONG = 0;
    let mut pvSrc0: PVOID = core::ptr::null_mut();
    let mut pvDst0: PVOID = core::ptr::null_mut();

    /* The glyph handle is the byte offset to the glyph in the table */
    pge = (*pface).pCharTable.offset(hg as isize) as PGLYPHENTRY;

    /* Get the bitmap offset depending on file version */
    if (*pface).ulVersion >= 0x300 {
        cxSrc = GETVAL!((*pge).ge20.geWidth) as ULONG;
        ulGlyphOffset = GETVAL!((*pge).ge30.geOffset) as ULONG;
    } else {
        cxSrc = GETVAL!((*pge).ge30.geWidth) as ULONG;
        ulGlyphOffset = GETVAL!((*pge).ge20.geOffset) as ULONG;
    }
    cySrc = (*pface).wPixHeight as ULONG;

    /* Pointer to the bitmap bits */
    pvSrc0 = ((*pface).pFontInfo as PBYTE).offset(ulGlyphOffset as isize) as PVOID;
    pvDst0 = core::ptr::addr_of_mut!((*pgb).aj) as *mut BYTE as PVOID;

    xScale = (*pfont).xScale;
    yScale = (*pfont).yScale;

    /* Calculate extents of destination bitmap */
    if (*pfont).ulAngle == 90 || (*pfont).ulAngle == 270 {
        cxDst = cySrc.wrapping_mul(xScale as ULONG);
        cyDst = cxSrc.wrapping_mul(yScale as ULONG);
    } else {
        cxDst = cxSrc.wrapping_mul(xScale as ULONG);
        cyDst = cySrc.wrapping_mul(yScale as ULONG);
    }
    cjDstRow = cxDst.wrapping_add(7).wrapping_div(8);

    if !pgd.is_null() {
        /* Fill GLYPHDATA structure */
        (*pgd).gdf.pgb = pgb;
        (*pgd).hg = hg;
        (*pgd).fxD = ((xScale as ULONG).wrapping_mul(
            ((*pface).wA as ULONG)
                .wrapping_add(cxDst)
                .wrapping_add((*pface).wC as ULONG),
        ) << 4) as FIX;
        (*pgd).fxA = ((xScale * (*pface).wA as LONG) << 4) as FIX;
        (*pgd).fxAB = ((xScale as ULONG).wrapping_mul(((*pface).wA as ULONG).wrapping_add(cxDst))
            << 4) as FIX;
        (*pgd).fxInkTop = ((yScale * (*pface).wAscent as LONG) << 4) as FIX;
        (*pgd).fxInkBottom = (-yScale
            * (((*pface).wDescent as LONG) << 4)) as FIX;
        (*pgd).rclInk.top = -yScale * (*pface).wAscent as LONG;
        (*pgd).rclInk.bottom = yScale * (*pface).wDescent as LONG;
        (*pgd).rclInk.left = xScale * (*pface).wA as LONG;
        (*pgd).rclInk.right = ((*pgd).rclInk.left as ULONG).wrapping_add(cxDst) as LONG;
        (*pgd).ptqD.x.__bindgen_anon_1.LowPart = 0;
        (*pgd).ptqD.x.__bindgen_anon_1.HighPart = (*pgd).fxD as LONG;
        (*pgd).ptqD.y.__bindgen_anon_1.LowPart = 0;
        (*pgd).ptqD.y.__bindgen_anon_1.HighPart = 0;
    }

    if !pgb.is_null() {
        /* Verify that the buffer is big enough */
        if cjSize
            < (FIELD_OFFSET!(GLYPHBITS, aj) as ULONG)
                .wrapping_add(cyDst.wrapping_mul(cjDstRow))
        {
            DbgPrint(
                b"Buffer too small (%ld), %ld,%ld\n\0".as_ptr() as *const i8 as PCHAR,
                cjSize,
                cxSrc,
                cySrc,
            );
            return FD_ERROR as ULONG;
        }

        /* Fill GLYPHBITS structure */
        (*pgb).ptlOrigin.x = xScale * (*pface).wA as LONG;
        (*pgb).ptlOrigin.y = -yScale * (*pface).wAscent as LONG;
        (*pgb).sizlBitmap.cx = cxDst as LONG;
        (*pgb).sizlBitmap.cy = cyDst as LONG;

        /* Erase destination surface */
        memset(
            pvDst0 as *mut core::ffi::c_void,
            0,
            cyDst.wrapping_mul(cjDstRow) as usize,
        );

        match (*pfont).ulAngle {
            90 => {
                /* Copy pixels */
                yDst = 0;
                while yDst < cyDst {
                    xSrc = yDst.wrapping_div(yScale as ULONG);
                    xDst = 0;
                    while xDst < cxDst {
                        ySrc = cxDst.wrapping_sub(xDst).wrapping_div(xScale as ULONG);
                        color = _ReadPixel(pvSrc0 as *mut CHAR, xSrc, ySrc, cySrc);
                        _WritePixel(pvDst0 as *mut CHAR, xDst, yDst, cjDstRow, color);
                        xDst = xDst.wrapping_add(1);
                    }
                    yDst = yDst.wrapping_add(1);
                }
                // break;
            }

            180 => {
                yDst = 0;
                while yDst < cyDst {
                    ySrc = cyDst.wrapping_sub(yDst).wrapping_div(yScale as ULONG);
                    xDst = 0;
                    while xDst < cxDst {
                        xSrc = cxDst.wrapping_sub(xDst).wrapping_div(xScale as ULONG);
                        color = _ReadPixel(pvSrc0 as *mut CHAR, xSrc, ySrc, cySrc);
                        _WritePixel(pvDst0 as *mut CHAR, xDst, yDst, cjDstRow, color);
                        xDst = xDst.wrapping_add(1);
                    }
                    yDst = yDst.wrapping_add(1);
                }
                // break;
            }

            270 => {
                yDst = 0;
                while yDst < cyDst {
                    xSrc = cyDst.wrapping_sub(yDst).wrapping_div(yScale as ULONG);
                    xDst = 0;
                    while xDst < cxDst {
                        ySrc = xDst.wrapping_div(xScale as ULONG);
                        color = _ReadPixel(pvSrc0 as *mut CHAR, xSrc, ySrc, cySrc);
                        _WritePixel(pvDst0 as *mut CHAR, xDst, yDst, cjDstRow, color);
                        xDst = xDst.wrapping_add(1);
                    }
                    yDst = yDst.wrapping_add(1);
                }
                // break;
            }

            0 | _ => {
                yDst = 0;
                while yDst < cyDst {
                    ySrc = yDst.wrapping_div(yScale as ULONG);
                    xDst = 0;
                    while xDst < cxDst {
                        xSrc = xDst.wrapping_div(xScale as ULONG);
                        color = _ReadPixel(pvSrc0 as *mut CHAR, xSrc, ySrc, cySrc);
                        _WritePixel(pvDst0 as *mut CHAR, xDst, yDst, cjDstRow, color);
                        xDst = xDst.wrapping_add(1);
                    }
                    yDst = yDst.wrapping_add(1);
                }
            }
        }
    }

    /* Return the size of the GLYPHBITS structure */
    return (FIELD_OFFSET!(GLYPHBITS, aj) as ULONG)
        .wrapping_add(cyDst.wrapping_mul(cjDstRow));
}
#[no_mangle]
pub unsafe extern "C" fn BmfdQueryMaxExtents(
    pfont: PBMFD_FONT,
    pfddm: PFD_DEVICEMETRICS,
    cjSize: ULONG,
) -> ULONG {
    let mut cjMaxWidth: ULONG = 0;
    let mut cjMaxBitmapSize: ULONG = 0;
    let mut pFontInfo: PFONTINFO16 = core::ptr::null_mut();
    let mut xScale: ULONG = 0;
    let mut yScale: ULONG = 0;

    if !pfddm.is_null() {
        if cjSize < core::mem::size_of::<FD_DEVICEMETRICS>() as ULONG {
            /* Not enough space, fail */
            return FD_ERROR;
        }

        pFontInfo = (*(*pfont).pface).pFontInfo;

        xScale = ((*pfont).xScale) as u32;
        yScale = ((*pfont).yScale) as u32;

        /* Fill FD_DEVICEMETRICS */
        (*pfddm).flRealizedType = FDM_MASK;
        (*pfddm).pteBase.x = FLOATL_1;
        (*pfddm).pteBase.y = 0 as FLOATL;
        (*pfddm).pteSide.x = 0 as FLOATL;
        (*pfddm).pteSide.y = FLOATL_1;
        (*pfddm).ptlUnderline1.x = 0;
        (*pfddm).ptlUnderline1.y = 1;
        (*pfddm).ptlStrikeout.x = 0;
        (*pfddm).ptlStrikeout.y = -4;
        (*pfddm).ptlULThickness.x = 0;
        (*pfddm).ptlULThickness.y = 1;
        (*pfddm).ptlSOThickness.x = 0;
        (*pfddm).ptlSOThickness.y = 1;
        (*pfddm).lMinA = 0;
        (*pfddm).lMinC = 0;
        (*pfddm).lMinD = 0;

        if (*pfont).ulAngle == 90 || (*pfont).ulAngle == 270 {
            (*pfddm).cxMax = xScale.wrapping_mul(GETVAL!((*pFontInfo).dfPixHeight) as ULONG);
            (*pfddm).cyMax = yScale.wrapping_mul(GETVAL!((*pFontInfo).dfMaxWidth) as ULONG);
            (*pfddm).fxMaxAscender = (yScale.wrapping_mul(GETVAL!((*pFontInfo).dfAscent) as ULONG) << 4) as i32;
            (*pfddm).fxMaxDescender = ((*pfddm).cyMax << 4).wrapping_sub((*pfddm).fxMaxAscender as ULONG) as FIX;
        } else {
            (*pfddm).cxMax = xScale.wrapping_mul(GETVAL!((*pFontInfo).dfMaxWidth) as ULONG);
            (*pfddm).cyMax = yScale.wrapping_mul(GETVAL!((*pFontInfo).dfPixHeight) as ULONG);
            (*pfddm).fxMaxAscender = (yScale.wrapping_mul(GETVAL!((*pFontInfo).dfAscent) as ULONG) << 4) as i32;
            (*pfddm).fxMaxDescender = ((*pfddm).cyMax << 4).wrapping_sub((*pfddm).fxMaxAscender as ULONG) as FIX;
        }

        (*pfddm).lD = ((*pfddm).cxMax) as i32;

        /* Calculate Width in bytes */
        cjMaxWidth = ((*pfddm).cxMax + 7) >> 3;

        /* Calculate size of the bitmap, rounded to DWORDs */
        cjMaxBitmapSize = ((cjMaxWidth.wrapping_mul((*pfddm).cyMax)) + 3)
            & !3;

        /* cjGlyphMax is the full size of the GLYPHBITS structure */
        (*pfddm).cjGlyphMax = (FIELD_OFFSET!(GLYPHBITS, aj).wrapping_add((cjMaxBitmapSize) as i32)) as u32;

        /* NOTE: fdxQuantized and NonLinear... stay unchanged */
    }

    /* Return the size of the structure */
    return core::mem::size_of::<FD_DEVICEMETRICS>() as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn BmfdQueryGlyphAttrs(pfo: *mut FONTOBJ, iMode: ULONG) -> PFD_GLYPHATTR {
    // DbgPrint("BmfdQueryGlyphAttrs()\n");
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "system" fn BmfdQueryFontData(dhpdev: DHPDEV, pfo: *mut FONTOBJ, iMode: ULONG, hg: HGLYPH, pgd: *mut GLYPHDATA, pv: PVOID, cjSize: ULONG) -> LONG {
    let pfile: PBMFD_FILE = (*pfo).iFile as PBMFD_FILE;
    let pface: PBMFD_FACE = ((&raw mut (*pfile).aface) as *mut BMFD_FACE)
        .offset((*pfo).iFace.wrapping_sub(1 as ULONG) as isize)
        as PBMFD_FACE;
    let pfont: PBMFD_FONT = BmfdGetFontInstance(pfo, pface);

    DbgPrint(
        b"BmfdQueryFontData(pfo=%p, iMode=%ld, hg=%p, pgd=%p, pv=%p, cjSize=%ld)\n\0" as *const u8
            as *const ::core::ffi::c_char as PCHAR,
        pfo,
        iMode,
        hg,
        pgd,
        pv,
        cjSize,
    );

    match iMode {
        QFD_GLYPHANDBITMAP => {
            return BmfdQueryGlyphAndBitmap(pfont, hg, pgd, pv as *mut GLYPHBITS, cjSize) as LONG;
        }
        QFD_MAXEXTENTS => {
            return BmfdQueryMaxExtents(pfont, pv as PFD_DEVICEMETRICS, cjSize) as LONG;
        }
        _ => {
            return FD_ERROR as LONG;
        }
    };

    return FD_ERROR as LONG;
}
