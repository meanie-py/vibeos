use win32k_sys::win32k::*;

use crate::dib_helpers::*;

extern "C" {
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
pub const DEC_OR_INC: () = ();

#[no_mangle]
pub unsafe extern "C" fn DIB_8BPP_PutPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32, c: u32) {
    let byteaddr = ((*SurfObj).pvScan0 as PBYTE).offset((y * (*SurfObj).lDelta + x) as isize);
    *byteaddr = c as u8;
}

#[no_mangle]
pub unsafe extern "C" fn DIB_8BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32 {
    let byteaddr = ((*SurfObj).pvScan0 as PBYTE)
        .offset((y as isize) * ((*SurfObj).lDelta as isize) + (x as isize));
    
    return (*byteaddr) as u32;
}

#[no_mangle]
pub unsafe extern "C" fn DIB_8BPP_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32) {
    if x1 >= x2 {
        return;
    }
    
    
    let dest = ((*SurfObj).pvScan0 as PBYTE)
        .offset((y * (*SurfObj).lDelta + x1) as isize);
    memset(dest as *mut core::ffi::c_void, c as i32, (x2 - x1) as usize);
}

#[no_mangle]
pub unsafe extern "C" fn DIB_8BPP_VLine(SurfObj: *mut SURFOBJ, x: i32, y1: i32, y2: i32, c: u32) {
    let byteaddr = ((*SurfObj).pvScan0 as PBYTE).offset((y1 * (*SurfObj).lDelta) as isize);
    let mut addr = byteaddr.offset(x as isize);
    let lDelta = (*SurfObj).lDelta;
    let mut y1 = y1;

    while { y1 += 1; y1 - 1 } < y2 {
        *addr = c as u8;
        addr = addr.offset(lDelta as isize);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_8BPP_BitBltSrcCopy(BltInfo: PBLTINFO) -> u8 {
    let mut i: i32;
    let mut j: i32;
    let mut sx: i32;
    let mut sy: i32;
    let mut xColor: i32;
    let mut f1: i32;
    let mut SourceBits: PBYTE;
    let mut DestBits: PBYTE;
    let mut SourceLine: PBYTE;
    let mut DestLine: PBYTE;
    let mut SourceBits_4BPP: PBYTE;
    let mut SourceLine_4BPP: PBYTE;
    let bTopToBottom: u8;
    let bLeftToRight: u8;

    let bLeftToRight = if (*BltInfo).DestRect.left > (*BltInfo).DestRect.right { 1u8 } else { 0u8 };
    let bTopToBottom = if (*BltInfo).DestRect.top > (*BltInfo).DestRect.bottom { 1u8 } else { 0u8 };

    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!((*BltInfo).DestRect));

    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
        .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
        .offset((*BltInfo).DestRect.left as isize);

    match (*(*BltInfo).SourceSurface).iBitmapFormat as u32 {
        BMF_1BPP => {
            sx = (*BltInfo).SourcePoint.x;
            sy = (*BltInfo).SourcePoint.y;

            if bTopToBottom != 0 {
                sy += (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1;
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    if DIB_1BPP_GetPixel((*BltInfo).SourceSurface, sx, sy) == 0 {
                        DIB_8BPP_PutPixel((*BltInfo).DestSurface, i, j, XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, 0));
                    } else {
                        DIB_8BPP_PutPixel((*BltInfo).DestSurface, i, j, XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, 1));
                    }
                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                sy = if bTopToBottom != 0 { sy - 1 } else { sy + 1 };
                j += 1;
            }
        }

        BMF_4BPP => {
            SourceBits_4BPP = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset(((*BltInfo).SourcePoint.x >> 1) as isize);

            if bTopToBottom != 0 {
                SourceBits_4BPP = SourceBits_4BPP.offset((((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize);
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceLine_4BPP = SourceBits_4BPP;
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                f1 = sx & 1;

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, ((*SourceLine_4BPP & altnotmask[f1 as usize]) >> (4 * (1 - f1))) as u32) as i32;
                    DIB_8BPP_PutPixel((*BltInfo).DestSurface, i, j, xColor as u32);
                    if f1 == 1 {
                        SourceLine_4BPP = if bLeftToRight != 0 { SourceLine_4BPP.offset(-1) } else { SourceLine_4BPP.offset(1) };
                        f1 = 0;
                    } else {
                        f1 = 1;
                    }

                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                SourceBits_4BPP = if bTopToBottom != 0 { SourceBits_4BPP.offset(-((*(*BltInfo).SourceSurface).lDelta as isize)) } else { SourceBits_4BPP.offset((*(*BltInfo).SourceSurface).lDelta as isize) };
                j += 1;
            }
        }

        BMF_8BPP => {
            if ((*BltInfo).XlateSourceToDest.is_null() || ((*(*BltInfo).XlateSourceToDest).flXlate & XO_TRIVIAL) != 0) && (bTopToBottom == 0 && bLeftToRight == 0) {
                if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((*BltInfo).SourcePoint.x as isize);
                    j = (*BltInfo).DestRect.top;
                    while j < (*BltInfo).DestRect.bottom {
                        core::ptr::copy_nonoverlapping(SourceBits, DestBits, ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left) as usize);
                        SourceBits = SourceBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                        DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                        j += 1;
                    }
                } else {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((*BltInfo).SourcePoint.x as isize);
                    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                        .offset((*BltInfo).DestRect.left as isize);
                    j = (*BltInfo).DestRect.bottom - 1;
                    while (*BltInfo).DestRect.top <= j {
                        core::ptr::copy_nonoverlapping(SourceBits, DestBits, ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left) as usize);
                        SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                        DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                        j -= 1;
                    }
                }
            } else {
                if bTopToBottom == 0 && bLeftToRight == 0 {
                    if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                        SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                            .offset((*BltInfo).SourcePoint.x as isize);
                        DestLine = DestBits;
                        j = (*BltInfo).DestRect.top;
                        while j < (*BltInfo).DestRect.bottom {
                            SourceBits = SourceLine;
                            DestBits = DestLine;
                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                *DestBits = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u8;
                                DestBits = DestBits.offset(1);
                                SourceBits = SourceBits.offset(1);
                                i += 1;
                            }
                            SourceLine = SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                            DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                            j += 1;
                        }
                    } else {
                        SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                            .offset((*BltInfo).SourcePoint.x as isize);
                        DestLine = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((*BltInfo).DestRect.left as isize);
                        j = (*BltInfo).DestRect.bottom - 1;
                        while (*BltInfo).DestRect.top <= j {
                            SourceBits = SourceLine;
                            DestBits = DestLine;
                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                *DestBits = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u8;
                                DestBits = DestBits.offset(1);
                                SourceBits = SourceBits.offset(1);
                                i += 1;
                            }
                            SourceLine = SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                            DestLine = DestLine.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                            j -= 1;
                        }
                    }
                } else {
                    let mut OneDone: u8 = 0u8;

                    if bLeftToRight != 0 {
                        let store = ExAllocatePoolWithTag(NonPagedPool as u32, ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left + 1) as SIZE_T, TAG_DIB) as *mut u8;
                        if store.is_null() {
                            return 0u8;
                        }
                        let mut Index: u16;

                        SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                            .offset((*BltInfo).SourcePoint.x as isize);

                        DestLine = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((*BltInfo).DestRect.left as isize);

                        j = (*BltInfo).DestRect.top;
                        while j < (*BltInfo).DestRect.bottom {
                            SourceBits = SourceLine;
                            DestBits = DestLine;

                            SourceBits = SourceBits.offset(((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) as isize);

                            Index = 0;

                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                *store.offset(Index as isize) = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u8;
                                SourceBits = SourceBits.offset(-1);
                                Index += 1;
                                i += 1;
                            }

                            Index = 0;

                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                *DestBits = *store.offset(Index as isize);
                                DestBits = DestBits.offset(1);
                                Index += 1;
                                i += 1;
                            }

                            SourceLine = SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                            DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                            j += 1;
                        }
                        ExFreePoolWithTag(store as *mut core::ffi::c_void, TAG_DIB);
                        OneDone = 1u8;
                    }

                    if bTopToBottom != 0 {
                        let mut Index: u32;

                        let store = ExAllocatePoolWithTag(NonPagedPool as u32, ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top + 1) as SIZE_T, TAG_DIB) as *mut u8;
                        if store.is_null() {
                            return 0u8;
                        }

                        if OneDone != 0 {
                            SourceLine = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                                .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
                                .offset((*BltInfo).DestRect.left as isize)
                                .offset((((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).DestSurface).lDelta) as isize);
                        } else {
                            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                                .offset((((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                                .offset((*BltInfo).SourcePoint.x as isize);
                        }

                        // When OneDone, SourceLine was re-pointed at the DEST surface above,
                        // so the row stride to walk it by is the DEST's. Reading the source
                        // surface's lDelta here walks the wrong pitch (upstream ae0e4ee305).
                        let SrcDelta = if OneDone != 0 {
                            (*(*BltInfo).DestSurface).lDelta
                        } else {
                            (*(*BltInfo).SourceSurface).lDelta
                        };

                        DestLine = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((*BltInfo).DestRect.left as isize);

                        i = (*BltInfo).DestRect.left;
                        while i < (*BltInfo).DestRect.right {
                            DestBits = DestLine;
                            SourceBits = SourceLine;

                            Index = 0;

                            j = (*BltInfo).DestRect.top;
                            while j < (*BltInfo).DestRect.bottom {
                                // Already dest-format on the OneDone pass; translating twice
                                // double-maps the palette (upstream ae0e4ee305).
                                *store.offset(Index as isize) = if OneDone != 0 {
                                    *SourceBits
                                } else {
                                    XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u8
                                };
                                SourceBits = SourceBits.offset(-(SrcDelta as isize));
                                Index += 1;
                                j += 1;
                            }

                            Index = 0;

                            j = (*BltInfo).DestRect.top;
                            while j < (*BltInfo).DestRect.bottom {
                                *DestBits = *store.offset(Index as isize);
                                DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                                Index += 1;
                                j += 1;
                            }
                            SourceLine = SourceLine.offset(1);
                            DestLine = DestLine.offset(1);
                            i += 1;
                        }
                        ExFreePoolWithTag(store as *mut core::ffi::c_void, TAG_DIB);
                    }
                }
            }
        }

        BMF_16BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((2 * (*BltInfo).SourcePoint.x) as isize);

            if bTopToBottom != 0 {
                SourceLine = SourceLine.offset((((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize);
            }
            DestLine = DestBits;

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset((((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) * 2) as isize);
                }
                DestBits = DestLine;

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = *(SourceBits as PWORD) as i32;
                    *DestBits = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, xColor as u32) as u8;

                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-2) } else { SourceBits.offset(2) };
                    DestBits = DestBits.offset(1);
                    i += 1;
                }
                SourceLine = if bTopToBottom != 0 { SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize)) } else { SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize) };
                DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        BMF_24BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((3 * (*BltInfo).SourcePoint.x) as isize);
            DestLine = DestBits;

            if bTopToBottom != 0 {
                SourceLine = SourceLine.offset(((*(*BltInfo).SourceSurface).lDelta * ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1)) as isize);
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;
                DestBits = DestLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset((((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) * 3) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = ((*SourceBits.offset(2) as i32) << 0x10) + ((*SourceBits.offset(1) as i32) << 0x08) + (*SourceBits as i32);
                    *DestBits = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, xColor as u32) as u8;
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-3) } else { SourceBits.offset(3) };
                    DestBits = DestBits.offset(1);
                    i += 1;
                }
                SourceLine = if bTopToBottom != 0 { SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize)) } else { SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize) };
                DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        BMF_32BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((4 * (*BltInfo).SourcePoint.x) as isize);

            if bTopToBottom != 0 {
                SourceLine = SourceLine.offset(((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) as isize);
            }
            DestLine = DestBits;

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;
                DestBits = DestLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset((((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) * 4) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = *(SourceBits as PDWORD) as i32;
                    *DestBits = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, xColor as u32) as u8;

                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-4) } else { SourceBits.offset(4) };
                    DestBits = DestBits.offset(1);
                    i += 1;
                }
                SourceLine = if bTopToBottom != 0 { SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize)) } else { SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize) };
                DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        _ => {
            return 0u8;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_8BPP_ColorFill(DestSurface: *mut SURFOBJ, DestRect: *mut RECTL, color: u32) -> u8 {

    RECTL_vMakeWellOrdered(DestRect);

    let mut DestY = (*DestRect).top;
    while DestY < (*DestRect).bottom {
        DIB_8BPP_HLine(DestSurface, (*DestRect).left, (*DestRect).right, DestY, color);
        DestY += 1;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_8BPP_TransparentBlt(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ColorTranslation: *mut XLATEOBJ,
    iTransColor: u32,
) -> u8 {
    let mut RoundedRight: i32;
    let mut X: i32;
    let mut Y: i32;
    let mut SourceX: i32 = 0;
    let mut SourceY: i32 = 0;
    let mut DestBits: *mut u32;
    let mut Source: u32;
    let mut Dest: u32;

    let DstHeight: i32 = (*DestRect).bottom - (*DestRect).top;
    let DstWidth: i32 = (*DestRect).right - (*DestRect).left;
    let SrcHeight: i32 = (*SourceRect).bottom - (*SourceRect).top;
    let SrcWidth: i32 = (*SourceRect).right - (*SourceRect).left;

    RoundedRight = (*DestRect).right - (((*DestRect).right - (*DestRect).left) & 0x3);
    DestBits = ((*DestSurf).pvScan0 as PBYTE).offset((*DestRect).left as isize)
        .offset(((*DestRect).top * (*DestSurf).lDelta) as isize) as *mut u32;

    Y = (*DestRect).top;
    while Y < (*DestRect).bottom {
        DestBits = ((*DestSurf).pvScan0 as PBYTE).offset((*DestRect).left as isize)
            .offset((Y * (*DestSurf).lDelta) as isize) as *mut u32;
        SourceY = (*SourceRect).top + (Y - (*DestRect).top) * SrcHeight / DstHeight;
        
        X = (*DestRect).left;
        while X < RoundedRight {
            Dest = *DestBits;

            SourceX = (*SourceRect).left + (X - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0 && SourceY >= 0 &&
                (*SourceSurf).sizlBitmap.cx > SourceX && (*SourceSurf).sizlBitmap.cy > SourceY {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if Source != iTransColor {
                    Dest &= 0xFFFFFF00;
                    Dest |= XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFF;
                }
            }

            SourceX = (*SourceRect).left + (X + 1 - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0 && SourceY >= 0 &&
                (*SourceSurf).sizlBitmap.cx > SourceX && (*SourceSurf).sizlBitmap.cy > SourceY {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if Source != iTransColor {
                    Dest &= 0xFFFF00FF;
                    Dest |= (XLATEOBJ_iXlate(ColorTranslation, Source) << 8) & 0xFF00;
                }
            }

            SourceX = (*SourceRect).left + (X + 2 - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0 && SourceY >= 0 &&
                (*SourceSurf).sizlBitmap.cx > SourceX && (*SourceSurf).sizlBitmap.cy > SourceY {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if Source != iTransColor {
                    Dest &= 0xFF00FFFF;
                    Dest |= (XLATEOBJ_iXlate(ColorTranslation, Source) << 16) & 0xFF0000;
                }
            }

            SourceX = (*SourceRect).left + (X + 3 - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0 && SourceY >= 0 &&
                (*SourceSurf).sizlBitmap.cx > SourceX && (*SourceSurf).sizlBitmap.cy > SourceY {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if Source != iTransColor {
                    Dest &= 0x00FFFFFF;
                    Dest |= (XLATEOBJ_iXlate(ColorTranslation, Source) << 24) & 0xFF000000;
                }
            }

            *DestBits = Dest;
            
            X += 4;
            DestBits = DestBits.offset(1);
        }

        if X < (*DestRect).right {
            while X < (*DestRect).right {
                SourceX = (*SourceRect).left + (X - (*DestRect).left) * SrcWidth / DstWidth;
                if SourceX >= 0 && SourceY >= 0 &&
                    (*SourceSurf).sizlBitmap.cx > SourceX && (*SourceSurf).sizlBitmap.cy > SourceY {
                    Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                    if Source != iTransColor {
                        *(DestBits as *mut u8) = (XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFF) as u8;
                    }
                }
                DestBits = (DestBits as usize + 1) as *mut u32;
                X += 1;
            }
        }
        
        Y += 1;
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Win32 subsystem
//  * LICENSE:         See COPYING in the top level directory
//  * FILE:            win32ss/gdi/dib/dib8bpp.c
//  * PURPOSE:         Device Independant Bitmap functions, 8bpp
//  * PROGRAMMERS:     Jason Filby
//  *                  Thomas Bluemel
//  *                  Gregor Anich
//  *                  Doug Lyons
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define DEC_OR_INC(var, decTrue, amount) \
//     ((var) = (decTrue) ? ((var) - (amount)) : ((var) + (amount)))
//
// VOID
// DIB_8BPP_PutPixel(SURFOBJ *SurfObj, LONG x, LONG y, ULONG c)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta + x;
//
//   *byteaddr = (BYTE)c;
// }
//
// ULONG
// DIB_8BPP_GetPixel(SURFOBJ *SurfObj, LONG x, LONG y)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta + x;
//
//   return (ULONG)(*byteaddr);
// }
//
// VOID
// DIB_8BPP_HLine(SURFOBJ *SurfObj, LONG x1, LONG x2, LONG y, ULONG c)
// {
//   if (x1 >= x2)
//     return;
//
//   memset((PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta + x1, (BYTE) c, x2 - x1);
// }
//
// VOID
// DIB_8BPP_VLine(SURFOBJ *SurfObj, LONG x, LONG y1, LONG y2, ULONG c)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y1 * SurfObj->lDelta;
//   PBYTE addr = byteaddr + x;
//   LONG lDelta = SurfObj->lDelta;
//
//   byteaddr = addr;
//   while(y1++ < y2)
//   {
//     *addr = (BYTE)c;
//
//     addr += lDelta;
//   }
// }
//
// BOOLEAN
// DIB_8BPP_BitBltSrcCopy(PBLTINFO BltInfo)
// {
//   LONG     i, j, sx, sy, xColor, f1;
//   PBYTE    SourceBits, DestBits, SourceLine, DestLine;
//   PBYTE    SourceBits_4BPP, SourceLine_4BPP;
//   BOOLEAN  bTopToBottom, bLeftToRight;
//
//   DPRINT("DIB_8BPP_BitBltSrcCopy: SrcSurf cx/cy (%d/%d), DestSuft cx/cy (%d/%d) dstRect: (%d,%d)-(%d,%d)\n",
//          BltInfo->SourceSurface->sizlBitmap.cx, BltInfo->SourceSurface->sizlBitmap.cy,
//          BltInfo->DestSurface->sizlBitmap.cx, BltInfo->DestSurface->sizlBitmap.cy,
//          BltInfo->DestRect.left, BltInfo->DestRect.top, BltInfo->DestRect.right, BltInfo->DestRect.bottom);
//
//   /* Get back left to right flip here */
//   bLeftToRight = (BltInfo->DestRect.left > BltInfo->DestRect.right);
//
//   /* Check for top to bottom flip needed. */
//   bTopToBottom = BltInfo->DestRect.top > BltInfo->DestRect.bottom;
//
//   DPRINT("bTopToBottom is '%d' and bLeftToRight is '%d'.\n", bTopToBottom, bLeftToRight);
//
//   /* Make WellOrdered by making top < bottom and left < right */
//   RECTL_vMakeWellOrdered(&BltInfo->DestRect);
//
//   DPRINT("BPP is '%d' & BltInfo->SourcePoint.x is '%d' & BltInfo->SourcePoint.y is '%d'.\n",
//          BltInfo->SourceSurface->iBitmapFormat, BltInfo->SourcePoint.x, BltInfo->SourcePoint.y);
//
//   DestBits = (PBYTE)BltInfo->DestSurface->pvScan0 + (BltInfo->DestRect.top * BltInfo->DestSurface->lDelta) + BltInfo->DestRect.left;
//
//   switch(BltInfo->SourceSurface->iBitmapFormat)
//   {
//     case BMF_1BPP:
//       DPRINT("1BPP Case Selected with DestRect Width of '%d'.\n",
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       sx = BltInfo->SourcePoint.x;
//
//       /* This sets sy to the top line */
//       sy = BltInfo->SourcePoint.y;
//
//       if (bTopToBottom)
//       {
//         /* This sets sy to the bottom line */
//         sy += BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1;
//       }
//
//       for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//       {
//         sx = BltInfo->SourcePoint.x;
//
//         if (bLeftToRight)
//         {
//           /* This sets the sx to the rightmost pixel */
//           sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//         }
//
//         for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//         {
//           if(DIB_1BPP_GetPixel(BltInfo->SourceSurface, sx, sy) == 0)
//           {
//             DIB_8BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 0));
//           }
//           else
//           {
//             DIB_8BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 1));
//           }
//           DEC_OR_INC(sx, bLeftToRight, 1);
//         }
//         DEC_OR_INC(sy, bTopToBottom, 1);
//       }
//       break;
//
//     case BMF_4BPP:
//       DPRINT("4BPP Case Selected with DestRect Width of '%d'.\n",
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       /* This sets SourceBits_4BPP to the top line */
//       SourceBits_4BPP = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + (BltInfo->SourcePoint.x >> 1);
//
//       if (bTopToBottom)
//       {
//         /* This sets SourceBits_4BPP to the bottom line */
//         SourceBits_4BPP += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;
//       }
//
//       for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//       {
//         SourceLine_4BPP = SourceBits_4BPP;
//         sx = BltInfo->SourcePoint.x;
//
//         if (bLeftToRight)
//         {
//           /* This sets sx to the rightmost pixel */
//           sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//         }
//
//         f1 = sx & 1;
//
//         for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//         {
//           xColor = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest,
//               (*SourceLine_4BPP & altnotmask[f1]) >> (4 * (1 - f1)));
//           DIB_8BPP_PutPixel(BltInfo->DestSurface, i, j, xColor);
//           if(f1 == 1)
//           {
//             DEC_OR_INC(SourceLine_4BPP, bLeftToRight, 1);
//             f1 = 0;
//           }
//           else
//           {
//             f1 = 1;
//           }
//
//           DEC_OR_INC(sx, bLeftToRight, 1);
//         }
//         DEC_OR_INC(SourceBits_4BPP, bTopToBottom, BltInfo->SourceSurface->lDelta);
//       }
//       break;
//
//     case BMF_8BPP:
//       DPRINT("8BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//               BltInfo->DestRect.left, BltInfo->DestRect.top,
//               BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//               BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       if ((BltInfo->XlateSourceToDest == NULL ||
//         (BltInfo->XlateSourceToDest->flXlate & XO_TRIVIAL) != 0) &&
//         (!bTopToBottom && !bLeftToRight))
//       {
//         if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//         {
//           DPRINT("BltInfo->DestRect.top < BltInfo->SourcePoint.y.\n");
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + BltInfo->SourcePoint.x;
//           for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//           {
//             RtlMoveMemory(DestBits, SourceBits, BltInfo->DestRect.right - BltInfo->DestRect.left);
//             SourceBits += BltInfo->SourceSurface->lDelta;
//             DestBits += BltInfo->DestSurface->lDelta;
//           }
//         }
//         else
//         {
//           DPRINT("BltInfo->DestRect.top >= BltInfo->SourcePoint.y.\n");
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0 + ((BltInfo->SourcePoint.y + BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta) + BltInfo->SourcePoint.x;
//           DestBits = (PBYTE)BltInfo->DestSurface->pvScan0 + ((BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta) + BltInfo->DestRect.left;
//           for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//           {
//             RtlMoveMemory(DestBits, SourceBits, BltInfo->DestRect.right - BltInfo->DestRect.left);
//             SourceBits -= BltInfo->SourceSurface->lDelta;
//             DestBits -= BltInfo->DestSurface->lDelta;
//           }
//         }
//       }
//       else
//       {
//         DPRINT("XO_TRIVIAL is NOT TRUE or we have flips.\n");
//
//         if (!bTopToBottom && !bLeftToRight)
//       /* **Note: Indent is purposefully less than desired to keep reviewable differences to a minimum for PR** */
//         {
//         if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//         {
//           DPRINT("Dest.top < SourcePoint.y.\n");
//           SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + BltInfo->SourcePoint.x;
//           DestLine = DestBits;
//           for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//           {
//             SourceBits = SourceLine;
//             DestBits = DestLine;
//             for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//             {
//               *DestBits++ = (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *SourceBits++);
//             }
//             SourceLine += BltInfo->SourceSurface->lDelta;
//             DestLine += BltInfo->DestSurface->lDelta;
//           }
//         }
//         else
//         {
//           DPRINT("Dest.top >= SourcePoint.y.\n");
//           SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 + ((BltInfo->SourcePoint.y + BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta) + BltInfo->SourcePoint.x;
//           DestLine = (PBYTE)BltInfo->DestSurface->pvScan0 + ((BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta) + BltInfo->DestRect.left;
//           for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//           {
//             SourceBits = SourceLine;
//             DestBits = DestLine;
//             for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//             {
//               *DestBits++ = (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *SourceBits++);
//             }
//             SourceLine -= BltInfo->SourceSurface->lDelta;
//             DestLine -= BltInfo->DestSurface->lDelta;
//           }
//         }
//       }
//       else
//       {
//           /* Buffering for source and destination flip overlaps. Fixes KHMZ MirrorTest CORE-16642 */
//           BOOL OneDone = FALSE;
//
//           if (bLeftToRight)
//           {
//             DPRINT("Flip is bLeftToRight.\n");
//
//             /* Allocate enough pixels for a row in BYTE's */
//             BYTE *store = ExAllocatePoolWithTag(NonPagedPool,
//               BltInfo->DestRect.right - BltInfo->DestRect.left + 1, TAG_DIB);
//             if (store == NULL)
//             {
//               DPRINT1("Storage Allocation Failed.\n");
//               return FALSE;
//             }
//             WORD  Index;
//
//             /* This sets SourceLine to the top line */
//             SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//               (BltInfo->SourcePoint.y *
//               BltInfo->SourceSurface->lDelta) +
//               BltInfo->SourcePoint.x;
//
//             /* This set the DestLine to the top line */
//             DestLine = (PBYTE)BltInfo->DestSurface->pvScan0 +
//               (BltInfo->DestRect.top * BltInfo->DestSurface->lDelta) +
//               BltInfo->DestRect.left;
//
//             for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//             {
//               SourceBits = SourceLine;
//               DestBits = DestLine;
//
//               /* This sets SourceBits to the rightmost pixel */
//               SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//
//               Index = 0;
//
//               for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//               {
//                 store[Index] = (BYTE)XLATEOBJ_iXlate(
//                   BltInfo->XlateSourceToDest,
//                   *SourceBits);
//                 SourceBits--;
//                 Index++;
//               }
//
//               Index = 0;
//
//               for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//               {
//                 *DestBits = store[Index];
//                 DestBits++;
//                 Index++;
//               }
//
//               SourceLine += BltInfo->SourceSurface->lDelta;
//               DestLine += BltInfo->DestSurface->lDelta;
//             }
//             ExFreePoolWithTag(store, TAG_DIB);
//             OneDone = TRUE;
//           }
//
//           if (bTopToBottom)
//           {
//             DPRINT("Flip is bTopToBottom.\n");
//
//             DWORD  Index;
//
//             /* Allocate enough pixels for a column in BYTE's */
//             BYTE *store = ExAllocatePoolWithTag(NonPagedPool,
//               BltInfo->DestRect.bottom - BltInfo->DestRect.top + 1, TAG_DIB);
//             if (store == NULL)
//             {
//               DPRINT1("Storage Allocation Failed.\n");
//               return FALSE;
//             }
//
//             /* The OneDone flag indicates that we are flipping for bTopToBottom and bLeftToRight
//              * and have already completed the bLeftToRight. So we will lose our first flip output
//              * unless we work with its output which is at the destination site. So in this case
//              * our new Source becomes the previous outputs Destination. */
//
//             if (OneDone)
//             {
//               /* This sets SourceLine to the bottom line of our previous destination */
//               SourceLine = (PBYTE)BltInfo->DestSurface->pvScan0 +
//                 (BltInfo->DestRect.top * BltInfo->DestSurface->lDelta) + BltInfo->DestRect.left  +
//                 (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->DestSurface->lDelta;
//             }
//             else
//             {
//               /* This sets SourceLine to the bottom line */
//               SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//                 ((BltInfo->SourcePoint.y + BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1)
//                 * BltInfo->SourceSurface->lDelta) +
//                 BltInfo->SourcePoint.x;
//             }
//
//             /* This set the DestLine to the top line */
//             DestLine = (PBYTE)BltInfo->DestSurface->pvScan0 +
//               (BltInfo->DestRect.top * BltInfo->DestSurface->lDelta) +
//               BltInfo->DestRect.left;
//
//             /* Read columns */
//             for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//             {
//
//               DestBits = DestLine;
//               SourceBits = SourceLine;
//
//               Index = 0;
//
//               /* Read up the column and store the pixels */
//               for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//               {
//                 store[Index] = (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *SourceBits);
//                 /* Go up a line */
//                 SourceBits -= BltInfo->SourceSurface->lDelta;
//                 Index++;
//               }
//
//               Index = 0;
//
//               /* Get the stored pixel and copy then down the column */
//               for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//               {
//                 *DestBits = store[Index];
//                 /* Go down a line */
//                 DestBits += BltInfo->SourceSurface->lDelta;
//                 Index++;
//               }
//               /* Index to next column */
//               SourceLine += 1;
//               DestLine += 1;
//             }
//             ExFreePoolWithTag(store, TAG_DIB);
//           }
//
//         }
//       }
//       break;
//
//     case BMF_16BPP:
//       DPRINT("16BPP Case Selected with DestRect Width of '%d'.\n",
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       DPRINT("BMF_16BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//              BltInfo->DestRect.left, BltInfo->DestRect.top,
//              BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + 2 * BltInfo->SourcePoint.x;
//
//       if (bTopToBottom)
//       {
//         /* This sets SourceLine to the bottom line */
//         SourceLine += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;;
//       }
//       DestLine = DestBits;
//
//       for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//       {
//         SourceBits = SourceLine;
//
//         if (bLeftToRight)
//         {
//           /* This sets SourceBits to the rightmost pixel */
//           SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1) * 2;
//         }
//         DestBits = DestLine;
//
//         for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//         {
//           xColor = *((PWORD) SourceBits);
//           *DestBits = (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, xColor);
//
//           DEC_OR_INC(SourceBits, bLeftToRight, 2);
//           DestBits += 1;
//         }
//         DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//         DestLine += BltInfo->DestSurface->lDelta;
//       }
//       break;
//
//     case BMF_24BPP:
//       DPRINT("24BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//              BltInfo->DestRect.left, BltInfo->DestRect.top,
//              BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + 3 * BltInfo->SourcePoint.x;
//       DestLine = DestBits;
//
//       if (bTopToBottom)
//       {
//         /* This sets SourceLine to the bottom line */
//         SourceLine += BltInfo->SourceSurface->lDelta * (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//       }
//
//       for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//       {
//         SourceBits = SourceLine;
//         DestBits = DestLine;
//
//         if (bLeftToRight)
//         {
//           /* This sets the SourceBits to the rightmost pixel */
//           SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1) * 3;
//         }
//
//         for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//         {
//           xColor = (*(SourceBits + 2) << 0x10) +
//              (*(SourceBits + 1) << 0x08) +
//              (*(SourceBits));
//           *DestBits = (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, xColor);
//           DEC_OR_INC(SourceBits, bLeftToRight, 3);
//           DestBits += 1;
//         }
//         DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//         DestLine += BltInfo->DestSurface->lDelta;
//       }
//       break;
//
//     case BMF_32BPP:
//       DPRINT("32BPP Case Selected with DestRect Width of '%d'.\n",
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + 4 * BltInfo->SourcePoint.x;
//
//       if (bTopToBottom)
//       {
//         /* This sets SourceLine to the bottom line */
//         SourceLine += BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1;
//       }
//       DestLine = DestBits;
//
//       for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//       {
//         SourceBits = SourceLine;
//         DestBits = DestLine;
//
//         if (bLeftToRight)
//         {
//           /* This sets SourceBits to the rightmost pixel */
//           SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1) * 4;
//         }
//
//         for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//         {
//           xColor = *((PDWORD) SourceBits);
//           *DestBits = (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, xColor);
//
//           DEC_OR_INC(SourceBits, bLeftToRight, 4);
//           DestBits += 1;
//         }
//         DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//         DestLine += BltInfo->DestSurface->lDelta;
//       }
//       break;
//
//     default:
//       DPRINT1("DIB_8BPP_Bitblt: Unhandled Source BPP: %u\n", BitsPerFormat(BltInfo->SourceSurface->iBitmapFormat));
//       return FALSE;
//   }
//
//   return TRUE;
// }
//
// /* BitBlt Optimize */
// BOOLEAN
// DIB_8BPP_ColorFill(SURFOBJ* DestSurface, RECTL* DestRect, ULONG color)
// {
//   LONG DestY;
//
//   /* Make WellOrdered by making top < bottom and left < right */
//   RECTL_vMakeWellOrdered(DestRect);
//
//   for (DestY = DestRect->top; DestY< DestRect->bottom; DestY++)
//   {
//     DIB_8BPP_HLine(DestSurface, DestRect->left, DestRect->right, DestY, color);
//   }
//   return TRUE;
// }
//
//
// BOOLEAN
// DIB_8BPP_TransparentBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf,
//                         RECTL*  DestRect,  RECTL *SourceRect,
//                         XLATEOBJ *ColorTranslation, ULONG iTransColor)
// {
//   LONG RoundedRight, X, Y, SourceX = 0, SourceY = 0;
//   ULONG *DestBits, Source, Dest;
//
//   LONG DstHeight;
//   LONG DstWidth;
//   LONG SrcHeight;
//   LONG SrcWidth;
//
//   DstHeight = DestRect->bottom - DestRect->top;
//   DstWidth = DestRect->right - DestRect->left;
//   SrcHeight = SourceRect->bottom - SourceRect->top;
//   SrcWidth = SourceRect->right - SourceRect->left;
//
//   RoundedRight = DestRect->right - ((DestRect->right - DestRect->left) & 0x3);
//   DestBits = (ULONG*)((PBYTE)DestSurf->pvScan0 + DestRect->left +
//                       (DestRect->top * DestSurf->lDelta));
//
//   for(Y = DestRect->top; Y < DestRect->bottom; Y++)
//   {
//     DestBits = (ULONG*)((PBYTE)DestSurf->pvScan0 + DestRect->left +
//                         (Y * DestSurf->lDelta));
//     SourceY = SourceRect->top+(Y - DestRect->top) * SrcHeight / DstHeight;
//     for (X = DestRect->left; X < RoundedRight; X += 4, DestBits++)
//     {
//       Dest = *DestBits;
//
//       SourceX = SourceRect->left+(X - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//           SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           Dest &= 0xFFFFFF00;
//           Dest |= (XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFF);
//         }
//       }
//
//       SourceX = SourceRect->left+(X+1 - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//           SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           Dest &= 0xFFFF00FF;
//           Dest |= ((XLATEOBJ_iXlate(ColorTranslation, Source) << 8) & 0xFF00);
//         }
//       }
//
//       SourceX = SourceRect->left+(X+2 - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//           SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           Dest &= 0xFF00FFFF;
//           Dest |= ((XLATEOBJ_iXlate(ColorTranslation, Source) << 16) & 0xFF0000);
//         }
//       }
//
//       SourceX = SourceRect->left+(X+3 - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//           SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           Dest &= 0x00FFFFFF;
//           Dest |= ((XLATEOBJ_iXlate(ColorTranslation, Source) << 24) & 0xFF000000);
//         }
//       }
//
//       *DestBits = Dest;
//     }
//
//     if(X < DestRect->right)
//     {
//       for (; X < DestRect->right; X++)
//       {
//         SourceX = SourceRect->left+(X - DestRect->left) * SrcWidth / DstWidth;
//         if (SourceX >= 0 && SourceY >= 0 &&
//             SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//         {
//           Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//           if(Source != iTransColor)
//           {
//             *((BYTE*)DestBits) = (BYTE)(XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFF);
//           }
//         }
//         DestBits = (PULONG)((ULONG_PTR)DestBits + 1);
//       }
//     }
//   }
//
//   return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
