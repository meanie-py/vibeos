use win32k_sys::win32k::*;

use crate::dib_helpers::*;

extern "C" {
}
extern "system" {
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
#[derive(Clone, Copy)]
pub union NICEPIXEL32 {
    pub ul: ULONG,
    pub col: NICEPIXEL32_col,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NICEPIXEL32_col {
    pub red: UCHAR,
    pub green: UCHAR,
    pub blue: UCHAR,
    pub alpha: UCHAR,
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_PutPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32, c: u32) {
    let addr = ((*SurfObj).pvScan0 as PBYTE)
        .offset((y * (*SurfObj).lDelta) as isize)
        .offset(((x << 1) + x) as isize);
    *(addr as *mut u16) = (c & 0xFFFF) as u16;
    *addr.offset(2) = ((c >> 16) & 0xFF) as u8;
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32 {
    let addr: PBYTE = ((*SurfObj).pvScan0 as PBYTE)
        .offset((y * (*SurfObj).lDelta) as isize)
        .offset(((x << 1) + x) as isize);
    *(addr as *const u16) as u32 + ((*(addr.offset(2)) as u32) << 16)
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_VLine(SurfObj: *mut SURFOBJ, x: i32, y1: i32, y2: i32, c: u32) {
    let mut addr = ((*SurfObj).pvScan0 as PBYTE)
        .offset((y1 * (*SurfObj).lDelta) as isize)
        .offset(((x << 1) + x) as isize);
    let lDelta = (*SurfObj).lDelta;
    let mut y1 = y1;
    
    let c = c & 0xFFFFFF;
    while y1 < y2 {
        y1 += 1;
        *(addr as *mut u16) = (c & 0xFFFF) as u16;
        *addr.offset(2) = (c >> 16) as u8;
        
        addr = addr.offset(lDelta as isize);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_BitBltSrcCopy(BltInfo: PBLTINFO) -> u8 {
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
    let mut SourceBits_16BPP: PWORD;
    let mut SourceLine_16BPP: PWORD;
    let bTopToBottom: u8;
    let mut bLeftToRight: u8;

    bLeftToRight = if (*BltInfo).DestRect.left > (*BltInfo).DestRect.right { 1u8 } else { 0u8 };
    bTopToBottom = if (*BltInfo).DestRect.top > (*BltInfo).DestRect.bottom { 1u8 } else { 0u8 };

    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!((*BltInfo).DestRect));

    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
        .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
        .offset(((*BltInfo).DestRect.left * 3) as isize);

    match (*(*BltInfo).SourceSurface).iBitmapFormat {
        BMF_1BPP => {
            sx = (*BltInfo).SourcePoint.x;
            sy = (*BltInfo).SourcePoint.y;

            if bTopToBottom != 0 {
                sy += (*(*BltInfo).SourceSurface).lDelta * ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1);
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
                        DIB_24BPP_PutPixel((*BltInfo).DestSurface, i, j, XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, 0));
                    } else {
                        DIB_24BPP_PutPixel((*BltInfo).DestSurface, i, j, XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, 1));
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
                SourceBits_4BPP = SourceBits_4BPP.offset(
                    ((*(*BltInfo).SourceSurface).lDelta * ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1)) as isize
                );
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceLine_4BPP = SourceBits_4BPP;
                DestLine = DestBits;
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                f1 = sx & 1;

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = XLATEOBJ_iXlate(
                        (*BltInfo).XlateSourceToDest,
                        ((*SourceLine_4BPP & altnotmask[f1 as usize]) >> (4 * (1 - f1))) as u32
                    ) as i32;
                    *DestLine = (xColor & 0xff) as u8;
                    DestLine = DestLine.offset(1);
                    *(DestLine as *mut u16) = (xColor >> 8) as u16;
                    DestLine = DestLine.offset(2);
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
                DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        BMF_8BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((*BltInfo).SourcePoint.x as isize);

            if bTopToBottom != 0 {
                SourceLine = SourceLine.offset(
                    ((*(*BltInfo).SourceSurface).lDelta * ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1)) as isize
                );
            }
            DestLine = DestBits;

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;
                DestBits = DestLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset(((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as i32;
                    *DestBits = (xColor & 0xff) as u8;
                    *(DestBits.offset(1) as *mut u16) = (xColor >> 8) as u16;
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-1) } else { SourceBits.offset(1) };
                    DestBits = DestBits.offset(3);
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
            SourceBits_16BPP = (((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((2 * (*BltInfo).SourcePoint.x) as isize)) as PWORD;

            if bTopToBottom != 0 {
                SourceBits_16BPP = SourceBits_16BPP.offset(
                    (((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize
                );
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceLine_16BPP = SourceBits_16BPP;
                DestLine = DestBits;

                if bLeftToRight != 0 {
                    SourceLine_16BPP = SourceLine_16BPP.offset(((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceLine_16BPP as u32) as i32;
                    *DestLine = (xColor & 0xff) as u8;
                    DestLine = DestLine.offset(1);
                    *(DestLine as *mut u16) = (xColor >> 8) as u16;
                    DestLine = DestLine.offset(2);
                    SourceLine_16BPP = if bLeftToRight != 0 { SourceLine_16BPP.offset(-1) } else { SourceLine_16BPP.offset(1) };
                    i += 1;
                }
                if bTopToBottom != 0 {
                    SourceBits_16BPP = ((SourceBits_16BPP as PBYTE).offset(-((*(*BltInfo).SourceSurface).lDelta as isize))) as PWORD;
                } else {
                    SourceBits_16BPP = ((SourceBits_16BPP as PBYTE).offset((*(*BltInfo).SourceSurface).lDelta as isize)) as PWORD;
                }
                DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                j += 1;
            }
        }

        BMF_24BPP => {
            if ((*BltInfo).XlateSourceToDest == core::ptr::null_mut() ||
                ((*(*BltInfo).XlateSourceToDest).flXlate & XO_TRIVIAL) != 0) &&
                (bTopToBottom == 0 && bLeftToRight == 0)
            {
                if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((3 * (*BltInfo).SourcePoint.x) as isize);
                    j = (*BltInfo).DestRect.top;
                    while j < (*BltInfo).DestRect.bottom {
                        core::ptr::copy_nonoverlapping(SourceBits, DestBits, (3 * ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left)) as usize);
                        SourceBits = SourceBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                        DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                        j += 1;
                    }
                } else {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((3 * (*BltInfo).SourcePoint.x) as isize);
                    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                        .offset((3 * (*BltInfo).DestRect.left) as isize);
                    j = (*BltInfo).DestRect.bottom - 1;
                    while (*BltInfo).DestRect.top <= j {
                        core::ptr::copy_nonoverlapping(SourceBits, DestBits, (3 * ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left)) as usize);
                        SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                        DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                        j -= 1;
                    }
                }
            } else {
                if bTopToBottom == 0 && bLeftToRight == 0 {
                    sx = (*BltInfo).SourcePoint.x;
                    sy = (*BltInfo).SourcePoint.y;

                    j = (*BltInfo).DestRect.top;
                    while j < (*BltInfo).DestRect.bottom {
                        sx = (*BltInfo).SourcePoint.x;
                        i = (*BltInfo).DestRect.left;
                        while i < (*BltInfo).DestRect.right {
                            let pixel = DIB_24BPP_GetPixel((*BltInfo).SourceSurface, sx, sy);
                            DIB_24BPP_PutPixel((*BltInfo).DestSurface, i, j, XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, pixel));
                            sx += 1;
                            i += 1;
                        }
                        sy += 1;
                        j += 1;
                    }
                } else {
                    let mut TopToBottomDone = 0u8;

                    if bLeftToRight != 0 {
                        let mut Index: u32;
                        let store = ExAllocatePoolWithTag(
                            NonPagedPool,
                            (((*BltInfo).DestRect.right - (*BltInfo).DestRect.left + 1) * 4) as SIZE_T,
                            0x20424944
                        ) as PDWORD;
                        if store == core::ptr::null_mut() {
                            return 0u8;
                        }

                        sx = (*BltInfo).SourcePoint.x;
                        sy = (*BltInfo).SourcePoint.y;
                        sx = (*BltInfo).SourcePoint.x + ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1);

                        j = (*BltInfo).DestRect.top;
                        while j < (*BltInfo).DestRect.bottom {
                            sx = (*BltInfo).SourcePoint.x + ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1);
                            Index = 0;

                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                *store.offset(Index as isize) = DIB_24BPP_GetPixel((*BltInfo).SourceSurface, sx, sy);
                                Index += 1;
                                sx -= 1;
                                i += 1;
                            }

                            Index = 0;

                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                DIB_24BPP_PutPixel((*BltInfo).DestSurface, i, j, XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *store.offset(Index as isize)));
                                Index += 1;
                                i += 1;
                            }
                            sy += 1;
                            j += 1;
                        }
                        ExFreePoolWithTag(store as *mut core::ffi::c_void, 0x20424944);
                        TopToBottomDone = 1u8;
                    }

                    if bTopToBottom != 0 {
                        let mut Index: u32;
                        let store = ExAllocatePoolWithTag(
                            NonPagedPool,
                            (((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top + 1) * 4) as SIZE_T,
                            0x20424944
                        ) as PDWORD;
                        if store == core::ptr::null_mut() {
                            return 0u8;
                        }

                        if TopToBottomDone != 0 {
                            sx = (*BltInfo).DestRect.left;
                            sy = (*BltInfo).DestRect.top;
                            sy += (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1;
                        } else {
                            sx = (*BltInfo).SourcePoint.x;
                            sy = (*BltInfo).SourcePoint.y;
                            sy = (*BltInfo).SourcePoint.y + ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1);
                        }

                        i = (*BltInfo).DestRect.left;
                        while i < (*BltInfo).DestRect.right {
                            sy = (*BltInfo).SourcePoint.y + ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1);
                            Index = 0;

                            j = (*BltInfo).DestRect.top;
                            while j < (*BltInfo).DestRect.bottom {
                                if TopToBottomDone != 0 {
                                    *store.offset(Index as isize) = DIB_24BPP_GetPixel((*BltInfo).DestSurface, sx, sy);
                                } else {
                                    *store.offset(Index as isize) = DIB_24BPP_GetPixel((*BltInfo).SourceSurface, sx, sy);
                                }
                                Index += 1;
                                sy -= 1;
                                j += 1;
                            }

                            Index = 0;

                            j = (*BltInfo).DestRect.top;
                            while j < (*BltInfo).DestRect.bottom {
                                DIB_24BPP_PutPixel((*BltInfo).DestSurface, i, j, XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *store.offset(Index as isize)));
                                Index += 1;
                                j += 1;
                            }
                            sx += 1;
                            i += 1;
                        }
                        ExFreePoolWithTag(store as *mut core::ffi::c_void, 0x20424944);
                    }
                }
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
                    xColor = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *(SourceBits as PDWORD)) as i32;
                    *DestBits = (xColor & 0xff) as u8;
                    *(DestBits.offset(1) as *mut u16) = (xColor >> 8) as u16;
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-4) } else { SourceBits.offset(4) };
                    DestBits = DestBits.offset(3);
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

        _ => {
            DbgPrint(b"DIB_24BPP_Bitblt: Unhandled Source BPP: %u\n\0".as_ptr() as *const u8, BitsPerFormat((*(*BltInfo).SourceSurface).iBitmapFormat));
            return 0u8;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_BitBlt(BltInfo: PBLTINFO) -> u8 {
    let mut DestX: i32;
    let mut DestY: i32;
    let mut SourceX: i32;
    let mut SourceY: i32;
    let mut PatternY: i32 = 0;
    let mut Dest: u32;
    let mut Source: u32 = 0;
    let mut Pattern: u32 = 0;
    let UsesSource: u32;
    let UsesPattern: u32;
    let mut DestBits: PBYTE;

    UsesSource = ROP4_USES_SOURCE((*BltInfo).Rop4);
    UsesPattern = ROP4_USES_PATTERN((*BltInfo).Rop4);

    SourceY = (*BltInfo).SourcePoint.y;
    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
        .offset((((*BltInfo).DestRect.left << 1) + (*BltInfo).DestRect.left) as isize)
        .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize);

    if UsesPattern != 0 {
        if !(*BltInfo).PatternSurface.is_null() {
            PatternY = ((*BltInfo).DestRect.top - (*BltInfo).BrushOrigin.y)
                % (*(*BltInfo).PatternSurface).sizlBitmap.cy;
        } else {
            if !(*BltInfo).Brush.is_null() {
                Pattern = (*(*BltInfo).Brush).iSolidColor;
            }
        }
    }

    DestY = (*BltInfo).DestRect.top;
    while DestY < (*BltInfo).DestRect.bottom {
        SourceX = (*BltInfo).SourcePoint.x;

        DestX = (*BltInfo).DestRect.left;
        while DestX < (*BltInfo).DestRect.right {
            Dest = *(DestBits as *const u16) as u32 + ((*DestBits.offset(2) as u32) << 16);

            if UsesSource != 0 {
                Source = DIB_GetSource(
                    (*BltInfo).SourceSurface,
                    SourceX,
                    SourceY,
                    (*BltInfo).XlateSourceToDest,
                );
            }

            if !(*BltInfo).PatternSurface.is_null() {
                Pattern = DIB_GetSourceIndex(
                    (*BltInfo).PatternSurface,
                    (DestX - (*BltInfo).BrushOrigin.x) % (*(*BltInfo).PatternSurface).sizlBitmap.cx,
                    PatternY,
                );
            }

            Dest = DIB_DoRop((*BltInfo).Rop4, Dest, Source, Pattern) & 0xFFFFFF;
            *(DestBits as *mut u16) = (Dest & 0xFFFF) as u16;
            *DestBits.offset(2) = (Dest >> 16) as u8;

            DestX += 1;
            DestBits = DestBits.offset(3);
            SourceX += 1;
        }

        SourceY += 1;
        if !(*BltInfo).PatternSurface.is_null() {
            PatternY += 1;
            PatternY %= (*(*BltInfo).PatternSurface).sizlBitmap.cy;
        }
        DestBits = DestBits.offset(-(((*BltInfo).DestRect.right - (*BltInfo).DestRect.left) * 3) as isize);
        DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);

        DestY += 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_ColorFill(DestSurface: *mut SURFOBJ, DestRect: *mut RECTL, color: u32) -> u8 {
    let mut DestY: i32;

    RECTL_vMakeWellOrdered(DestRect);

    // C original has an i386 GCC-asm fast-fill (12 bytes / 4 pixels per
    // iteration); the rustconv translation used AT&T syntax in a default-Intel
    // asm! block, clobbered LLVM-reserved ebx, and used 0/1-only labels — all
    // rejected. It is a micro-opt of this portable HLine loop (used by
    // amd64/arm64) — keep portable on all arches for correctness.
    {
        DestY = (*DestRect).top;
        while DestY < (*DestRect).bottom {
            DIB_24BPP_HLine(DestSurface, (*DestRect).left, (*DestRect).right, DestY, color);
            DestY += 1;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_TransparentBlt(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ColorTranslation: *mut XLATEOBJ,
    iTransColor: u32,
) -> u8 {
    let mut X: i32;
    let mut Y: i32;
    let mut SourceX: i32;
    let mut SourceY: i32 = 0;
    let wd: i32;
    let mut Source: u32 = 0;
    let mut Dest: u32;
    let mut DestBits: *mut u8;

    let DstHeight: i32;
    let DstWidth: i32;
    let SrcHeight: i32;
    let SrcWidth: i32;

    DstHeight = (*DestRect).bottom - (*DestRect).top;
    DstWidth = (*DestRect).right - (*DestRect).left;
    SrcHeight = (*SourceRect).bottom - (*SourceRect).top;
    SrcWidth = (*SourceRect).right - (*SourceRect).left;

    DestBits = ((*DestSurf).pvScan0 as PBYTE)
        .offset(((*DestRect).left * 3) as isize)
        .offset(((*DestRect).top * (*DestSurf).lDelta) as isize);
    wd = (*DestSurf).lDelta - (((*DestRect).right - (*DestRect).left) * 3);

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
                if Source != iTransColor {
                    Dest = XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFFFFFF;
                    *(DestBits as *mut u16) = (Dest & 0xFFFF) as u16;
                    *(DestBits.offset(2)) = (Dest >> 16) as u8;
                }
            }
            DestBits = DestBits.offset(3);
            X += 1;
        }

        DestBits = (DestBits as usize + wd as usize) as *mut u8;
        Y += 1;
    }

    1u8
}

unsafe fn Clamp8(val: ULONG) -> UCHAR {
    if val > 255 { 255 } else { val as UCHAR }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_AlphaBlend(
    Dest: *mut SURFOBJ,
    Source: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    BlendObj: *mut BLENDOBJ
) -> BOOLEAN {
    let mut Rows: INT;
    let mut Cols: INT;
    let mut SrcX: INT;
    let mut SrcY: INT;
    let mut Dst: PUCHAR;
    let BlendFunc: BLENDFUNCTION;
    let mut DstPixel: NICEPIXEL32 = core::mem::zeroed();
    let mut SrcPixel: NICEPIXEL32 = core::mem::zeroed();
    let mut Alpha: UCHAR;

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
    if (BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0 &&
       BitsPerFormat((*Source).iBitmapFormat) != 32 {
        return 0u8;
    }

    Dst = ((*Dest).pvScan0 as ULONG_PTR + 
           ((*DestRect).top * (*Dest).lDelta) as ULONG_PTR +
           ((*DestRect).left * 3) as ULONG_PTR) as PUCHAR;

    Rows = 0;
    SrcY = (*SourceRect).top;
    while { Rows += 1; Rows } <= (*DestRect).bottom - (*DestRect).top {
        Cols = 0;
        SrcX = (*SourceRect).left;
        while { Cols += 1; Cols } <= (*DestRect).right - (*DestRect).left {
            SrcPixel.ul = DIB_GetSource(Source, SrcX, SrcY, ColorTranslation);
            SrcPixel.col.red = (SrcPixel.col.red as ULONG * BlendFunc.SourceConstantAlpha as ULONG / 255) as UCHAR;
            SrcPixel.col.green = (SrcPixel.col.green as ULONG * BlendFunc.SourceConstantAlpha as ULONG / 255) as UCHAR;
            SrcPixel.col.blue = (SrcPixel.col.blue as ULONG * BlendFunc.SourceConstantAlpha as ULONG / 255) as UCHAR;
            if (BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) == 0 {
                Alpha = BlendFunc.SourceConstantAlpha;
            } else {
                Alpha = (SrcPixel.col.alpha as ULONG * BlendFunc.SourceConstantAlpha as ULONG / 255) as UCHAR;
            }

            DstPixel.col.red = Clamp8((*Dst as ULONG * (255 - Alpha as ULONG)) / 255 + SrcPixel.col.red as ULONG);
            DstPixel.col.green = Clamp8(*Dst.offset(1) as ULONG * (255 - Alpha as ULONG) / 255 + SrcPixel.col.green as ULONG );
            DstPixel.col.blue = Clamp8((*Dst.offset(2) as ULONG * (255 - Alpha as ULONG)) / 255 + SrcPixel.col.blue as ULONG);
            *Dst = DstPixel.col.red;
            Dst = Dst.offset(1);
            *Dst = DstPixel.col.green;
            Dst = Dst.offset(1);
            *Dst = DstPixel.col.blue;
            Dst = Dst.offset(1);
            SrcX = (*SourceRect).left + (Cols * ((*SourceRect).right - (*SourceRect).left)) / ((*DestRect).right - (*DestRect).left);
        }
        Dst = ((*Dest).pvScan0 as ULONG_PTR + 
               (((*DestRect).top + Rows) * (*Dest).lDelta) as ULONG_PTR +
               ((*DestRect).left * 3) as ULONG_PTR) as PUCHAR;
        SrcY = (*SourceRect).top + (Rows * ((*SourceRect).bottom - (*SourceRect).top)) / ((*DestRect).bottom - (*DestRect).top);
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
//  * FILE:            win32ss/gdi/dib/dib24bpp.c
//  * PURPOSE:         Device Independant Bitmap functions, 24bpp
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
// DIB_24BPP_PutPixel(SURFOBJ *SurfObj, LONG x, LONG y, ULONG c)
// {
//   PBYTE addr = (PBYTE)SurfObj->pvScan0 + (y * SurfObj->lDelta) + (x << 1) + x;
//   *(PUSHORT)(addr) = c & 0xFFFF;
//   *(addr + 2) = (c >> 16) & 0xFF;
// }
//
// ULONG
// DIB_24BPP_GetPixel(SURFOBJ *SurfObj, LONG x, LONG y)
// {
//   PBYTE addr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta + (x << 1) + x;
//   return *(PUSHORT)(addr) + (*(addr + 2) << 16);
// }
//
//
//
// VOID
// DIB_24BPP_VLine(SURFOBJ *SurfObj, LONG x, LONG y1, LONG y2, ULONG c)
// {
//   PBYTE addr = (PBYTE)SurfObj->pvScan0 + y1 * SurfObj->lDelta + (x << 1) + x;
//   LONG lDelta = SurfObj->lDelta;
//
//   c &= 0xFFFFFF;
//   while(y1++ < y2)
//   {
//     *(PUSHORT)(addr) = c & 0xFFFF;
//     *(addr + 2) = (BYTE)(c >> 16);
//
//     addr += lDelta;
//   }
// }
//
// BOOLEAN
// DIB_24BPP_BitBltSrcCopy(PBLTINFO BltInfo)
// {
//   LONG     i, j, sx, sy, xColor, f1;
//   PBYTE    SourceBits, DestBits, SourceLine, DestLine;
//   PBYTE    SourceBits_4BPP, SourceLine_4BPP;
//   PWORD    SourceBits_16BPP, SourceLine_16BPP;
//   BOOLEAN  bTopToBottom, bLeftToRight;
//
//   DPRINT("DIB_24BPP_BitBltSrcCopy: SrcSurf cx/cy (%d/%d), DestSuft cx/cy (%d/%d) dstRect: (%d,%d)-(%d,%d)\n",
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
//   DPRINT("BltInfo->SourcePoint.x is '%d' and BltInfo->SourcePoint.y is '%d'.\n",
//          BltInfo->SourcePoint.x, BltInfo->SourcePoint.y);
//
//   /* Make WellOrdered by making top < bottom and left < right */
//   RECTL_vMakeWellOrdered(&BltInfo->DestRect);
//
//   DestBits = (PBYTE)BltInfo->DestSurface->pvScan0 + (BltInfo->DestRect.top * BltInfo->DestSurface->lDelta) + BltInfo->DestRect.left * 3;
//
//   switch(BltInfo->SourceSurface->iBitmapFormat)
//   {
//     case BMF_1BPP:
//       sx = BltInfo->SourcePoint.x;
//
//       /* This sets sy to the top line */
//       sy = BltInfo->SourcePoint.y;
//
//       if (bTopToBottom)
//       {
//         /* This sets sy to the bottom line */
//         sy += BltInfo->SourceSurface->lDelta * (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//       }
//
//       for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//       {
//         sx = BltInfo->SourcePoint.x;
//
//         if (bLeftToRight)
//         {
//           /* This sets sx to the rightmost pixel */
//           sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//         }
//
//         for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//         {
//           if(DIB_1BPP_GetPixel(BltInfo->SourceSurface, sx, sy) == 0)
//           {
//             DIB_24BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 0));
//           } else {
//             DIB_24BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 1));
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
//         SourceBits_4BPP += BltInfo->SourceSurface->lDelta * (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//       }
//
//       for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//       {
//         SourceLine_4BPP = SourceBits_4BPP;
//         DestLine = DestBits;
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
//           *DestLine++ = xColor & 0xff;
//           *(PWORD)DestLine = (WORD)(xColor >> 8);
//           DestLine += 2;
//           if(f1 == 1) {
//             DEC_OR_INC(SourceLine_4BPP, bLeftToRight, 1);
//             f1 = 0;
//           }
//           else
//           {
//             f1 = 1;
//           }
//           DEC_OR_INC(sx, bLeftToRight, 1);
//         }
//         DEC_OR_INC(SourceBits_4BPP, bTopToBottom, BltInfo->SourceSurface->lDelta);
//         DestBits += BltInfo->DestSurface->lDelta;
//       }
//       break;
//
//     case BMF_8BPP:
//       DPRINT("8BPP Case Selected with DestRect Width of '%d'.\n",
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       /* This sets SourceLine to the top line */
//       SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + BltInfo->SourcePoint.x;
//
//       if (bTopToBottom)
//       {
//         /* This sets SourceLine to the bottom line */
//         SourceLine += BltInfo->SourceSurface->lDelta * (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
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
//           /* This sets the SourceBits to the rightmost pixel */
//           SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//         }
//
//         for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//         {
//           xColor = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *SourceBits);
//           *DestBits = xColor & 0xff;
//           *(PWORD)(DestBits + 1) = (WORD)(xColor >> 8);
//           DEC_OR_INC(SourceBits, bLeftToRight, 1);
//           DestBits += 3;
//         }
//
//         DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//         DestLine += BltInfo->DestSurface->lDelta;
//       }
//       break;
//
//     case BMF_16BPP:
//       DPRINT("16BPP Case Selected with DestRect Width of '%d'.\n",
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       /* This sets SourceBits_16BPP to the top line */
//       SourceBits_16BPP = (PWORD)((PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + 2 * BltInfo->SourcePoint.x);
//
//       if (bTopToBottom)
//       {
//         /* This sets SourceBits_16BPP to the bottom line */
//         SourceBits_16BPP += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;
//       }
//
//       for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//       {
//         SourceLine_16BPP = SourceBits_16BPP;
//         DestLine = DestBits;
//
//         if (bLeftToRight)
//         {
//           /* This sets the SourceLine_16BPP to the rightmost pixel */
//           SourceLine_16BPP += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//         }
//
//         for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//         {
//           xColor = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *SourceLine_16BPP);
//           *DestLine++ = xColor & 0xff;
//           *(PWORD)DestLine = (WORD)(xColor >> 8);
//           DestLine += 2;
//           DEC_OR_INC(SourceLine_16BPP, bLeftToRight, 1);
//         }
//         if (bTopToBottom)
//         {
//           SourceBits_16BPP = (PWORD)((PBYTE)SourceBits_16BPP - BltInfo->SourceSurface->lDelta);
//         }
//         else
//         {
//           SourceBits_16BPP = (PWORD)((PBYTE)SourceBits_16BPP + BltInfo->SourceSurface->lDelta);
//         }
//         DestBits += BltInfo->DestSurface->lDelta;
//       }
//       break;
//
//     case BMF_24BPP:
//       DPRINT("24BPP Case Selected with DestRect Width of '%d'.\n",
//              BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       /* Check for no flips here because we are about to use RtlMoveMemory and it can only do increasing src & dst */
//       if ((BltInfo->XlateSourceToDest == NULL ||
//         (BltInfo->XlateSourceToDest->flXlate & XO_TRIVIAL) != 0) &&
//         (!bTopToBottom && !bLeftToRight))
//       {
//         DPRINT("XO_TRIVIAL is TRUE.\n");
//         if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//         {
//           /* This sets SourceBits to the top line */
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0 + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) + 3 * BltInfo->SourcePoint.x;
//           for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//           {
//             RtlMoveMemory(DestBits, SourceBits, 3 * (BltInfo->DestRect.right - BltInfo->DestRect.left));
//             SourceBits += BltInfo->SourceSurface->lDelta;
//             DestBits += BltInfo->DestSurface->lDelta;
//           }
//         }
//         else
//         {
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0 + ((BltInfo->SourcePoint.y + BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta) + 3 * BltInfo->SourcePoint.x;
//           DestBits = (PBYTE)BltInfo->DestSurface->pvScan0 + ((BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta) + 3 * BltInfo->DestRect.left;
//           for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//           {
//             RtlMoveMemory(DestBits, SourceBits, 3 * (BltInfo->DestRect.right - BltInfo->DestRect.left));
//             SourceBits -= BltInfo->SourceSurface->lDelta;
//             DestBits -= BltInfo->DestSurface->lDelta;
//           }
//         }
//       }
//       else
//       {
//         DPRINT("XO_TRIVIAL is NOT TRUE.\n");
//
//         if (!bTopToBottom && !bLeftToRight)
//       /* **Note: Indent is purposefully less than desired to keep reviewable differences to a minimum for PR** */
//       {
//         sx = BltInfo->SourcePoint.x;
//         sy = BltInfo->SourcePoint.y;
//
//         for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//         {
//           sx = BltInfo->SourcePoint.x;
//           for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//           {
//             DWORD pixel = DIB_24BPP_GetPixel(BltInfo->SourceSurface, sx, sy);
//             DIB_24BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, pixel));
//             sx++;
//           }
//           sy++;
//         }
//       }
//         else
//         {
//           /* Buffering for source and destination flip overlaps. Fixes KHMZ MirrorTest CORE-16642 */
//           BOOL TopToBottomDone = FALSE;
//
//           if (bLeftToRight)
//           {
//             DPRINT("Flip is bLeftToRight.\n");
//             DWORD  Index;
//
//             /* Allocate enough pixels for a row in DWORD's */
//             DWORD *store = ExAllocatePoolWithTag(NonPagedPool,
//               (BltInfo->DestRect.right - BltInfo->DestRect.left + 1) * 4, TAG_DIB);
//             if (store == NULL)
//             {
//               DPRINT1("Storage Allocation Failed.\n");
//               return FALSE;
//             }
//
//             sx = BltInfo->SourcePoint.x;
//             /* This sets sy to the top line */
//             sy = BltInfo->SourcePoint.y;
//
//             /* This sets sx to the rightmost pixel */
//             sx = BltInfo->SourcePoint.x + (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//
//             for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//             {
//
//               /* This sets sx to the rightmost pixel */
//               sx = BltInfo->SourcePoint.x + (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//
//               Index = 0;
//
//               // Read right to left and store
//               for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//               {
//                 store[Index] = DIB_24BPP_GetPixel(BltInfo->SourceSurface, sx, sy);
//                 Index++;
//                 sx--;
//               }
//
//               Index = 0;
//
//               // Write left to right to pixel
//               for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//               {
//                 DIB_24BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, store[Index]));
//                 Index++;
//               }
//               sy++;
//             }
//             ExFreePoolWithTag(store, TAG_DIB);
//             TopToBottomDone = TRUE;
//           }
//
//           if (bTopToBottom)
//           {
//             DPRINT("Flip is bTopToBottom.\n");
//             DWORD  Index;
//
//             /* Allocate enough pixels for a column in DWORD's */
//             DWORD *store = ExAllocatePoolWithTag(NonPagedPool,
//               (BltInfo->DestRect.bottom - BltInfo->DestRect.top + 1) * 4, TAG_DIB);
//             if (store == NULL)
//             {
//               DPRINT1("Storage Allocation Failed.\n");
//               return FALSE;
//             }
//
//             /* The TopToBottomDone flag indicates that we are flipping for bTopToBottom and bLeftToRight
//              * and have already completed the bLeftToRight. So we will lose our first flip output
//              * unless we work with its output which is at the destination site. So in this case
//              * our new Source becomes the previous outputs Destination.
//              */
//
//             if (TopToBottomDone)
//             {
//              sx = BltInfo->DestRect.left;
//               sy = BltInfo->DestRect.top;
//
//               /* This sets sy to the bottom line */
//               sy += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//             }
//             else
//             {
//               sx = BltInfo->SourcePoint.x;
//
//               /* This sets sy to the top line */
//               sy = BltInfo->SourcePoint.y;
//
//               /* This sets sy to the bottom line */
//               sy = BltInfo->SourcePoint.y + (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//             }
//
//             for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//             {
//
//               /* This sets sy to the bottom line */
//               sy = BltInfo->SourcePoint.y + (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//               Index = 0;
//
//               /* Read bottom to top and store */
//               for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//               {
//                 if (TopToBottomDone)
//                 {
//                   store[Index] = DIB_24BPP_GetPixel(BltInfo->DestSurface, sx, sy);
//                 }
//                 else
//                 {
//                   store[Index] = DIB_24BPP_GetPixel(BltInfo->SourceSurface, sx, sy);
//                 }
//                 Index++;
//                 sy--;
//               }
//
//               Index = 0;
//
//               for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//               {
//                 DIB_24BPP_PutPixel(BltInfo->DestSurface, i, j, XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, store[Index]));
//                 Index++;
//               }
//               sx++;
//             }
//             ExFreePoolWithTag(store, TAG_DIB);
//           }
//
//         }
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
//         for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//         {
//           xColor = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *((PDWORD) SourceBits));
//           *DestBits = xColor & 0xff;
//           *(PWORD)(DestBits + 1) = (WORD)(xColor >> 8);
//           DEC_OR_INC(SourceBits, bLeftToRight, 4);
//           DestBits += 3;
//         }
//
//         DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//         DestLine += BltInfo->DestSurface->lDelta;
//       }
//       break;
//
//     default:
//       DbgPrint("DIB_24BPP_Bitblt: Unhandled Source BPP: %u\n", BitsPerFormat(BltInfo->SourceSurface->iBitmapFormat));
//       return FALSE;
//   }
//
//   return TRUE;
// }
//
// BOOLEAN
// DIB_24BPP_BitBlt(PBLTINFO BltInfo)
// {
//    LONG DestX, DestY;
//    LONG SourceX, SourceY;
//    LONG PatternY = 0;
//    ULONG Dest, Source = 0, Pattern = 0;
//    BOOL UsesSource;
//    BOOL UsesPattern;
//    PBYTE DestBits;
//
//    UsesSource = ROP4_USES_SOURCE(BltInfo->Rop4);
//    UsesPattern = ROP4_USES_PATTERN(BltInfo->Rop4);
//
//    SourceY = BltInfo->SourcePoint.y;
//    DestBits = (PBYTE)(
//       (PBYTE)BltInfo->DestSurface->pvScan0 +
//       (BltInfo->DestRect.left << 1) + BltInfo->DestRect.left +
//       BltInfo->DestRect.top * BltInfo->DestSurface->lDelta);
//
//    if (UsesPattern)
//    {
//       if (BltInfo->PatternSurface)
//       {
//          PatternY = (BltInfo->DestRect.top - BltInfo->BrushOrigin.y) %
//                     BltInfo->PatternSurface->sizlBitmap.cy;
//       }
//       else
//       {
//          if (BltInfo->Brush)
//          {
//             Pattern = BltInfo->Brush->iSolidColor;
//          }
//       }
//    }
//
//    for (DestY = BltInfo->DestRect.top; DestY < BltInfo->DestRect.bottom; DestY++)
//    {
//       SourceX = BltInfo->SourcePoint.x;
//
//       for (DestX = BltInfo->DestRect.left; DestX < BltInfo->DestRect.right; DestX++, DestBits += 3, SourceX++)
//       {
//          Dest = *((PUSHORT)DestBits) + (*(DestBits + 2) << 16);
//
//          if (UsesSource)
//          {
//             Source = DIB_GetSource(BltInfo->SourceSurface, SourceX, SourceY, BltInfo->XlateSourceToDest);
//          }
//
//          if (BltInfo->PatternSurface)
//          {
//             Pattern = DIB_GetSourceIndex(BltInfo->PatternSurface, (DestX - BltInfo->BrushOrigin.x) % BltInfo->PatternSurface->sizlBitmap.cx, PatternY);
//          }
//
//          Dest = DIB_DoRop(BltInfo->Rop4, Dest, Source, Pattern) & 0xFFFFFF;
//          *(PUSHORT)(DestBits) = Dest & 0xFFFF;
//          *(DestBits + 2) = (BYTE)(Dest >> 16);
//       }
//
//       SourceY++;
//       if (BltInfo->PatternSurface)
//       {
//          PatternY++;
//          PatternY %= BltInfo->PatternSurface->sizlBitmap.cy;
//       }
//       DestBits -= (BltInfo->DestRect.right - BltInfo->DestRect.left) * 3;
//       DestBits += BltInfo->DestSurface->lDelta;
//    }
//
//    return TRUE;
// }
//
// /* BitBlt Optimize */
// BOOLEAN
// DIB_24BPP_ColorFill(SURFOBJ* DestSurface, RECTL* DestRect, ULONG color)
// {
//   LONG DestY;
//
//   /* Make WellOrdered by making top < bottom and left < right */
//   RECTL_vMakeWellOrdered(DestRect);
//
// #if defined(_M_IX86) && !defined(_MSC_VER)
//   PBYTE xaddr = (PBYTE)DestSurface->pvScan0 + DestRect->top * DestSurface->lDelta + (DestRect->left << 1) + DestRect->left;
//   PBYTE addr;
//   ULONG Count;
//   ULONG xCount=DestRect->right - DestRect->left;
//
//   for (DestY = DestRect->top; DestY< DestRect->bottom; DestY++)
//   {
//     Count = xCount;
//     addr = xaddr;
//     xaddr = (PBYTE)((ULONG_PTR)addr + DestSurface->lDelta);
//
//     if (Count < 8)
//     {
//       /* For small fills, don't bother doing anything fancy */
//       while (Count--)
//         {
//           *(PUSHORT)(addr) = color;
//           addr += 2;
//           *(addr) = color >> 16;
//           addr += 1;
//         }
//     }
//     else
//     {
//       /* Align to 4-byte address */
//       while (0 != ((ULONG_PTR) addr & 0x3))
//       {
//         *(PUSHORT)(addr) = color;
//         addr += 2;
//         *(addr) = color >> 16;
//         addr += 1;
//         Count--;
//       }
//       /* If the color we need to fill with is 0ABC, then the final mem pattern
//        * (note little-endianness) would be:
//        *
//        * |C.B.A|C.B.A|C.B.A|C.B.A|   <- pixel borders
//        * |C.B.A.C|B.A.C.B|A.C.B.A|   <- ULONG borders
//        *
//        * So, taking endianness into account again, we need to fill with these
//        * ULONGs: CABC BCAB ABCA */
//
//       /* This is about 30% faster than the generic C code below */
//       __asm__ __volatile__ (
//         "movl %1, %%ecx\n\t"
//         "andl $0xffffff, %%ecx\n\t"     /* 0ABC */
//         "movl %%ecx, %%ebx\n\t"         /* Construct BCAB in ebx */
//         "shrl $8, %%ebx\n\t"
//         "movl %%ecx, %%eax\n\t"
//         "shll $16, %%eax\n\t"
//         "orl  %%eax, %%ebx\n\t"
//         "movl %%ecx, %%edx\n\t"         /* Construct ABCA in edx */
//         "shll $8, %%edx\n\t"
//         "movl %%ecx, %%eax\n\t"
//         "shrl $16, %%eax\n\t"
//         "orl  %%eax, %%edx\n\t"
//         "movl %%ecx, %%eax\n\t"         /* Construct CABC in eax */
//         "shll $24, %%eax\n\t"
//         "orl  %%ecx, %%eax\n\t"
//         "movl %2, %%ecx\n\t"            /* Load count */
//         "shr  $2, %%ecx\n\t"
//         "movl %3, %%edi\n"              /* Load dest */
//         "1:\n\t"
//         "movl %%eax, (%%edi)\n\t"       /* Store 4 pixels, 12 bytes */
//         "movl %%ebx, 4(%%edi)\n\t"
//         "movl %%edx, 8(%%edi)\n\t"
//         "addl $12, %%edi\n\t"
//         "dec  %%ecx\n\t"
//         "jnz  1b\n\t"
//         "movl %%edi, %0"
//         : "=m"(addr)
//         : "m"(color), "m"(Count), "m"(addr)
//         : "%eax", "%ebx", "%ecx", "%edx", "%edi");
//       Count = Count & 0x03;
//       while (0 != Count--)
//       {
//         *(PUSHORT)(addr) = color;
//         addr += 2;
//         *(addr) = color >> 16;
//         addr += 1;
//       }
//     }
//   }
// #else
//
//   for (DestY = DestRect->top; DestY< DestRect->bottom; DestY++)
//   {
//     DIB_24BPP_HLine(DestSurface, DestRect->left, DestRect->right, DestY, color);
//   }
// #endif
//   return TRUE;
// }
//
// BOOLEAN
// DIB_24BPP_TransparentBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf,
//                          RECTL*  DestRect,  RECTL *SourceRect,
//                          XLATEOBJ *ColorTranslation, ULONG iTransColor)
// {
//   LONG X, Y, SourceX, SourceY = 0, wd;
//   ULONG Source = 0, Dest;
//   BYTE *DestBits;
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
//   DestBits = (BYTE*)((PBYTE)DestSurf->pvScan0 +
//                       (DestRect->left * 3) +
//                       DestRect->top * DestSurf->lDelta);
//   wd = DestSurf->lDelta - ((DestRect->right - DestRect->left) * 3);
//
//   for(Y = DestRect->top; Y < DestRect->bottom; Y++)
//   {
//     SourceY = SourceRect->top+(Y - DestRect->top) * SrcHeight / DstHeight;
//     for(X = DestRect->left; X < DestRect->right; X++, DestBits += 3)
//     {
//       SourceX = SourceRect->left+(X - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//           SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           Dest = XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFFFFFF;
//            *(PUSHORT)(DestBits) = Dest & 0xFFFF;
//            *(DestBits + 2) = (BYTE)(Dest >> 16);
//         }
//       }
//     }
//
//     DestBits = (BYTE*)((ULONG_PTR)DestBits + wd);
//   }
//
//   return TRUE;
// }
//
// typedef union {
//    ULONG ul;
//    struct {
//       UCHAR red;
//       UCHAR green;
//       UCHAR blue;
//       UCHAR alpha;
//    } col;
// } NICEPIXEL32;
//
// static __inline UCHAR
// Clamp8(ULONG val)
// {
//    return (val > 255) ? 255 : (UCHAR)val;
// }
//
// BOOLEAN
// DIB_24BPP_AlphaBlend(SURFOBJ* Dest, SURFOBJ* Source, RECTL* DestRect,
//                      RECTL* SourceRect, CLIPOBJ* ClipRegion,
//                      XLATEOBJ* ColorTranslation, BLENDOBJ* BlendObj)
// {
//    INT Rows, Cols, SrcX, SrcY;
//    register PUCHAR Dst;
//    BLENDFUNCTION BlendFunc;
//    register NICEPIXEL32 DstPixel, SrcPixel;
//    UCHAR Alpha;
//    //UCHAR SrcBpp;
//
//    DPRINT("DIB_24BPP_AlphaBlend: srcRect: (%d,%d)-(%d,%d), dstRect: (%d,%d)-(%d,%d)\n",
//           SourceRect->left, SourceRect->top, SourceRect->right, SourceRect->bottom,
//           DestRect->left, DestRect->top, DestRect->right, DestRect->bottom);
//
//    BlendFunc = BlendObj->BlendFunction;
//    if (BlendFunc.BlendOp != AC_SRC_OVER)
//    {
//       DPRINT1("BlendOp != AC_SRC_OVER\n");
//       return FALSE;
//    }
//    if (BlendFunc.BlendFlags != 0)
//    {
//       DPRINT1("BlendFlags != 0\n");
//       return FALSE;
//    }
//    if ((BlendFunc.AlphaFormat & ~AC_SRC_ALPHA) != 0)
//    {
//       DPRINT1("Unsupported AlphaFormat (0x%x)\n", BlendFunc.AlphaFormat);
//       return FALSE;
//    }
//    if ((BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0 &&
//        BitsPerFormat(Source->iBitmapFormat) != 32)
//    {
//       DPRINT1("Source bitmap must be 32bpp when AC_SRC_ALPHA is set\n");
//       return FALSE;
//    }
//
//    Dst = (PUCHAR)((ULONG_PTR)Dest->pvScan0 + (DestRect->top * Dest->lDelta) +
//                              (DestRect->left * 3));
//    //SrcBpp = BitsPerFormat(Source->iBitmapFormat);
//
//    Rows = 0;
//    SrcY = SourceRect->top;
//    while (++Rows <= DestRect->bottom - DestRect->top)
//   {
//     Cols = 0;
//     SrcX = SourceRect->left;
//     while (++Cols <= DestRect->right - DestRect->left)
//     {
//       SrcPixel.ul = DIB_GetSource(Source, SrcX, SrcY, ColorTranslation);
//       SrcPixel.col.red = (SrcPixel.col.red * BlendFunc.SourceConstantAlpha) / 255;
//       SrcPixel.col.green = (SrcPixel.col.green * BlendFunc.SourceConstantAlpha) / 255;
//       SrcPixel.col.blue = (SrcPixel.col.blue * BlendFunc.SourceConstantAlpha) / 255;
//       if (!(BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA))
//       {
//           Alpha = BlendFunc.SourceConstantAlpha ;
//       }
//       else
//       {
//         Alpha = (SrcPixel.col.alpha * BlendFunc.SourceConstantAlpha) / 255;
//       }
//
//       DstPixel.col.red = Clamp8((*Dst * (255 - Alpha)) / 255 + SrcPixel.col.red) ;
//       DstPixel.col.green = Clamp8((*(Dst+1) * (255 - Alpha) / 255 + SrcPixel.col.green)) ;
//       DstPixel.col.blue = Clamp8((*(Dst+2) * (255 - Alpha)) / 255 + SrcPixel.col.blue) ;
//       *Dst++ = DstPixel.col.red;
//       *Dst++ = DstPixel.col.green;
//       *Dst++ = DstPixel.col.blue;
//       SrcX = SourceRect->left + (Cols*(SourceRect->right - SourceRect->left))/(DestRect->right - DestRect->left);
//     }
//     Dst = (PUCHAR)((ULONG_PTR)Dest->pvScan0 + ((DestRect->top + Rows) * Dest->lDelta) +
//                 (DestRect->left*3));
//     SrcY = SourceRect->top + (Rows*(SourceRect->bottom - SourceRect->top))/(DestRect->bottom - DestRect->top);
//   }
//
//    return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
