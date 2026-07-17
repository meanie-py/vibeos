use win32k_sys::win32k::*;

use crate::dib_helpers::*;

extern "system" {
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
#[derive(Copy, Clone)]
pub union NICEPIXEL32 {
    pub ul: ULONG,
    pub col: NICEPIXEL32_col,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NICEPIXEL32_col {
    pub red: UCHAR,
    pub green: UCHAR,
    pub blue: UCHAR,
    pub alpha: UCHAR,
}


#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_PutPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32, c: u32) {
    let byteaddr: PBYTE = ((*SurfObj).pvScan0 as PBYTE).offset((y * (*SurfObj).lDelta) as isize);
    let addr: PDWORD = (byteaddr as PDWORD).offset(x as isize);
    
    *addr = c;
}

#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32 {
    let byteaddr = ((*SurfObj).pvScan0 as PBYTE).offset((y * (*SurfObj).lDelta) as isize);
    let addr = (byteaddr as PDWORD).offset(x as isize);
    *addr as u32
}

#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_VLine(SurfObj: *mut SURFOBJ, x: i32, y1: i32, y2: i32, c: u32) {
    let byteaddr = ((*SurfObj).pvScan0 as PBYTE).offset((y1 * (*SurfObj).lDelta) as isize);
    let mut addr = (byteaddr as PDWORD).offset(x as isize);
    let lDelta = (*SurfObj).lDelta >> 2;
    
    let mut y1 = y1;
    while { y1 += 1; y1 - 1 } < y2 {
        *addr = c;
        addr = addr.offset(lDelta as isize);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_BitBltSrcCopy(BltInfo: PBLTINFO) -> u8 {
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
    let mut SourceBitsT: PBYTE;
    let mut SourceBitsB: PBYTE;
    let mut DestBitsT: PBYTE;
    let mut DestBitsB: PBYTE;
    let mut SourceBits_4BPP: PBYTE;
    let mut SourceLine_4BPP: PBYTE;
    let mut Source32: PDWORD;
    let mut Dest32: PDWORD;
    let mut Index: u32;
    let DestWidth: i32;
    let DestHeight: i32;
    let bTopToBottom: u8;
    let bLeftToRight: u8;
    let blDeltaSrcNeg: u8;
    let blDeltaDestNeg: u8;
    let mut blDeltaAdjustDone: u8 = 0u8;

    if ((*BltInfo).DestRect.left < 0) || ((*BltInfo).DestRect.top < 0) ||
       ((*BltInfo).DestRect.right < 0) || ((*BltInfo).DestRect.bottom < 0) {
        return 0u8;
    }

    blDeltaSrcNeg = if (*(*BltInfo).SourceSurface).lDelta < 0 { 1u8 } else { 0u8 };
    blDeltaDestNeg = if (*(*BltInfo).DestSurface).lDelta < 0 { 1u8 } else { 0u8 };

    bLeftToRight = if (*BltInfo).DestRect.left > (*BltInfo).DestRect.right { 1u8 } else { 0u8 };
    bTopToBottom = if (*BltInfo).DestRect.top > (*BltInfo).DestRect.bottom { 1u8 } else { 0u8 };

    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!((*BltInfo).DestRect));

    DestWidth = (*BltInfo).DestRect.right - (*BltInfo).DestRect.left;
    DestHeight = (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top;

    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
        .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
        .offset((4 * (*BltInfo).DestRect.left) as isize);

    match (*(*BltInfo).SourceSurface).iBitmapFormat {
        BMF_1BPP => {
            sx = (*BltInfo).SourcePoint.x;
            sy = (*BltInfo).SourcePoint.y;

            if bTopToBottom != 0 {
                sy += (*(*BltInfo).SourceSurface).lDelta * (DestHeight - 1);
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += DestWidth - 1;
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    if DIB_1BPP_GetPixel((*BltInfo).SourceSurface, sx, sy) == 0 {
                        DIB_32BPP_PutPixel((*BltInfo).DestSurface, i, j, 
                            XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, 0));
                    } else {
                        DIB_32BPP_PutPixel((*BltInfo).DestSurface, i, j,
                            XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, 1));
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
                SourceBits_4BPP = SourceBits_4BPP.offset(((*(*BltInfo).SourceSurface).lDelta * (DestHeight - 1)) as isize);
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceLine_4BPP = SourceBits_4BPP;
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += DestWidth - 1;
                }

                f1 = sx & 1;

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest,
                        ((*SourceLine_4BPP as i32 & altnotmask[f1 as usize] as i32) >> (4 * (1 - f1))) as u32) as i32;
                    DIB_32BPP_PutPixel((*BltInfo).DestSurface, i, j, xColor as u32);
                    if f1 == 1 {
                        SourceLine_4BPP = if bLeftToRight != 0 { SourceLine_4BPP.offset(-1) } else { SourceLine_4BPP.offset(1) };
                        f1 = 0;
                    } else {
                        f1 = 1;
                    }
                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                SourceBits_4BPP = if bTopToBottom != 0 {
                    SourceBits_4BPP.offset(-((*(*BltInfo).SourceSurface).lDelta as isize))
                } else {
                    SourceBits_4BPP.offset((*(*BltInfo).SourceSurface).lDelta as isize)
                };
                j += 1;
            }
        }

        BMF_8BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((*BltInfo).SourcePoint.x as isize);
            DestLine = DestBits;

            if bTopToBottom != 0 {
                SourceLine = SourceLine.offset(((*(*BltInfo).SourceSurface).lDelta * (DestHeight - 1)) as isize);
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;
                DestBits = DestLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset((DestWidth - 1) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = *SourceBits as i32;
                    *(DestBits as PDWORD) = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, xColor as u32);
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-1) } else { SourceBits.offset(1) };
                    DestBits = DestBits.offset(4);
                    i += 1;
                }
                SourceLine = if bTopToBottom != 0 {
                    SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize))
                } else {
                    SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize)
                };
                DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        BMF_16BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((2 * (*BltInfo).SourcePoint.x) as isize);
            DestLine = DestBits;

            if bTopToBottom != 0 {
                SourceLine = SourceLine.offset(((*(*BltInfo).SourceSurface).lDelta * (DestHeight - 1)) as isize);
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;
                DestBits = DestLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset(((DestWidth - 1) * 2) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = *(SourceBits as PWORD) as i32;
                    *(DestBits as PDWORD) = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, xColor as u32);
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-2) } else { SourceBits.offset(2) };
                    DestBits = DestBits.offset(4);
                    i += 1;
                }

                SourceLine = if bTopToBottom != 0 {
                    SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize))
                } else {
                    SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize)
                };
                DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        BMF_24BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((3 * (*BltInfo).SourcePoint.x) as isize);

            if bTopToBottom != 0 {
                SourceLine = SourceLine.offset(((*(*BltInfo).SourceSurface).lDelta * (DestHeight - 1)) as isize);
            }

            DestLine = DestBits;

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;
                DestBits = DestLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset(((DestWidth - 1) * 3) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = ((*SourceBits.offset(2) as i32) << 0x10) +
                             ((*SourceBits.offset(1) as i32) << 0x08) +
                             (*SourceBits as i32);
                    *(DestBits as PDWORD) = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, xColor as u32);
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-3) } else { SourceBits.offset(3) };
                    DestBits = DestBits.offset(4);
                    i += 1;
                }

                SourceLine = if bTopToBottom != 0 {
                    SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize))
                } else {
                    SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize)
                };
                DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        BMF_32BPP => {
            if ((blDeltaSrcNeg != 0 || blDeltaDestNeg != 0) && !(blDeltaSrcNeg != 0 && blDeltaDestNeg != 0)) && bTopToBottom != 0 {
                if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                    j = (*BltInfo).DestRect.top;
                    while j < (*BltInfo).DestRect.bottom {
                        core::ptr::copy(SourceBits, DestBits, (4 * DestWidth) as usize); // C: RtlMoveMemory — same-surface blits may overlap
                        SourceBits = SourceBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                        DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                        j += 1;
                    }
                } else {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).SourcePoint.y + DestHeight - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                        .offset((4 * (*BltInfo).DestRect.left) as isize);
                    j = (*BltInfo).DestRect.bottom - 1;
                    while (*BltInfo).DestRect.top <= j {
                        core::ptr::copy(SourceBits, DestBits, (4 * DestWidth) as usize); // C: RtlMoveMemory — same-surface blits may overlap
                        SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                        DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                        j -= 1;
                    }
                }
                blDeltaAdjustDone = 1u8;
            }

            if ((*BltInfo).XlateSourceToDest == core::ptr::null_mut() ||
                ((*(*BltInfo).XlateSourceToDest).flXlate & XO_TRIVIAL) != 0) &&
               (bTopToBottom == 0 && bLeftToRight == 0) {
                if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                    j = (*BltInfo).DestRect.top;
                    while j < (*BltInfo).DestRect.bottom {
                        core::ptr::copy(SourceBits, DestBits, (4 * DestWidth) as usize); // C: RtlMoveMemory — same-surface blits may overlap
                        SourceBits = SourceBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                        DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                        j += 1;
                    }
                } else {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).SourcePoint.y + DestHeight - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                        .offset((4 * (*BltInfo).DestRect.left) as isize);
                    j = (*BltInfo).DestRect.bottom - 1;
                    while (*BltInfo).DestRect.top <= j {
                        core::ptr::copy(SourceBits, DestBits, (4 * DestWidth) as usize); // C: RtlMoveMemory — same-surface blits may overlap
                        SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                        DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                        j -= 1;
                    }
                }
            } else {
                if bTopToBottom == 0 && bLeftToRight == 0 {
                    if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                        SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                            .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                        j = (*BltInfo).DestRect.top;
                        while j < (*BltInfo).DestRect.bottom {
                            if (*BltInfo).DestRect.left < (*BltInfo).SourcePoint.x {
                                Dest32 = DestBits as PDWORD;
                                Source32 = SourceBits as PDWORD;
                                i = (*BltInfo).DestRect.left;
                                while i < (*BltInfo).DestRect.right {
                                    *Dest32 = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *Source32);
                                    Dest32 = Dest32.offset(1);
                                    Source32 = Source32.offset(1);
                                    i += 1;
                                }
                            } else {
                                Dest32 = (DestBits as PDWORD).offset((DestWidth - 1) as isize);
                                Source32 = (SourceBits as PDWORD).offset((DestWidth - 1) as isize);
                                i = (*BltInfo).DestRect.right - 1;
                                while (*BltInfo).DestRect.left <= i {
                                    *Dest32 = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *Source32);
                                    Dest32 = Dest32.offset(-1);
                                    Source32 = Source32.offset(-1);
                                    i -= 1;
                                }
                            }
                            SourceBits = SourceBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                            DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                            j += 1;
                        }
                    } else {
                        SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).SourcePoint.y + DestHeight - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                            .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                        DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((4 * (*BltInfo).DestRect.left) as isize);
                        j = (*BltInfo).DestRect.bottom - 1;
                        while (*BltInfo).DestRect.top <= j {
                            if (*BltInfo).DestRect.left < (*BltInfo).SourcePoint.x {
                                Dest32 = DestBits as PDWORD;
                                Source32 = SourceBits as PDWORD;
                                i = (*BltInfo).DestRect.left;
                                while i < (*BltInfo).DestRect.right {
                                    *Dest32 = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *Source32);
                                    Dest32 = Dest32.offset(1);
                                    Source32 = Source32.offset(1);
                                    i += 1;
                                }
                            } else {
                                Dest32 = (DestBits as PDWORD).offset((DestWidth - 1) as isize);
                                Source32 = (SourceBits as PDWORD).offset((DestWidth - 1) as isize);
                                i = (*BltInfo).DestRect.right - 1;
                                while (*BltInfo).DestRect.left <= i {
                                    *Dest32 = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *Source32);
                                    Dest32 = Dest32.offset(-1);
                                    Source32 = Source32.offset(-1);
                                    i -= 1;
                                }
                            }
                            SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                            DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                            j -= 1;
                        }
                    }
                } else {
                    let mut TopToBottomDone: u8 = 0u8;

                    if (bLeftToRight != 0) && (DestWidth > 1) {
                        let store = ExAllocatePoolWithTag(NonPagedPool, ((DestWidth + 1) * 4) as SIZE_T, TAG_DIB);
                        if store == core::ptr::null_mut() {
                            return 0u8;
                        }

                        SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).SourcePoint.y + DestHeight - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                            .offset((4 * (*BltInfo).SourcePoint.x) as isize);

                        DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((4 * (*BltInfo).DestRect.left) as isize);

                        j = (*BltInfo).DestRect.bottom - 1;
                        while (*BltInfo).DestRect.top <= j {
                            Dest32 = (DestBits as PDWORD).offset((DestWidth - 1) as isize);
                            Source32 = SourceBits as PDWORD;

                            Index = 0;

                            i = (*BltInfo).DestRect.right - 1;
                            while (*BltInfo).DestRect.left <= i {
                                *(store as PDWORD).offset(Index as isize) = *Source32;
                                Source32 = Source32.offset(1);
                                Index += 1;
                                i -= 1;
                            }

                            Index = 0;

                            i = (*BltInfo).DestRect.right - 1;
                            while (*BltInfo).DestRect.left <= i {
                                *Dest32 = *(store as PDWORD).offset(Index as isize);
                                Dest32 = Dest32.offset(-1);
                                Index += 1;
                                i -= 1;
                            }
                            SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                            DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                            j -= 1;
                        }
                        ExFreePoolWithTag(store, TAG_DIB);
                        TopToBottomDone = 1u8;
                    }

                    if (bTopToBottom != 0) && (DestHeight > 1) {
                        let store = ExAllocatePoolWithTag(NonPagedPool, ((DestWidth + 1) * 4) as SIZE_T, TAG_DIB);
                        if store == core::ptr::null_mut() {
                            return 0u8;
                        }

                        DestBitsT = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((4 * (*BltInfo).DestRect.left) as isize);

                        DestBitsB = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((4 * (*BltInfo).DestRect.left) as isize);

                        if TopToBottomDone != 0 || blDeltaAdjustDone != 0 {
                            SourceBitsB = DestBitsB;
                            SourceBitsT = DestBitsT;
                        } else {
                            SourceBitsB = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                                .offset((((*BltInfo).SourcePoint.y + DestHeight - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                                .offset((4 * (*BltInfo).SourcePoint.x) as isize);

                            SourceBitsT = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                                .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                        }

                        if ((*BltInfo).SourcePoint.y != (*BltInfo).DestRect.top) &&
                           (abs((*BltInfo).SourcePoint.y - (*BltInfo).DestRect.top) < (DestHeight + 2)) &&
                           ((*(*BltInfo).SourceSurface).pvScan0 == (*(*BltInfo).DestSurface).pvScan0) {
                            if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                                SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                                    .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                                    .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                                j = (*BltInfo).DestRect.top;
                                while j < (*BltInfo).DestRect.bottom {
                                    core::ptr::copy(SourceBits, DestBits, (4 * DestWidth) as usize); // C: RtlMoveMemory — same-surface blits may overlap
                                    SourceBits = SourceBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                                    DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                                    j += 1;
                                }
                            } else {
                                SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                                    .offset((((*BltInfo).SourcePoint.y + DestHeight - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                                    .offset((4 * (*BltInfo).SourcePoint.x) as isize);
                                DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                                    .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                                    .offset((4 * (*BltInfo).DestRect.left) as isize);
                                j = (*BltInfo).DestRect.bottom - 1;
                                while (*BltInfo).DestRect.top <= j {
                                    core::ptr::copy(SourceBits, DestBits, (4 * DestWidth) as usize); // C: RtlMoveMemory — same-surface blits may overlap
                                    SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                                    DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                                    j -= 1;
                                }
                            }

                            SourceBitsB = DestBitsB;
                            SourceBitsT = DestBitsT;
                        }

                        j = 0;
                        while j < DestHeight / 2 {
                            core::ptr::copy(SourceBitsB, store as PBYTE, (4 * DestWidth) as usize); // C: RtlMoveMemory
                            core::ptr::copy(SourceBitsT, DestBitsB, (4 * DestWidth) as usize); // C: RtlMoveMemory
                            core::ptr::copy(store as PBYTE, DestBitsT, (4 * DestWidth) as usize); // C: RtlMoveMemory

                            SourceBitsT = SourceBitsT.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                            SourceBitsB = SourceBitsB.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));

                            DestBitsT = DestBitsT.offset((*(*BltInfo).DestSurface).lDelta as isize);
                            DestBitsB = DestBitsB.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                            j += 1;
                        }
                        if DestHeight % 2 != 0 {
                            core::ptr::copy(SourceBitsT, DestBitsB, (4 * DestWidth) as usize); // C: RtlMoveMemory
                        }
                        ExFreePoolWithTag(store, TAG_DIB);
                    }
                }
            }
        }

        _ => {
            return 0u8;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_TransparentBlt(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ColorTranslation: *mut XLATEOBJ,
    iTransColor: ULONG,
) -> BOOLEAN {
    let mut X: LONG;
    let mut Y: LONG;
    let mut SourceX: LONG;
    let mut SourceY: LONG = 0;
    let wd: LONG;
    let mut DestBits: *mut ULONG;
    let mut Source: ULONG = 0;

    let DstHeight: LONG;
    let DstWidth: LONG;
    let SrcHeight: LONG;
    let SrcWidth: LONG;

    DstHeight = (*DestRect).bottom - (*DestRect).top;
    DstWidth = (*DestRect).right - (*DestRect).left;
    SrcHeight = (*SourceRect).bottom - (*SourceRect).top;
    SrcWidth = (*SourceRect).right - (*SourceRect).left;

    DestBits = ((*DestSurf).pvScan0 as PBYTE)
        .offset(((*DestRect).left << 2) as isize)
        .offset(((*DestRect).top * (*DestSurf).lDelta) as isize) as *mut ULONG;
    wd = (*DestSurf).lDelta - (((*DestRect).right - (*DestRect).left) << 2);

    Y = (*DestRect).top;
    while Y < (*DestRect).bottom {
        SourceY = (*SourceRect).top + (Y - (*DestRect).top) * SrcHeight / DstHeight;
        X = (*DestRect).left;
        while X < (*DestRect).right {
            SourceX = (*SourceRect).left + (X - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0
                && SourceY >= 0
                && (*SourceSurf).sizlBitmap.cx > SourceX
                && (*SourceSurf).sizlBitmap.cy > SourceY
            {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if (0x00FFFFFF & Source) != (0x00FFFFFF & iTransColor) {
                    *DestBits = XLATEOBJ_iXlate(ColorTranslation, Source);
                }
            }
            DestBits = DestBits.offset(1);
            X += 1;
        }

        DestBits = (DestBits as ULONG_PTR).wrapping_add(wd as ULONG_PTR) as *mut ULONG;
        Y += 1;
    }

    1u8
}

unsafe fn Clamp8(val: ULONG) -> UCHAR {
    if val > 255 { 255 } else { val as UCHAR }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_AlphaBlend(
    Dest: *mut SURFOBJ,
    Source: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    BlendObj: *mut BLENDOBJ,
) -> BOOLEAN {
    let mut Rows: INT;
    let mut Cols: INT;
    let mut SrcX: INT;
    let mut SrcY: INT;
    let mut Dst: *mut ULONG;
    let BlendFunc: BLENDFUNCTION;
    let mut DstPixel: NICEPIXEL32 = core::mem::zeroed();
    let mut SrcPixel: NICEPIXEL32 = core::mem::zeroed();
    let mut Alpha: UCHAR;
    let SrcBpp: ULONG;

    BlendFunc = (*BlendObj).BlendFunction;
    if BlendFunc.BlendOp as u32 != AC_SRC_OVER {
        return 0u8;
    }
    if BlendFunc.BlendFlags != 0 {
        return 0u8;
    }
    if (BlendFunc.AlphaFormat as u32 & !AC_SRC_ALPHA) != 0 {
        return 0u8;
    }
    if (BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0
        && BitsPerFormat((*Source).iBitmapFormat) != 32
    {
        return 0u8;
    }

    Dst = ((*Dest).pvScan0 as ULONG_PTR
        + ((*DestRect).top * (*Dest).lDelta) as ULONG_PTR
        + (((*DestRect).left << 2) as ULONG_PTR)) as *mut ULONG;
    SrcBpp = BitsPerFormat((*Source).iBitmapFormat);

    Rows = 0;
    SrcY = (*SourceRect).top;
    while { Rows += 1; Rows } <= (*DestRect).bottom - (*DestRect).top {
        Cols = 0;
        SrcX = (*SourceRect).left;
        while { Cols += 1; Cols } <= (*DestRect).right - (*DestRect).left {
            SrcPixel.ul = DIB_GetSource(Source, SrcX, SrcY, ColorTranslation);
            SrcPixel.col.red =
                (SrcPixel.col.red as ULONG * BlendFunc.SourceConstantAlpha as ULONG / 255) as UCHAR;
            SrcPixel.col.green = (SrcPixel.col.green as ULONG
                * BlendFunc.SourceConstantAlpha as ULONG
                / 255) as UCHAR;
            SrcPixel.col.blue = (SrcPixel.col.blue as ULONG
                * BlendFunc.SourceConstantAlpha as ULONG
                / 255) as UCHAR;
            SrcPixel.col.alpha = if 32 == SrcBpp {
                (SrcPixel.col.alpha as ULONG * BlendFunc.SourceConstantAlpha as ULONG / 255)
                    as UCHAR
            } else {
                BlendFunc.SourceConstantAlpha
            };

            Alpha = if (BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0 {
                SrcPixel.col.alpha
            } else {
                BlendFunc.SourceConstantAlpha
            };

            DstPixel.ul = *Dst;
            DstPixel.col.red = Clamp8(
                DstPixel.col.red as ULONG * (255 - Alpha as ULONG) / 255
                    + SrcPixel.col.red as ULONG,
            );
            DstPixel.col.green = Clamp8(
                DstPixel.col.green as ULONG * (255 - Alpha as ULONG) / 255
                    + SrcPixel.col.green as ULONG,
            );
            DstPixel.col.blue = Clamp8(
                DstPixel.col.blue as ULONG * (255 - Alpha as ULONG) / 255
                    + SrcPixel.col.blue as ULONG,
            );
            DstPixel.col.alpha = Clamp8(
                DstPixel.col.alpha as ULONG * (255 - Alpha as ULONG) / 255
                    + SrcPixel.col.alpha as ULONG,
            );
            *Dst = DstPixel.ul;
            Dst = Dst.offset(1);
            SrcX = (*SourceRect).left
                + (Cols * ((*SourceRect).right - (*SourceRect).left))
                    / ((*DestRect).right - (*DestRect).left);
        }
        Dst = ((*Dest).pvScan0 as ULONG_PTR
            + (((*DestRect).top + Rows) * (*Dest).lDelta) as ULONG_PTR
            + (((*DestRect).left << 2) as ULONG_PTR)) as *mut ULONG;
        SrcY = (*SourceRect).top
            + (Rows * ((*SourceRect).bottom - (*SourceRect).top))
                / ((*DestRect).bottom - (*DestRect).top);
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
//  * FILE:            win32ss/gdi/dib/dib32bpp.c
//  * PURPOSE:         Device Independant Bitmap functions, 32bpp
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
// DIB_32BPP_PutPixel(SURFOBJ *SurfObj, LONG x, LONG y, ULONG c)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta;
//   PDWORD addr = (PDWORD)byteaddr + x;
//
//   *addr = c;
// }
//
// ULONG
// DIB_32BPP_GetPixel(SURFOBJ *SurfObj, LONG x, LONG y)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta;
//   PDWORD addr = (PDWORD)byteaddr + x;
//
//   return (ULONG)(*addr);
// }
//
// VOID
// DIB_32BPP_VLine(SURFOBJ *SurfObj, LONG x, LONG y1, LONG y2, ULONG c)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y1 * SurfObj->lDelta;
//   PDWORD addr = (PDWORD)byteaddr + x;
//   LONG lDelta = SurfObj->lDelta >> 2; // >> 2 == / sizeof(DWORD)
//
//   byteaddr = (PBYTE)addr;
//   while (y1++ < y2)
//   {
//     *addr = (DWORD)c;
//     addr += lDelta;
//   }
// }
//
// BOOLEAN
// DIB_32BPP_BitBltSrcCopy(PBLTINFO BltInfo)
// {
//   LONG     i, j, sx, sy, xColor, f1;
//   PBYTE    SourceBits, DestBits, SourceLine, DestLine;
//   PBYTE    SourceBitsT, SourceBitsB, DestBitsT, DestBitsB;
//   PBYTE    SourceBits_4BPP, SourceLine_4BPP;
//   PDWORD   Source32, Dest32;
//   DWORD    Index;
//   LONG     DestWidth, DestHeight;
//   BOOLEAN  bTopToBottom, bLeftToRight;
//   BOOLEAN  blDeltaSrcNeg, blDeltaDestNeg;
//   BOOLEAN  blDeltaAdjustDone = FALSE;
//
//   DPRINT("DIB_32BPP_BitBltSrcCopy: SourcePoint (%d, %d), SourceSurface cx/cy (%d/%d), "
//          "DestSurface cx/cy (%d/%d) DestRect: (%d,%d)-(%d,%d)\n",
//          BltInfo->SourcePoint.x, BltInfo->SourcePoint.y,
//          BltInfo->SourceSurface->sizlBitmap.cx, BltInfo->SourceSurface->sizlBitmap.cy,
//          BltInfo->DestSurface->sizlBitmap.cx, BltInfo->DestSurface->sizlBitmap.cy,
//          BltInfo->DestRect.left, BltInfo->DestRect.top, BltInfo->DestRect.right, BltInfo->DestRect.bottom);
//
//   DPRINT("BltInfo->DestSurface->lDelta is '%d' and BltInfo->SourceSurface->lDelta is '%d'.\n",
//            BltInfo->DestSurface->lDelta,  BltInfo->SourceSurface->lDelta);
//
//   DPRINT("iBitmapFormat is %d and width,height is (%d,%d).\n", BltInfo->SourceSurface->iBitmapFormat,
//          BltInfo->DestRect.right - BltInfo->DestRect.left, BltInfo->DestRect.bottom - BltInfo->DestRect.top);
//
//   DPRINT("BltInfo->SourcePoint.x is '%d' and BltInfo->SourcePoint.y is '%d'.\n",
//          BltInfo->SourcePoint.x, BltInfo->SourcePoint.y);
//
//   /* Do not deal with negative numbers for these values */
//   if ((BltInfo->DestRect.left < 0) || (BltInfo->DestRect.top < 0) ||
//       (BltInfo->DestRect.right < 0) || (BltInfo->DestRect.bottom < 0))
//     return FALSE;
//
//   /* Detect negative lDelta's meaning Bottom-Up bitmaps */
//   blDeltaSrcNeg = BltInfo->SourceSurface->lDelta < 0;
//   blDeltaDestNeg = BltInfo->DestSurface->lDelta < 0;
//
//   /* Get back left to right flip here */
//   bLeftToRight = BltInfo->DestRect.left > BltInfo->DestRect.right;
//
//   /* Check for top to bottom flip needed. */
//   bTopToBottom = BltInfo->DestRect.top > BltInfo->DestRect.bottom;
//
//   DPRINT("bTopToBottom is '%d' and DestSurface->lDelta < 0 is '%d' and SourceSurface->lDelta < 0 is '%d'.\n",
//            bTopToBottom, BltInfo->DestSurface->lDelta < 0 ? 1 : 0,  BltInfo->SourceSurface->lDelta < 0 ? 1 : 0);
//
//   /* Make WellOrdered with top < bottom and left < right */
//   RECTL_vMakeWellOrdered(&BltInfo->DestRect);
//
//   DestWidth = BltInfo->DestRect.right - BltInfo->DestRect.left;
//   DestHeight = BltInfo->DestRect.bottom - BltInfo->DestRect.top;
//
//   DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//     + (BltInfo->DestRect.top * BltInfo->DestSurface->lDelta)
//     + 4 * BltInfo->DestRect.left;
//
//   DPRINT("iBitmapFormat is %d and width,height is (%d,%d).\n", BltInfo->SourceSurface->iBitmapFormat,
//          DestWidth, DestHeight);
//
//   switch (BltInfo->SourceSurface->iBitmapFormat)
//   {
//   case BMF_1BPP:
//     DPRINT("1BPP Case Selected with DestRect Width of '%d'.\n",
//            DestWidth);
//
//     if (bLeftToRight || bTopToBottom)
//     DPRINT("bLeftToRight is '%d' and bTopToBottom is '%d'.\n", bLeftToRight, bTopToBottom);
//
//     sx = BltInfo->SourcePoint.x;
//
//     /* This sets sy to the top line */
//     sy = BltInfo->SourcePoint.y;
//
//     if (bTopToBottom)
//     {
//       /* This sets sy to the bottom line */
//       sy += BltInfo->SourceSurface->lDelta * (DestHeight - 1);
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//         /* This sets the sx to the rightmost pixel */
//         sx += (DestWidth - 1);
//       }
//
//       for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//       {
//         if (DIB_1BPP_GetPixel(BltInfo->SourceSurface, sx, sy) == 0)
//         {
//           DIB_32BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 0));
//         }
//         else
//         {
//           DIB_32BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 1));
//         }
//
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(sy, bTopToBottom, 1);
//     }
//     break;
//
//   case BMF_4BPP:
//     DPRINT("4BPP Case Selected with DestRect Width of '%d'.\n",
//            DestWidth);
//
//     if (bLeftToRight || bTopToBottom)
//     DPRINT("bLeftToRight is '%d' and bTopToBottom is '%d'.\n", bLeftToRight, bTopToBottom);
//
//     /* This sets SourceBits_4BPP to the top line */
//     SourceBits_4BPP = (PBYTE)BltInfo->SourceSurface->pvScan0
//       + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//       + (BltInfo->SourcePoint.x >> 1);
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceBits_4BPP to the bottom line */
//       SourceBits_4BPP += BltInfo->SourceSurface->lDelta * (DestHeight - 1);
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       SourceLine_4BPP = SourceBits_4BPP;
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//         /* This sets sx to the rightmost pixel */
//         sx += (DestWidth - 1);
//       }
//
//       f1 = sx & 1;
//
//       for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//       {
//         xColor = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest,
//           (*SourceLine_4BPP & altnotmask[f1]) >> (4 * (1 - f1)));
//         DIB_32BPP_PutPixel(BltInfo->DestSurface, i, j, xColor);
//         if (f1 == 1) {
//           DEC_OR_INC(SourceLine_4BPP, bLeftToRight, 1);
//           f1 = 0;
//         } else {
//           f1 = 1;
//         }
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(SourceBits_4BPP, bTopToBottom, BltInfo->SourceSurface->lDelta);
//     }
//     break;
//
//   case BMF_8BPP:
//     DPRINT("8BPP Case Selected with DestRect Width of '%d'.\n",
//            DestWidth);
//
//     if (bLeftToRight || bTopToBottom)
//     DPRINT("bLeftToRight is '%d' and bTopToBottom is '%d'.\n", bLeftToRight, bTopToBottom);
//
//     /* This sets SourceLine to the top line */
//     SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0
//       + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//       + BltInfo->SourcePoint.x;
//     DestLine = DestBits;
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceLine to the bottom line */
//       SourceLine += BltInfo->SourceSurface->lDelta * (DestHeight - 1);
//     }
//
//     for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//     {
//       SourceBits = SourceLine;
//       DestBits = DestLine;
//
//       if (bLeftToRight)
//       {
//         /* This sets the SourceBits to the rightmost pixel */
//         SourceBits += (DestWidth - 1);
//       }
//
//       for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//       {
//         xColor = *SourceBits;
//         *((PDWORD) DestBits) = (DWORD)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, xColor);
//         DEC_OR_INC(SourceBits, bLeftToRight, 1);
//         DestBits += 4;
//       }
//       DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//       DestLine += BltInfo->DestSurface->lDelta;
//     }
//     break;
//
//   case BMF_16BPP:
//     DPRINT("16BPP Case Selected with DestRect Width of '%d'.\n",
//             DestWidth);
//
//     if (bLeftToRight || bTopToBottom)
//     DPRINT("bLeftToRight is '%d' and bTopToBottom is '%d'.\n", bLeftToRight, bTopToBottom);
//
//     /* This sets SourceLine to the top line */
//     SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0
//       + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//       + 2 * BltInfo->SourcePoint.x;
//     DestLine = DestBits;
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceLine to the bottom line */
//       SourceLine += BltInfo->SourceSurface->lDelta * (DestHeight - 1);
//     }
//
//     for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//     {
//       SourceBits = SourceLine;
//       DestBits = DestLine;
//
//       if (bLeftToRight)
//       {
//         /* This sets the SourceBits to the rightmost pixel */
//         SourceBits += (DestWidth - 1) * 2;
//       }
//
//       for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//       {
//         xColor = *((PWORD) SourceBits);
//         *((PDWORD) DestBits) = (DWORD)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, xColor);
//         DEC_OR_INC(SourceBits, bLeftToRight, 2);
//         DestBits += 4;
//       }
//
//       DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//       DestLine += BltInfo->DestSurface->lDelta;
//     }
//     break;
//
//   case BMF_24BPP:
//     DPRINT("24BPP Case Selected with DestRect Width of '%d'.\n",
//       DestWidth);
//
//     if (bLeftToRight || bTopToBottom)
//     DPRINT("bLeftToRight is '%d' and bTopToBottom is '%d'.\n", bLeftToRight, bTopToBottom);
//
//     /* This sets SourceLine to the top line */
//     SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0
//       + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//       + 3 * BltInfo->SourcePoint.x;
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceLine to the bottom line */
//       SourceLine += BltInfo->SourceSurface->lDelta * (DestHeight - 1);
//     }
//
//     DestLine = DestBits;
//
//     for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//     {
//       SourceBits = SourceLine;
//       DestBits = DestLine;
//
//       if (bLeftToRight)
//       {
//         /* This sets the SourceBits to the rightmost pixel */
//         SourceBits += (DestWidth - 1) * 3;
//       }
//
//       for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//       {
//         xColor = (*(SourceBits + 2) << 0x10) +
//           (*(SourceBits + 1) << 0x08) +
//           (*(SourceBits));
//         *((PDWORD)DestBits) = (DWORD)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, xColor);
//         DEC_OR_INC(SourceBits, bLeftToRight, 3);
//         DestBits += 4;
//       }
//
//       DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//       DestLine += BltInfo->DestSurface->lDelta;
//     }
//     break;
//
//   case BMF_32BPP:
//     DPRINT("32BPP Case Selected with SourcePoint (%d,%d) and DestRect Width/height of '%d/%d' DestRect: (%d,%d)-(%d,%d).\n",
//            BltInfo->SourcePoint.x, BltInfo->SourcePoint.y, DestWidth, DestHeight,
//            BltInfo->DestRect.left, BltInfo->DestRect.top, BltInfo->DestRect.right, BltInfo->DestRect.bottom);
//
//     if (bLeftToRight || bTopToBottom)
//     DPRINT("bLeftToRight is '%d' and bTopToBottom is '%d'.\n", bLeftToRight, bTopToBottom);
//
//     /* This handles the negative lDelta's which represent Top-to-Bottom bitmaps */
//     if (((blDeltaSrcNeg || blDeltaDestNeg) && !(blDeltaSrcNeg && blDeltaDestNeg)) && bTopToBottom)
//     {
//       DPRINT("Adjusting for lDelta's here.\n");
//       if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//       {
//         /* SourceBits points to top-left pixel for lDelta < 0 and bottom-left for lDelta > 0 */
//         SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//           + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//           + 4 * BltInfo->SourcePoint.x;
//         for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//         {
//           RtlMoveMemory(DestBits, SourceBits, 4 * DestWidth);
//           SourceBits += BltInfo->SourceSurface->lDelta;
//           DestBits += BltInfo->DestSurface->lDelta;
//         }
//       }
//       else
//       {
//         /* SourceBits points to bottom-left pixel for lDelta < 0 and top-left for lDelta > 0 */
//         SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//           + ((BltInfo->SourcePoint.y + DestHeight - 1) * BltInfo->SourceSurface->lDelta)
//           + 4 * BltInfo->SourcePoint.x;
//         /* DestBits points to bottom-left pixel for lDelta < 0 and top-left for lDelta > 0 */
//         DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//           + ((BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta)
//           + 4 * BltInfo->DestRect.left;
//         for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//         {
//           RtlMoveMemory(DestBits, SourceBits, 4 * DestWidth);
//           SourceBits -= BltInfo->SourceSurface->lDelta;
//           DestBits -= BltInfo->DestSurface->lDelta;
//         }
//       }
//       blDeltaAdjustDone = TRUE;
//     }
//
//     /* This tests for whether we can use simplified/quicker code below.
//      * It works for increasing source and destination areas only and there is no overlap and no flip.
//      */
//     if ((BltInfo->XlateSourceToDest == NULL ||
//       (BltInfo->XlateSourceToDest->flXlate & XO_TRIVIAL) != 0) &&
//       (!bTopToBottom && !bLeftToRight))
//     {
//       DPRINT("XO_TRIVIAL is TRUE.\n");
//
//       if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//       {
//         /* SourceBits points to top-left pixel for lDelta < 0 and bottom-left for lDelta > 0 */
//         SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//           + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//           + 4 * BltInfo->SourcePoint.x;
//         for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//         {
//           RtlMoveMemory(DestBits, SourceBits, 4 * DestWidth);
//           SourceBits += BltInfo->SourceSurface->lDelta;
//           DestBits += BltInfo->DestSurface->lDelta;
//         }
//       }
//       else
//       {
//         /* SourceBits points to bottom-left pixel for lDelta < 0 and top-left for lDelta > 0 */
//         SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//           + ((BltInfo->SourcePoint.y
//           + DestHeight - 1) * BltInfo->SourceSurface->lDelta)
//           + 4 * BltInfo->SourcePoint.x;
//         /* SourceBits points to bottom-left pixel for lDelta < 0 and top-left for lDelta > 0 */
//         DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//           + ((BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta)
//           + 4 * BltInfo->DestRect.left;
//         for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//         {
//           RtlMoveMemory(DestBits, SourceBits, 4 * DestWidth);
//           SourceBits -= BltInfo->SourceSurface->lDelta;
//           DestBits -= BltInfo->DestSurface->lDelta;
//         }
//       }
//     }
//     else
//     {
//       DPRINT("XO_TRIVIAL is NOT TRUE.\n");
//
//       if (!bTopToBottom && !bLeftToRight)
//       {
//         if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//         {
//           SourceBits = ((PBYTE)BltInfo->SourceSurface->pvScan0
//             + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//             + 4 * BltInfo->SourcePoint.x);
//           for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//           {
//             if (BltInfo->DestRect.left < BltInfo->SourcePoint.x)
//             {
//               Dest32 = (DWORD *) DestBits;
//               Source32 = (DWORD *) SourceBits;
//               for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//               {
//                 *Dest32++ = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *Source32++);
//               }
//             }
//             else
//             {
//               Dest32 = (DWORD *) DestBits + (DestWidth - 1);
//               Source32 = (DWORD *) SourceBits + (DestWidth - 1);
//               for (i = BltInfo->DestRect.right - 1; BltInfo->DestRect.left <= i; i--)
//               {
//                 *Dest32-- = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *Source32--);
//               }
//             }
//             SourceBits += BltInfo->SourceSurface->lDelta;
//             DestBits += BltInfo->DestSurface->lDelta;
//           }
//         }
//         else
//         {
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//             + ((BltInfo->SourcePoint.y
//             + DestHeight - 1) * BltInfo->SourceSurface->lDelta)
//             + 4 * BltInfo->SourcePoint.x;
//           DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//             + ((BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta)
//             + 4 * BltInfo->DestRect.left;
//           for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//           {
//             if (BltInfo->DestRect.left < BltInfo->SourcePoint.x)
//             {
//               Dest32 = (DWORD *) DestBits;
//               Source32 = (DWORD *) SourceBits;
//               for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//               {
//                 *Dest32++ = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *Source32++);
//               }
//             }
//             else
//             {
//               Dest32 = (DWORD *) DestBits + (DestWidth - 1);
//               Source32 = (DWORD *) SourceBits + (DestWidth - 1);
//               for (i = BltInfo->DestRect.right - 1; BltInfo->DestRect.left <= i; i--)
//               {
//                 *Dest32-- = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *Source32--);
//               }
//             }
//             SourceBits -= BltInfo->SourceSurface->lDelta;
//             DestBits -= BltInfo->DestSurface->lDelta;
//           }
//         }
//       }
//       else
//       {
//         /* Buffering for source and destination flip overlaps. Fixes KHMZ MirrorTest CORE-16642 */
//         BOOL TopToBottomDone = FALSE;
//
//         /* No need to flip a LeftToRight bitmap only one pixel wide */
//         if ((bLeftToRight) && (DestWidth > 1))
//         {
//           DPRINT("Flip is bLeftToRight.\n");
//
//           /* Allocate enough pixels for a row in DWORD's */
//           DWORD *store = ExAllocatePoolWithTag(NonPagedPool,
//             (DestWidth + 1) * 4, TAG_DIB);
//           if (store == NULL)
//           {
//             DPRINT1("Storage Allocation Failed.\n");
//             return FALSE;
//           }
//
//           /* This sets SourceBits to the bottom line */
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//             + ((BltInfo->SourcePoint.y + DestHeight - 1)
//             * BltInfo->SourceSurface->lDelta) + 4 * BltInfo->SourcePoint.x;
//
//           /* This sets DestBits to the bottom line */
//           DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//            + (BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta
//            + 4 * BltInfo->DestRect.left;
//
//           for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//           {
//
//               /* Set Dest32 to right pixel */
//               Dest32 = (DWORD *) DestBits + (DestWidth - 1);
//               Source32 = (DWORD *) SourceBits;
//
//               Index = 0;
//
//               /* Store pixels from left to right */
//               for (i = BltInfo->DestRect.right - 1; BltInfo->DestRect.left <= i; i--)
//               {
//                 store[Index] = *Source32++;
//                 Index++;
//               }
//
//               Index = 0;
//
//               /* Copy stored dat to pixels from right to left */
//               for (i = BltInfo->DestRect.right - 1; BltInfo->DestRect.left <= i; i--)
//               {
//                 *Dest32-- = store[Index];
//                 Index++;
//               }
//             SourceBits -= BltInfo->SourceSurface->lDelta;
//             DestBits -= BltInfo->DestSurface->lDelta;
//           }
//           ExFreePoolWithTag(store, TAG_DIB);
//           TopToBottomDone = TRUE;
//         }
//
//         /* Top to Botoom Handling if bitmap more than one pixel high */
//         if ((bTopToBottom) && (DestHeight > 1))
//          {
//           /* Note: It is very important that this code remain optimized for time used.
//            *   Otherwise you will have random crashes in ReactOS that are undesirable.
//            *   For an example of this just try executing the code here two times.
//            */
//
//           DPRINT("Flip is bTopToBottom.\n");
//
//           /* Allocate enough pixels for a row in DWORD's */
//           DWORD *store = ExAllocatePoolWithTag(NonPagedPool,
//             (DestWidth + 1) * 4, TAG_DIB);
//           if (store == NULL)
//           {
//             DPRINT1("Storage Allocation Failed.\n");
//             return FALSE;
//           }
//
//           /* This set DestBitsT to the top line */
//           DestBitsT = (PBYTE)BltInfo->DestSurface->pvScan0
//             + ((BltInfo->DestRect.top) * BltInfo->DestSurface->lDelta)
//             + 4 * BltInfo->DestRect.left;
//
//           /* This sets DestBitsB to the bottom line */
//           DestBitsB = (PBYTE)BltInfo->DestSurface->pvScan0
//            + (BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta
//            + 4 * BltInfo->DestRect.left;
//
//           /* The TopToBottomDone flag indicates that we are flipping for bTopToBottom and bLeftToRight
//            * and have already completed the bLeftToRight. So we will lose our first flip output
//            * unless we work with its output which is at the destination site. So in this case
//            * our new Source becomes the previous outputs Destination.
//            * Also in we use the same logic when we have corrected for negative lDelta's above
//            * and already completed a flip from Source to Destination for the first step
//            */
//
//           if (TopToBottomDone || blDeltaAdjustDone)
//           {
//             /* This sets SourceBitsB to the bottom line */
//             SourceBitsB = DestBitsB;
//
//             /* This sets SourceBitsT to the top line */
//             SourceBitsT = DestBitsT;
//           }
//           else
//           {
//             /* This sets SourceBitsB to the bottom line */
//             SourceBitsB = (PBYTE)BltInfo->SourceSurface->pvScan0
//               + ((BltInfo->SourcePoint.y + DestHeight - 1)
//               * BltInfo->SourceSurface->lDelta) + 4 * BltInfo->SourcePoint.x;
//
//             /* This sets SourceBitsT to the top line */
//             SourceBitsT = (PBYTE)BltInfo->SourceSurface->pvScan0
//               + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + 4 * BltInfo->SourcePoint.x;
//           }
//
//           /* Overlaps and Vertical flips do not mix well. So we test for this and handle it
//            * by using two operations. First we just do a copy of the source to the destination.
//            * Then we do a flip in place at the destination location and we are done.
//            */
//           if ((BltInfo->SourcePoint.y != BltInfo->DestRect.top) &&                        // The values are not equal and
//              (abs(BltInfo->SourcePoint.y - BltInfo->DestRect.top) < (DestHeight + 2)) &&  // they are NOT separated by > DestHeight
//              (BltInfo->SourceSurface->pvScan0 == BltInfo->DestSurface->pvScan0))          // and same surface (probably screen)
//           {
//             DPRINT("Flips Need Adjustments, so do move here.\n");
//
//             if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//             {
//               /* SourceBits points to top-left pixel for lDelta < 0 and bottom-left for lDelta > 0 */
//               SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//                 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//                 + 4 * BltInfo->SourcePoint.x;
//               for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//               {
//                 RtlMoveMemory(DestBits, SourceBits, 4 * DestWidth);
//                 SourceBits += BltInfo->SourceSurface->lDelta;
//                 DestBits += BltInfo->DestSurface->lDelta;
//               }
//             }
//             else
//             {
//               /* SourceBits points to bottom-left pixel for lDelta < 0 and top-left for lDelta > 0 */
//               SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//                 + ((BltInfo->SourcePoint.y
//                 + DestHeight - 1) * BltInfo->SourceSurface->lDelta)
//                 + 4 * BltInfo->SourcePoint.x;
//               /* SourceBits points to bottom-left pixel for lDelta < 0 and top-left for lDelta > 0 */
//               DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//                 + ((BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta)
//                 + 4 * BltInfo->DestRect.left;
//               for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//               {
//                 RtlMoveMemory(DestBits, SourceBits, 4 * DestWidth);
//                 SourceBits -= BltInfo->SourceSurface->lDelta;
//                 DestBits -= BltInfo->DestSurface->lDelta;
//               }
//             }
//
//             /* This sets SourceBitsB to the bottom line */
//             SourceBitsB = DestBitsB;
//
//             /* This sets SourceBitsT to the top line */
//             SourceBitsT = DestBitsT;
//           }
//
//           /* Vertical Flip code starts here */
//           for (j = 0; j < DestHeight / 2 ; j++)
//           {
//             /* Store bottom row of Source pixels */
//             RtlMoveMemory(store, SourceBitsB, 4 * DestWidth);
//
//             /* Copy top Source row to bottom Destination row overwriting it */
//             RtlMoveMemory(DestBitsB, SourceBitsT, 4 * DestWidth);
//
//             /* Copy stored bottom row of Source pixels to Destination top row of pixels */
//             RtlMoveMemory(DestBitsT, store, 4 * DestWidth);
//
//             /* Index top rows down and bottom rows up */
//             SourceBitsT += BltInfo->SourceSurface->lDelta;
//             SourceBitsB -= BltInfo->SourceSurface->lDelta;
//
//             DestBitsT += BltInfo->DestSurface->lDelta;
//             DestBitsB -= BltInfo->DestSurface->lDelta;
//           }
//           if (DestHeight % 2)
//           {
//             /* If we had an odd number of lines we handle the center one here */
//             DPRINT("Handling Top To Bottom with Odd Number of lines.\n");
//             RtlMoveMemory(DestBitsB, SourceBitsT, 4 * DestWidth);
//           }
//           ExFreePoolWithTag(store, TAG_DIB);
//         }
//       }
//     }
//     break;
//
//   default:
//     DPRINT1("DIB_32BPP_BitBltSrcCopy: Unhandled Source BPP: %u\n", BitsPerFormat(BltInfo->SourceSurface->iBitmapFormat));
//     return FALSE;
//   }
//
//   return TRUE;
// }
//
// BOOLEAN
// DIB_32BPP_TransparentBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf,
//                          RECTL*  DestRect,  RECTL *SourceRect,
//                          XLATEOBJ *ColorTranslation, ULONG iTransColor)
// {
//   LONG X, Y, SourceX, SourceY = 0, wd;
//   ULONG *DestBits, Source = 0;
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
//   DestBits = (ULONG*)((PBYTE)DestSurf->pvScan0 +
//     (DestRect->left << 2) +
//     DestRect->top * DestSurf->lDelta);
//   wd = DestSurf->lDelta - ((DestRect->right - DestRect->left) << 2);
//
//   for (Y = DestRect->top; Y < DestRect->bottom; Y++)
//   {
//     SourceY = SourceRect->top+(Y - DestRect->top) * SrcHeight / DstHeight;
//     for (X = DestRect->left; X < DestRect->right; X++, DestBits++)
//     {
//       SourceX = SourceRect->left+(X - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//         SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if ((0x00FFFFFF & Source) != (0x00FFFFFF & iTransColor))
//         {
//           *DestBits = XLATEOBJ_iXlate(ColorTranslation, Source);
//         }
//       }
//     }
//
//     DestBits = (ULONG*)((ULONG_PTR)DestBits + wd);
//   }
//
//   return TRUE;
// }
//
// typedef union {
//   ULONG ul;
//   struct {
//     UCHAR red;
//     UCHAR green;
//     UCHAR blue;
//     UCHAR alpha;
//   } col;
// } NICEPIXEL32;
//
// static __inline UCHAR
// Clamp8(ULONG val)
// {
//   return (val > 255) ? 255 : (UCHAR)val;
// }
//
// BOOLEAN
// DIB_32BPP_AlphaBlend(SURFOBJ* Dest, SURFOBJ* Source, RECTL* DestRect,
//                      RECTL* SourceRect, CLIPOBJ* ClipRegion,
//                      XLATEOBJ* ColorTranslation, BLENDOBJ* BlendObj)
// {
//   INT Rows, Cols, SrcX, SrcY;
//   register PULONG Dst;
//   BLENDFUNCTION BlendFunc;
//   register NICEPIXEL32 DstPixel, SrcPixel;
//   UCHAR Alpha, SrcBpp;
//
//   DPRINT("DIB_32BPP_AlphaBlend: SourceRect: (%d,%d)-(%d,%d), DestRect: (%d,%d)-(%d,%d)\n",
//     SourceRect->left, SourceRect->top, SourceRect->right, SourceRect->bottom,
//     DestRect->left, DestRect->top, DestRect->right, DestRect->bottom);
//
//   BlendFunc = BlendObj->BlendFunction;
//   if (BlendFunc.BlendOp != AC_SRC_OVER)
//   {
//     DPRINT1("BlendOp != AC_SRC_OVER\n");
//     return FALSE;
//   }
//   if (BlendFunc.BlendFlags != 0)
//   {
//     DPRINT1("BlendFlags != 0\n");
//     return FALSE;
//   }
//   if ((BlendFunc.AlphaFormat & ~AC_SRC_ALPHA) != 0)
//   {
//     DPRINT1("Unsupported AlphaFormat (0x%x)\n", BlendFunc.AlphaFormat);
//     return FALSE;
//   }
//   if ((BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0 &&
//        BitsPerFormat(Source->iBitmapFormat) != 32)
//   {
//     DPRINT1("Source bitmap must be 32bpp when AC_SRC_ALPHA is set\n");
//     return FALSE;
//   }
//
//   Dst = (PULONG)((ULONG_PTR)Dest->pvScan0 + (DestRect->top * Dest->lDelta) +
//     (DestRect->left << 2));
//   SrcBpp = BitsPerFormat(Source->iBitmapFormat);
//
//   Rows = 0;
//    SrcY = SourceRect->top;
//    while (++Rows <= DestRect->bottom - DestRect->top)
//   {
//     Cols = 0;
//     SrcX = SourceRect->left;
//     while (++Cols <= DestRect->right - DestRect->left)
//     {
//       SrcPixel.ul = DIB_GetSource(Source, SrcX, SrcY, ColorTranslation);
//       SrcPixel.col.red = (SrcPixel.col.red * BlendFunc.SourceConstantAlpha) / 255;
//       SrcPixel.col.green = (SrcPixel.col.green * BlendFunc.SourceConstantAlpha)  / 255;
//       SrcPixel.col.blue = (SrcPixel.col.blue * BlendFunc.SourceConstantAlpha) / 255;
//       SrcPixel.col.alpha = (32 == SrcBpp) ?
//                         (SrcPixel.col.alpha * BlendFunc.SourceConstantAlpha) / 255 :
//                         BlendFunc.SourceConstantAlpha ;
//
//       Alpha = ((BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0) ?
//            SrcPixel.col.alpha : BlendFunc.SourceConstantAlpha ;
//
//       DstPixel.ul = *Dst;
//       DstPixel.col.red = Clamp8((DstPixel.col.red * (255 - Alpha)) / 255 + SrcPixel.col.red) ;
//       DstPixel.col.green = Clamp8((DstPixel.col.green * (255 - Alpha)) / 255 + SrcPixel.col.green) ;
//       DstPixel.col.blue = Clamp8((DstPixel.col.blue * (255 - Alpha)) / 255 + SrcPixel.col.blue) ;
//       DstPixel.col.alpha = Clamp8((DstPixel.col.alpha * (255 - Alpha)) / 255 + SrcPixel.col.alpha) ;
//       *Dst++ = DstPixel.ul;
//       SrcX = SourceRect->left + (Cols*(SourceRect->right - SourceRect->left))/(DestRect->right - DestRect->left);
//     }
//     Dst = (PULONG)((ULONG_PTR)Dest->pvScan0 + ((DestRect->top + Rows) * Dest->lDelta) +
//                 (DestRect->left << 2));
//     SrcY = SourceRect->top + (Rows*(SourceRect->bottom - SourceRect->top))/(DestRect->bottom - DestRect->top);
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
