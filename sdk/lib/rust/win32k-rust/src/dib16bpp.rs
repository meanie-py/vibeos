use win32k_sys::win32k::*;

use crate::dib_helpers::*;

// EXLATEOBJ_vInitialize/Cleanup are NTAPI in C (win32ss/gdi/eng/xlateobj.c).
// NTAPI = stdcall on i386, cdecl on amd64/arm64; `extern "system"` resolves
// per-arch. ppalSrc/ppalDst are typed `PALETTE *` in C.
extern "system" {
    fn EXLATEOBJ_vInitialize(
        exlo: *mut EXLATEOBJ,
        ppalSrc: *mut _PALETTE,
        ppalDst: *mut _PALETTE,
        crSrcBackColor: COLORREF,
        crDstBackColor: COLORREF,
        crDstForeColor: COLORREF,
    );
    fn EXLATEOBJ_vCleanup(exlo: *mut EXLATEOBJ);
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

#[repr(C)]
#[derive(Clone, Copy)]
pub union NICEPIXEL16_565 {
    pub us: USHORT,
    pub col: NICEPIXEL16_565_col,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NICEPIXEL16_565_col {
    _bitfield: USHORT,
}

impl NICEPIXEL16_565_col {
    pub unsafe fn blue(&self) -> USHORT {
        (self._bitfield & 0x1F) as USHORT
    }
    pub unsafe fn set_blue(&mut self, val: USHORT) {
        self._bitfield = (self._bitfield & !0x1F) | (val & 0x1F);
    }
    pub unsafe fn green(&self) -> USHORT {
        ((self._bitfield >> 5) & 0x3F) as USHORT
    }
    pub unsafe fn set_green(&mut self, val: USHORT) {
        self._bitfield = (self._bitfield & !0x7E0) | ((val & 0x3F) << 5);
    }
    pub unsafe fn red(&self) -> USHORT {
        ((self._bitfield >> 11) & 0x1F) as USHORT
    }
    pub unsafe fn set_red(&mut self, val: USHORT) {
        self._bitfield = (self._bitfield & !0xF800) | ((val & 0x1F) << 11);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union NICEPIXEL16_555 {
    pub us: USHORT,
    pub col: NICEPIXEL16_555_col,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NICEPIXEL16_555_col {
    _bitfield: USHORT,
}

impl NICEPIXEL16_555_col {
    pub unsafe fn blue(&self) -> USHORT {
        (self._bitfield & 0x1F) as USHORT
    }
    pub unsafe fn set_blue(&mut self, val: USHORT) {
        self._bitfield = (self._bitfield & !0x1F) | (val & 0x1F);
    }
    pub unsafe fn green(&self) -> USHORT {
        ((self._bitfield >> 5) & 0x1F) as USHORT
    }
    pub unsafe fn set_green(&mut self, val: USHORT) {
        self._bitfield = (self._bitfield & !0x3E0) | ((val & 0x1F) << 5);
    }
    pub unsafe fn red(&self) -> USHORT {
        ((self._bitfield >> 10) & 0x1F) as USHORT
    }
    pub unsafe fn set_red(&mut self, val: USHORT) {
        self._bitfield = (self._bitfield & !0x7C00) | ((val & 0x1F) << 10);
    }
    pub unsafe fn xxxx(&self) -> USHORT {
        ((self._bitfield >> 15) & 0x1) as USHORT
    }
    pub unsafe fn set_xxxx(&mut self, val: USHORT) {
        self._bitfield = (self._bitfield & !0x8000) | ((val & 0x1) << 15);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_PutPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32, c: u32) {
    let byteaddr = ((*SurfObj).pvScan0 as PBYTE).offset((y * (*SurfObj).lDelta) as isize);
    let addr = (byteaddr as PWORD).offset(x as isize);
    
    *addr = c as u16;
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32 {
    let byteaddr = ((*SurfObj).pvScan0 as PBYTE).offset((y * (*SurfObj).lDelta) as isize);
    let addr = (byteaddr as PWORD).offset(x as isize);
    (*addr) as u32
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32) {
    let mut addr = ((*SurfObj).pvScan0 as PBYTE)
        .offset(y as isize * (*SurfObj).lDelta as isize) as PWORD;
    addr = addr.offset(x1 as isize);
    let mut addr = addr as PDWORD;

    // C original has an i386 GCC-asm `rep stosd/stosw` fast fill; the rustconv
    // translation mistranslated the operand binding (scratch on input-only
    // operands, addr→edi) and used 0/1-only asm labels (rejected by Rust
    // asm!). It is a micro-opt of this portable two-pixels-at-a-time loop
    // (used by amd64/arm64) — keep portable on all arches for correctness.
    {
        let mut cx = x1;
        let cc: u32;

        if x1 >= x2 {
            return;
        }

        if 0 != (cx & 0x01) {
            *(addr as PWORD) = c as u16;
            cx += 1;
            addr = (addr as PWORD).offset(1) as PDWORD;
        }
        cc = ((c & 0xffff) << 16) | (c & 0xffff);
        while cx + 1 < x2 {
            *addr = cc;
            addr = addr.offset(1);
            cx += 2;
        }
        if cx < x2 {
            *(addr as PWORD) = c as u16;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_VLine(SurfObj: *mut SURFOBJ, x: i32, y1: i32, y2: i32, c: u32) {
    // The C original (dib16bpp.c) has a hand-unrolled i386 GCC-asm store loop
    // for `_M_IX86 && !_MSC_VER`; the rustconv translation of it was broken
    // (load instead of store, `inout` on an rvalue). The asm is a pure
    // micro-optimization of this portable column-store loop, which amd64/arm64
    // already use — keep the portable path on all arches for correctness.
    {
        let byteaddr = ((*SurfObj).pvScan0 as PBYTE).offset((y1 * (*SurfObj).lDelta) as isize);
        let mut addr = (byteaddr as PWORD).offset(x as isize);
        let lDelta = (*SurfObj).lDelta;

        let mut byteaddr = addr as PBYTE;
        let mut y = y1;
        while y < y2 {
            *addr = c as u16;

            byteaddr = byteaddr.offset(lDelta as isize);
            addr = byteaddr as PWORD;
            y += 1;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_BitBltSrcCopy(BltInfo: PBLTINFO) -> u8 {
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
    let mut Source32: PWORD;
    let mut Dest32: PWORD;
    let mut Index: u32;
    let mut StartLeft: u32;
    let mut EndRight: u32;
    let bTopToBottom: u8;
    let bLeftToRight: u8;

    bLeftToRight = if (*BltInfo).DestRect.left > (*BltInfo).DestRect.right { 1 } else { 0 };
    bTopToBottom = if (*BltInfo).DestRect.top > (*BltInfo).DestRect.bottom { 1 } else { 0 };

    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!((*BltInfo).DestRect));

    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
        .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
        .offset((2 * (*BltInfo).DestRect.left) as isize);

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
                    StartLeft = ((*BltInfo).DestRect.left + 1) as u32;
                    EndRight = ((*BltInfo).DestRect.right + 1) as u32;
                } else {
                    StartLeft = (*BltInfo).DestRect.left as u32;
                    EndRight = (*BltInfo).DestRect.right as u32;
                }

                i = StartLeft as i32;
                while i < EndRight as i32 {
                    if DIB_1BPP_GetPixel((*BltInfo).SourceSurface, sx, sy) == 0 {
                        DIB_16BPP_PutPixel((*BltInfo).DestSurface, i, j,
                            XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, 0));
                    } else {
                        DIB_16BPP_PutPixel((*BltInfo).DestSurface, i, j,
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
                SourceBits_4BPP = SourceBits_4BPP.offset(
                    (((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize
                );
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
                    xColor = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest,
                        ((*SourceLine_4BPP & altnotmask[f1 as usize]) >> (4 * (1 - f1))) as u32) as i32;
                    DIB_16BPP_PutPixel((*BltInfo).DestSurface, i, j, xColor as u32);
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
                SourceLine = SourceLine.offset(
                    ((*(*BltInfo).SourceSurface).lDelta * ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1)) as isize
                );
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                SourceBits = SourceLine;
                DestBits = DestLine;

                if bLeftToRight != 0 {
                    SourceBits = SourceBits.offset(((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) as isize);
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    *(DestBits as *mut u16) = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u16;
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-1) } else { SourceBits.offset(1) };
                    DestBits = DestBits.offset(2);
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
            if ((*BltInfo).XlateSourceToDest.is_null() ||
                ((*(*BltInfo).XlateSourceToDest).flXlate & XO_TRIVIAL) != 0) &&
                (bTopToBottom == 0 && bLeftToRight == 0) {
                if (*BltInfo).DestRect.top < (*BltInfo).SourcePoint.y {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                        .offset((2 * (*BltInfo).SourcePoint.x) as isize);

                    j = (*BltInfo).DestRect.top;
                    while j < (*BltInfo).DestRect.bottom {
                        core::ptr::copy_nonoverlapping(
                            SourceBits,
                            DestBits,
                            (2 * ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left)) as usize
                        );
                        SourceBits = SourceBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                        DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                        j += 1;
                    }
                } else {
                    SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                        .offset((
                            ((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) *
                            (*(*BltInfo).SourceSurface).lDelta
                        ) as isize)
                        .offset((2 * (*BltInfo).SourcePoint.x) as isize);

                    DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                        .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                        .offset((2 * (*BltInfo).DestRect.left) as isize);

                    j = (*BltInfo).DestRect.bottom - 1;
                    while (*BltInfo).DestRect.top <= j {
                        core::ptr::copy_nonoverlapping(
                            SourceBits,
                            DestBits,
                            (2 * ((*BltInfo).DestRect.right - (*BltInfo).DestRect.left)) as usize
                        );
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
                            .offset((2 * (*BltInfo).SourcePoint.x) as isize);

                        DestLine = DestBits;
                        j = (*BltInfo).DestRect.top;
                        while j < (*BltInfo).DestRect.bottom {
                            SourceBits = SourceLine;
                            DestBits = DestLine;
                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                *(DestBits as *mut u16) = XLATEOBJ_iXlate(
                                    (*BltInfo).XlateSourceToDest,
                                    *(SourceBits as *mut u16) as u32
                                ) as u16;
                                SourceBits = SourceBits.offset(2);
                                DestBits = DestBits.offset(2);
                                i += 1;
                            }
                            SourceLine = SourceLine.offset((*(*BltInfo).SourceSurface).lDelta as isize);
                            DestLine = DestLine.offset((*(*BltInfo).DestSurface).lDelta as isize);
                            j += 1;
                        }
                    } else {
                        SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset((
                                ((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) *
                                (*(*BltInfo).SourceSurface).lDelta
                            ) as isize)
                            .offset((2 * (*BltInfo).SourcePoint.x) as isize);

                        DestLine = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((2 * (*BltInfo).DestRect.left) as isize);

                        j = (*BltInfo).DestRect.bottom - 1;
                        while (*BltInfo).DestRect.top <= j {
                            SourceBits = SourceLine;
                            DestBits = DestLine;
                            i = (*BltInfo).DestRect.left;
                            while i < (*BltInfo).DestRect.right {
                                *(DestBits as *mut u16) = XLATEOBJ_iXlate(
                                    (*BltInfo).XlateSourceToDest,
                                    *(SourceBits as *mut u16) as u32
                                ) as u16;
                                SourceBits = SourceBits.offset(2);
                                DestBits = DestBits.offset(2);
                                i += 1;
                            }
                            SourceLine = SourceLine.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                            DestLine = DestLine.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                            j -= 1;
                        }
                    }
                } else {
                    let mut TopToBottomDone: u8 = 0;

                    if bLeftToRight != 0 {
                        let store = ExAllocatePoolWithTag(
                            NonPagedPool,
                            (((*BltInfo).DestRect.right - (*BltInfo).DestRect.left + 1) * 2) as SIZE_T,
                            TAG_DIB
                        ) as *mut u16;
                        if store.is_null() {
                            return 0u8;
                        }

                        SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset((
                                ((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) *
                                (*(*BltInfo).SourceSurface).lDelta
                            ) as isize)
                            .offset((2 * (*BltInfo).SourcePoint.x) as isize);

                        DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset((((*BltInfo).DestRect.bottom - 1) * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((2 * (*BltInfo).DestRect.left) as isize)
                            .offset(2);

                        j = (*BltInfo).DestRect.bottom - 1;
                        while (*BltInfo).DestRect.top <= j {
                            Dest32 = (DestBits as PWORD).offset(((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) as isize);
                            Source32 = SourceBits as PWORD;

                            Index = 0;

                            i = (*BltInfo).DestRect.right - 1;
                            while (*BltInfo).DestRect.left <= i {
                                *store.offset(Index as isize) = XLATEOBJ_iXlate(
                                    (*BltInfo).XlateSourceToDest,
                                    *Source32 as u32
                                ) as u16;
                                Source32 = Source32.offset(1);
                                Index += 1;
                                i -= 1;
                            }

                            Index = 0;

                            i = (*BltInfo).DestRect.right - 1;
                            while (*BltInfo).DestRect.left <= i {
                                *Dest32 = *store.offset(Index as isize);
                                Dest32 = Dest32.offset(-1);
                                Index += 1;
                                i -= 1;
                            }

                            SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
                            DestBits = DestBits.offset(-((*(*BltInfo).DestSurface).lDelta as isize));
                            j -= 1;
                        }
                        ExFreePoolWithTag(store as *mut core::ffi::c_void, TAG_DIB);
                        TopToBottomDone = 1;
                    }

                    if bTopToBottom != 0 {
                        let store = ExAllocatePoolWithTag(
                            NonPagedPool,
                            (((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top + 1) * 2) as SIZE_T,
                            TAG_DIB
                        ) as *mut u16;
                        if store.is_null() {
                            return 0u8;
                        }

                        SourceBits = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                            .offset((2 * (*BltInfo).SourcePoint.x) as isize);

                        DestBits = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                            .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
                            .offset((2 * (*BltInfo).DestRect.left) as isize);

                        if ((*(*BltInfo).SourceSurface).fjBitmap as u32 & BMF_TOPDOWN) == 0 {
                            DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                        }

                        if bLeftToRight != 0 {
                            DestBits = DestBits.offset(2);
                        }

                        if TopToBottomDone != 0 {
                            SourceBits = DestBits;
                        }

                        j = (*BltInfo).DestRect.right - 1;
                        while (*BltInfo).DestRect.left <= j {
                            Dest32 = (DestBits as PWORD).offset(
                                (((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).DestSurface).lDelta / 2) as isize
                            );
                            Source32 = SourceBits as PWORD;

                            Index = 0;

                            i = (*BltInfo).DestRect.top;
                            while i <= (*BltInfo).DestRect.bottom - 1 {
                                *store.offset(Index as isize) = XLATEOBJ_iXlate(
                                    (*BltInfo).XlateSourceToDest,
                                    *Source32 as u32
                                ) as u16;
                                Source32 = Source32.offset(((*(*BltInfo).SourceSurface).lDelta / 2) as isize);
                                Index += 1;
                                i += 1;
                            }

                            Index = 0;

                            i = (*BltInfo).DestRect.bottom - 1;
                            while (*BltInfo).DestRect.top <= i {
                                *Dest32 = *store.offset(Index as isize);
                                Dest32 = Dest32.offset(-(((*(*BltInfo).DestSurface).lDelta / 2) as isize));
                                Index += 1;
                                i -= 1;
                            }
                            SourceBits = SourceBits.offset(2);
                            DestBits = DestBits.offset(2);
                            j -= 1;
                        }
                        ExFreePoolWithTag(store as *mut core::ffi::c_void, TAG_DIB);
                    }
                }
            }
        }

        BMF_24BPP => {
            SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                .offset(((*BltInfo).SourcePoint.y * (*(*BltInfo).SourceSurface).lDelta) as isize)
                .offset((3 * (*BltInfo).SourcePoint.x) as isize);

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
                    SourceBits = SourceBits.offset((((*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1) * 3) as isize);
                }
                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    xColor = ((*SourceBits.offset(2) as i32) << 0x10) +
                        ((*SourceBits.offset(1) as i32) << 0x08) + (*SourceBits as i32);

                    *(DestBits as *mut u16) = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, xColor as u32) as u16;

                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-3) } else { SourceBits.offset(3) };
                    DestBits = DestBits.offset(2);
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
                    *(DestBits as *mut u16) = XLATEOBJ_iXlate(
                        (*BltInfo).XlateSourceToDest,
                        *(SourceBits as PDWORD)
                    ) as u16;
                    SourceBits = if bLeftToRight != 0 { SourceBits.offset(-4) } else { SourceBits.offset(4) };
                    DestBits = DestBits.offset(2);
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
            return 0u8;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_ColorFill(DestSurface: *mut SURFOBJ, DestRect: *mut RECTL, color: u32) -> u8 {

    RECTL_vMakeWellOrdered(DestRect);

    // C original has an i386 GCC-asm `rep stosl/stosw` fast fill for
    // `_M_IX86 && !_MSC_VER`; rustconv mistranslated the operand binding
    // (named/positional mix). It is a micro-opt of this portable HLine loop
    // (used by amd64/arm64) — keep portable on all arches for correctness.
    {
        let mut DestY = (*DestRect).top;
        while DestY < (*DestRect).bottom {
            DIB_16BPP_HLine(DestSurface, (*DestRect).left, (*DestRect).right, DestY, color);
            DestY += 1;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_TransparentBlt(
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
    let mut wd: i32;
    let mut DestBits: *mut u32;
    let mut Source: u32;
    let mut Dest: u32;

    let DstHeight: i32;
    let DstWidth: i32;
    let SrcHeight: i32;
    let SrcWidth: i32;

    DstHeight = (*DestRect).bottom - (*DestRect).top;
    DstWidth = (*DestRect).right - (*DestRect).left;
    SrcHeight = (*SourceRect).bottom - (*SourceRect).top;
    SrcWidth = (*SourceRect).right - (*SourceRect).left;

    RoundedRight = (*DestRect).right - (((*DestRect).right - (*DestRect).left) & 0x1);
    // C: `(PBYTE)DestSurf->pvScan0 + (left << 1) + top * lDelta` -- byte arithmetic.
    // Through `*mut PBYTE` both terms were scaled by the pointer size, so a transparent
    // blit into a 16bpp surface at any non-origin rect wrote 4x/8x past its target.
    DestBits = ((*DestSurf).pvScan0 as PBYTE)
        .offset(((*DestRect).left << 1) as isize)
        .offset(((*DestRect).top * (*DestSurf).lDelta) as isize) as *mut u32;
    wd = (*DestSurf).lDelta - (((*DestRect).right - (*DestRect).left) << 1);

    Y = (*DestRect).top;
    while Y < (*DestRect).bottom {
        SourceY = (*SourceRect).top + (Y - (*DestRect).top) * SrcHeight / DstHeight;
        X = (*DestRect).left;
        while X < RoundedRight {
            Dest = *DestBits;

            SourceX = (*SourceRect).left + (X - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0
                && SourceY >= 0
                && (*SourceSurf).sizlBitmap.cx > SourceX
                && (*SourceSurf).sizlBitmap.cy > SourceY
            {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if Source != iTransColor {
                    Dest &= 0xFFFF0000;
                    Dest |= XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFFFF;
                }
            }

            SourceX = (*SourceRect).left + (X + 1 - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0
                && SourceY >= 0
                && (*SourceSurf).sizlBitmap.cx > SourceX
                && (*SourceSurf).sizlBitmap.cy > SourceY
            {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if Source != iTransColor {
                    Dest &= 0xFFFF;
                    Dest |= XLATEOBJ_iXlate(ColorTranslation, Source) << 16;
                }
            }

            *DestBits = Dest;

            X += 2;
            DestBits = DestBits.offset(1);
            SourceX += 2;
        }

        if X < (*DestRect).right {
            SourceX = (*SourceRect).left + (X - (*DestRect).left) * SrcWidth / DstWidth;
            if SourceX >= 0
                && SourceY >= 0
                && (*SourceSurf).sizlBitmap.cx > SourceX
                && (*SourceSurf).sizlBitmap.cy > SourceY
            {
                Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
                if Source != iTransColor {
                    *(DestBits as *mut u16) = XLATEOBJ_iXlate(ColorTranslation, Source) as u16;
                }
            }

            DestBits = (DestBits as usize + 2) as *mut u32;
        }

        DestBits = (DestBits as usize + wd as usize) as *mut u32;
        Y += 1;
    }

    1u8
}

unsafe fn Clamp6(val: ULONG) -> UCHAR {
    if val > 63 { 63 } else { val as UCHAR }
}

unsafe fn Clamp5(val: ULONG) -> UCHAR {
    if val > 31 { 31 } else { val as UCHAR }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_16BPP_AlphaBlend(
    Dest: *mut SURFOBJ,
    Source: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    BlendObj: *mut BLENDOBJ,
) -> u8 {
    let mut DstX: i32;
    let mut DstY: i32;
    let mut SrcX: i32;
    let mut SrcY: i32;
    let BlendFunc: BLENDFUNCTION;
    let mut SrcPixel32: NICEPIXEL32 = core::mem::zeroed();
    let mut Alpha: u8;
    let mut pexlo: *mut EXLATEOBJ;
    let mut exloSrcRGB: EXLATEOBJ = core::mem::zeroed();

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
        && (BitsPerFormat((*Source).iBitmapFormat) != 32)
    {
        return 0u8;
    }

    if ColorTranslation.is_null() {
        return 0u8;
    }

    pexlo = (ColorTranslation as *mut u8).sub(
        core::mem::offset_of!(EXLATEOBJ, xlo)
    ) as *mut EXLATEOBJ;
    EXLATEOBJ_vInitialize(&mut exloSrcRGB, (*pexlo).ppalSrc, core::ptr::addr_of_mut!(gpalRGB), 0, 0, 0);

    if ((*(*pexlo).ppalDst).flFlags & PAL_RGB16_555) != 0 {
        let mut DstPixel16: NICEPIXEL16_555 = core::mem::zeroed();

        SrcY = (*SourceRect).top;
        DstY = (*DestRect).top;
        while DstY < (*DestRect).bottom {
            SrcX = (*SourceRect).left;
            DstX = (*DestRect).left;
            while DstX < (*DestRect).right {
                SrcPixel32.ul = DIB_GetSource(Source, SrcX, SrcY, &mut exloSrcRGB.xlo);
                SrcPixel32.col.red = (SrcPixel32.col.red as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;
                SrcPixel32.col.green = (SrcPixel32.col.green as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;
                SrcPixel32.col.blue = (SrcPixel32.col.blue as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;

                Alpha = if (BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0 {
                    (SrcPixel32.col.alpha as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8
                } else {
                    BlendFunc.SourceConstantAlpha
                };

                Alpha >>= 3;

                DstPixel16.us = (DIB_16BPP_GetPixel(Dest, DstX, DstY) & 0xFFFF) as u16;
                SrcPixel32.col.red >>= 3;
                SrcPixel32.col.green >>= 3;
                SrcPixel32.col.blue >>= 3;

                DstPixel16.col.set_red(Clamp5((DstPixel16.col.red() as u32 * (31 - Alpha as u32)) / 31 + SrcPixel32.col.red as u32) as u16);
                DstPixel16.col.set_green(Clamp5((DstPixel16.col.green() as u32 * (31 - Alpha as u32)) / 31 + SrcPixel32.col.green as u32) as u16);
                DstPixel16.col.set_blue(Clamp5((DstPixel16.col.blue() as u32 * (31 - Alpha as u32)) / 31 + SrcPixel32.col.blue as u32) as u16);

                DIB_16BPP_PutPixel(Dest, DstX, DstY, DstPixel16.us as u32);

                DstX += 1;
                SrcX = (*SourceRect).left + ((DstX - (*DestRect).left) * ((*SourceRect).right - (*SourceRect).left))
                    / ((*DestRect).right - (*DestRect).left);
            }
            DstY += 1;
            SrcY = (*SourceRect).top + ((DstY - (*DestRect).top) * ((*SourceRect).bottom - (*SourceRect).top))
                / ((*DestRect).bottom - (*DestRect).top);
        }
    } else {
        let mut DstPixel16: NICEPIXEL16_565 = core::mem::zeroed();
        let mut Alpha6: u8;
        let mut Alpha5: u8;

        SrcY = (*SourceRect).top;
        DstY = (*DestRect).top;
        while DstY < (*DestRect).bottom {
            SrcX = (*SourceRect).left;
            DstX = (*DestRect).left;
            while DstX < (*DestRect).right {
                SrcPixel32.ul = DIB_GetSource(Source, SrcX, SrcY, &mut exloSrcRGB.xlo);
                SrcPixel32.col.red = (SrcPixel32.col.red as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;
                SrcPixel32.col.green = (SrcPixel32.col.green as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;
                SrcPixel32.col.blue = (SrcPixel32.col.blue as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;

                Alpha = if (BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0 {
                    (SrcPixel32.col.alpha as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8
                } else {
                    BlendFunc.SourceConstantAlpha
                };

                Alpha6 = Alpha >> 2;
                Alpha5 = Alpha >> 3;

                DstPixel16.us = (DIB_16BPP_GetPixel(Dest, DstX, DstY) & 0xFFFF) as u16;
                SrcPixel32.col.red >>= 3;
                SrcPixel32.col.green >>= 2;
                SrcPixel32.col.blue >>= 3;

                DstPixel16.col.set_red(Clamp5((DstPixel16.col.red() as u32 * (31 - Alpha5 as u32)) / 31 + SrcPixel32.col.red as u32) as u16);
                DstPixel16.col.set_green(Clamp6((DstPixel16.col.green() as u32 * (63 - Alpha6 as u32)) / 63 + SrcPixel32.col.green as u32) as u16);
                DstPixel16.col.set_blue(Clamp5((DstPixel16.col.blue() as u32 * (31 - Alpha5 as u32)) / 31 + SrcPixel32.col.blue as u32) as u16);

                DIB_16BPP_PutPixel(Dest, DstX, DstY, DstPixel16.us as u32);

                DstX += 1;
                SrcX = (*SourceRect).left + ((DstX - (*DestRect).left) * ((*SourceRect).right - (*SourceRect).left))
                    / ((*DestRect).right - (*DestRect).left);
            }
            DstY += 1;
            SrcY = (*SourceRect).top + ((DstY - (*DestRect).top) * ((*SourceRect).bottom - (*SourceRect).top))
                / ((*DestRect).bottom - (*DestRect).top);
        }
    }

    EXLATEOBJ_vCleanup(&mut exloSrcRGB);

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Win32 subsystem
//  * LICENSE:         See COPYING in the top level directory
//  * FILE:            win32ss/gdi/dib/dib16bpp.c
//  * PURPOSE:         Device Independant Bitmap functions, 16bpp
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
// DIB_16BPP_PutPixel(SURFOBJ *SurfObj, LONG x, LONG y, ULONG c)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta;
//   PWORD addr = (PWORD)byteaddr + x;
//
//   *addr = (WORD)c;
// }
//
// ULONG
// DIB_16BPP_GetPixel(SURFOBJ *SurfObj, LONG x, LONG y)
// {
//   PBYTE byteaddr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta;
//   PWORD addr = (PWORD)byteaddr + x;
//   return (ULONG)(*addr);
// }
//
// VOID
// DIB_16BPP_HLine(SURFOBJ *SurfObj, LONG x1, LONG x2, LONG y, ULONG c)
// {
//   PDWORD addr = (PDWORD)((PWORD)((PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta) + x1);
//
// #if defined(_M_IX86) && !defined(_MSC_VER)
//   /* This is about 10% faster than the generic C code below */
//   LONG Count = x2 - x1;
//
//   if (x1 >= x2)
//     return;
//
//   __asm__ __volatile__ (
//     "  cld\n"
//     "  mov  %0, %%eax\n"
//     "  shl  $16, %%eax\n"
//     "  andl $0xffff, %0\n"  /* If the pixel value is "abcd", put "abcdabcd" in %eax */
//     "  or   %0, %%eax\n"
//     "  mov  %2, %%edi\n"
//     "  test $0x03, %%edi\n" /* Align to fullword boundary */
//     "  jz   0f\n"
//     "  stosw\n"
//     "  dec  %1\n"
//     "  jz   1f\n"
//     "0:\n"
//     "  mov  %1,%%ecx\n"     /* Setup count of fullwords to fill */
//     "  shr  $1,%%ecx\n"
//     "  rep stosl\n"         /* The actual fill */
//     "  test $0x01, %1\n"    /* One left to do at the right side? */
//     "  jz   1f\n"
//     "  stosw\n"
//     "1:\n"
//     : /* no output */
//   : "r"(c), "r"(Count), "m"(addr)
//     : "%eax", "%ecx", "%edi");
// #else /* _M_IX86 */
//   LONG cx = x1;
//   DWORD cc;
//
//   if (x1 >= x2)
//     return;
//
//   if (0 != (cx & 0x01))
//   {
//     *((PWORD) addr) = (WORD)c;
//     cx++;
//     addr = (PDWORD)((PWORD)(addr) + 1);
//   }
//   cc = ((c & 0xffff) << 16) | (c & 0xffff);
//   while(cx + 1 < x2)
//   {
//     *addr++ = cc;
//     cx += 2;
//   }
//   if (cx < x2)
//   {
//     *((PWORD) addr) = (WORD)c;
//   }
// #endif /* _M_IX86 */
// }
//
//
// VOID
// DIB_16BPP_VLine(SURFOBJ *SurfObj, LONG x, LONG y1, LONG y2, ULONG c)
// {
// #if defined(_M_IX86) && !defined(_MSC_VER)
//   asm volatile(
//     "   testl %2, %2"       "\n\t"
//     "   jle   2f"           "\n\t"
//     "   movl  %2, %%ecx"    "\n\t"
//     "   shrl  $2, %2"       "\n\t"
//     "   andl  $3, %%ecx"    "\n\t"
//     "   jz    1f"           "\n\t"
//     "0:"                    "\n\t"
//     "   movw  %%ax, (%0)"   "\n\t"
//     "   addl  %1, %0"       "\n\t"
//     "   decl  %%ecx"        "\n\t"
//     "   jnz   0b"           "\n\t"
//     "   testl %2, %2"       "\n\t"
//     "   jz    2f"           "\n\t"
//     "1:"                    "\n\t"
//     "   movw  %%ax, (%0)"   "\n\t"
//     "   addl  %1, %0"       "\n\t"
//     "   movw  %%ax, (%0)"   "\n\t"
//     "   addl  %1, %0"       "\n\t"
//     "   movw  %%ax, (%0)"   "\n\t"
//     "   addl  %1, %0"       "\n\t"
//     "   movw  %%ax, (%0)"   "\n\t"
//     "   addl  %1, %0"       "\n\t"
//     "   decl  %2"           "\n\t"
//     "   jnz   1b"           "\n\t"
//     "2:"                    "\n\t"
//     : /* no output */
//   : "r"((PBYTE)SurfObj->pvScan0 + (y1 * SurfObj->lDelta) + (x * sizeof (WORD))),
//     "r"(SurfObj->lDelta), "r"(y2 - y1), "a"(c)
//     : "cc", "memory", "%ecx");
// #else
//   PBYTE byteaddr = (PBYTE)(ULONG_PTR)SurfObj->pvScan0 + y1 * SurfObj->lDelta;
//   PWORD addr = (PWORD)byteaddr + x;
//   LONG lDelta = SurfObj->lDelta;
//
//   byteaddr = (PBYTE)addr;
//   while(y1++ < y2)
//   {
//     *addr = (WORD)c;
//
//     byteaddr += lDelta;
//     addr = (PWORD)byteaddr;
//   }
// #endif
// }
//
// BOOLEAN
// DIB_16BPP_BitBltSrcCopy(PBLTINFO BltInfo)
// {
//   LONG     i, j, sx, sy, xColor, f1;
//   PBYTE    SourceBits, DestBits, SourceLine, DestLine;
//   PBYTE    SourceBits_4BPP, SourceLine_4BPP;
//   PWORD    Source32, Dest32;
//   DWORD    Index, StartLeft, EndRight;
//   BOOLEAN  bTopToBottom, bLeftToRight;
//
//   DPRINT("DIB_16BPP_BitBltSrcCopy: SrcSurf cx/cy (%d/%d), DestSuft cx/cy (%d/%d) dstRect: (%d,%d)-(%d,%d)\n",
//          BltInfo->SourceSurface->sizlBitmap.cx, BltInfo->SourceSurface->sizlBitmap.cy,
//          BltInfo->DestSurface->sizlBitmap.cx, BltInfo->DestSurface->sizlBitmap.cy,
//          BltInfo->DestRect.left, BltInfo->DestRect.top, BltInfo->DestRect.right, BltInfo->DestRect.bottom);
//
//   /* Get back left to right flip here */
//   bLeftToRight = BltInfo->DestRect.left > BltInfo->DestRect.right;
//
//   /* Check for top to bottom flip needed. */
//   bTopToBottom = BltInfo->DestRect.top > BltInfo->DestRect.bottom;
//
//   DPRINT("BltInfo->SourcePoint.x is '%d' & BltInfo->SourcePoint.y is '%d'.\n",
//          BltInfo->SourcePoint.x, BltInfo->SourcePoint.y);
//
//   /* Make WellOrdered with top < bottom and left < right */
//   RECTL_vMakeWellOrdered(&BltInfo->DestRect);
//
//   DPRINT("BPP is '%d/%d' & BltInfo->SourcePoint.x is '%d' & BltInfo->SourcePoint.y is '%d'.\n",
//          BltInfo->SourceSurface->iBitmapFormat, BltInfo->SourcePoint.x, BltInfo->SourcePoint.y);
//
//   DestBits = (PBYTE)BltInfo->DestSurface->pvScan0 + (BltInfo->DestRect.top * BltInfo->DestSurface->lDelta) + 2 * BltInfo->DestRect.left;
//
//   switch(BltInfo->SourceSurface->iBitmapFormat)
//   {
//   case BMF_1BPP:
//     DPRINT("1BPP Case Selected with DestRect Width of '%d'.\n",
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//     sx = BltInfo->SourcePoint.x;
//
//     /* This sets sy to the top line */
//     sy = BltInfo->SourcePoint.y;
//
//     if (bTopToBottom)
//     {
//       /* This sets sy to the bottom line */
//       sy += BltInfo->SourceSurface->lDelta * (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//         /* This sets the sx to the rightmost pixel */
//         sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//         StartLeft = BltInfo->DestRect.left + 1;
//         EndRight = BltInfo->DestRect.right + 1;
//       }
//       else
//       {
//         StartLeft = BltInfo->DestRect.left;
//         EndRight = BltInfo->DestRect.right;
//       }
//
//       for (i = StartLeft; i < EndRight; i++)
//       {
//         if(DIB_1BPP_GetPixel(BltInfo->SourceSurface, sx, sy) == 0)
//         {
//           DIB_16BPP_PutPixel(BltInfo->DestSurface, i, j,
//             XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 0));
//         }
//         else
//         {
//           DIB_16BPP_PutPixel(BltInfo->DestSurface, i, j,
//             XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, 1));
//         }
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(sy, bTopToBottom, 1);
//     }
//     break;
//
//   case BMF_4BPP:
//     DPRINT("4BPP Case Selected with DestRect Width of '%d'.\n",
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     /* This sets SourceBits_4BPP to the top line */
//     SourceBits_4BPP = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//       (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) +
//       (BltInfo->SourcePoint.x >> 1);
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceBits_4BPP to the bottom line */
//       SourceBits_4BPP += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       SourceLine_4BPP = SourceBits_4BPP;
//       sx = BltInfo->SourcePoint.x;
//       if (bLeftToRight)
//       {
//         /* This sets sx to the rightmost pixel */
//         sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//       }
//
//       f1 = sx & 1;
//
//       for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//       {
//         xColor = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest,
//           (*SourceLine_4BPP & altnotmask[f1]) >> (4 * (1 - f1)));
//         DIB_16BPP_PutPixel(BltInfo->DestSurface, i, j, xColor);
//         if(f1 == 1)
//         {
//           DEC_OR_INC(SourceLine_4BPP, bLeftToRight, 1);
//           f1 = 0;
//         }
//         else
//         {
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
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     /* This sets SourceLine to the top line */
//     SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//       (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) +
//       BltInfo->SourcePoint.x;
//     DestLine = DestBits;
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceLine to the bottom line */
//       SourceLine += BltInfo->SourceSurface->lDelta
//         * (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//     }
//
//     for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//     {
//       SourceBits = SourceLine;
//       DestBits = DestLine;
//
//       if (bLeftToRight)
//       {
//         /* This sets SourceBits to the rightmost pixel */
//         SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//       }
//
//       for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//       {
//         *((WORD *)DestBits) = (WORD)XLATEOBJ_iXlate(
//           BltInfo->XlateSourceToDest, *SourceBits);
//         DEC_OR_INC(SourceBits, bLeftToRight, 1);
//         DestBits += 2;
//       }
//       DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//       DestLine += BltInfo->DestSurface->lDelta;
//     }
//     break;
//
//   case BMF_16BPP:
//     DPRINT("16BPP Case Selected with DestRect Width of '%d'.\n",
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     DPRINT("BMF_16BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//            BltInfo->DestRect.left, BltInfo->DestRect.top,
//            BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     if ((BltInfo->XlateSourceToDest == NULL ||
//       (BltInfo->XlateSourceToDest->flXlate & XO_TRIVIAL) != 0) &&
//       (!bTopToBottom && !bLeftToRight))
//     {
//       DPRINT("XO_TRIVIAL is TRUE.\n");
//       if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//       {
//         /* This sets SourceBits to the top line */
//         SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//           (BltInfo->SourcePoint.y *
//           BltInfo->SourceSurface->lDelta) + 2 *
//           BltInfo->SourcePoint.x;
//
//         for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//         {
//           RtlMoveMemory(DestBits, SourceBits,
//             2 * (BltInfo->DestRect.right -
//             BltInfo->DestRect.left));
//
//           SourceBits += BltInfo->SourceSurface->lDelta;
//           DestBits += BltInfo->DestSurface->lDelta;
//         }
//       }
//       else
//       {
//         /* This sets SourceBits to the bottom line */
//         SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//           ((BltInfo->SourcePoint.y + BltInfo->DestRect.bottom -
//           BltInfo->DestRect.top - 1) *
//           BltInfo->SourceSurface->lDelta) + 2 *
//           BltInfo->SourcePoint.x;
//
//         DestBits = (PBYTE)BltInfo->DestSurface->pvScan0 +
//           ((BltInfo->DestRect.bottom - 1) *
//           BltInfo->DestSurface->lDelta) + 2 *
//           BltInfo->DestRect.left;
//
//         for (j = BltInfo->DestRect.bottom - 1;
//           BltInfo->DestRect.top <= j; j--)
//         {
//           RtlMoveMemory(DestBits, SourceBits, 2 *
//             (BltInfo->DestRect.right -
//             BltInfo->DestRect.left));
//
//           SourceBits -= BltInfo->SourceSurface->lDelta;
//           DestBits -= BltInfo->DestSurface->lDelta;
//         }
//       }
//     }
//     else
//     {
//       DPRINT("XO_TRIVIAL is NOT TRUE.\n");
//       if (!bTopToBottom && !bLeftToRight)
//       /* **Note: Indent is purposefully less than desired to keep reviewable differences to a minimum for PR** */
//     {
//       DPRINT("Flip is None.\n");
//       if (BltInfo->DestRect.top < BltInfo->SourcePoint.y)
//       {
//         SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//           (BltInfo->SourcePoint.y *
//           BltInfo->SourceSurface->lDelta) + 2 *
//           BltInfo->SourcePoint.x;
//
//         DestLine = DestBits;
//         for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//         {
//           SourceBits = SourceLine;
//           DestBits = DestLine;
//           for (i = BltInfo->DestRect.left; i <
//             BltInfo->DestRect.right; i++)
//           {
//             *((WORD *)DestBits) = (WORD)XLATEOBJ_iXlate(
//               BltInfo->XlateSourceToDest,
//               *((WORD *)SourceBits));
//             SourceBits += 2;
//             DestBits += 2;
//           }
//           SourceLine += BltInfo->SourceSurface->lDelta;
//           DestLine += BltInfo->DestSurface->lDelta;
//         }
//       }
//       else
//       {
//         SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//           ((BltInfo->SourcePoint.y +
//           BltInfo->DestRect.bottom -
//           BltInfo->DestRect.top - 1) *
//           BltInfo->SourceSurface->lDelta) + 2 *
//           BltInfo->SourcePoint.x;
//
//         DestLine = (PBYTE)BltInfo->DestSurface->pvScan0 +
//           ((BltInfo->DestRect.bottom - 1) *
//           BltInfo->DestSurface->lDelta) + 2 *
//           BltInfo->DestRect.left;
//
//         for (j = BltInfo->DestRect.bottom - 1;
//           BltInfo->DestRect.top <= j; j--)
//         {
//           SourceBits = SourceLine;
//           DestBits = DestLine;
//           for (i = BltInfo->DestRect.left; i <
//             BltInfo->DestRect.right; i++)
//           {
//             *((WORD *)DestBits) = (WORD)XLATEOBJ_iXlate(
//               BltInfo->XlateSourceToDest,
//               *((WORD *)SourceBits));
//             SourceBits += 2;
//             DestBits += 2;
//           }
//           SourceLine -= BltInfo->SourceSurface->lDelta;
//           DestLine -= BltInfo->DestSurface->lDelta;
//         }
//       }
//     }
//       else
//       {
//         /* Buffering for source and destination flip overlaps. Fixes KHMZ MirrorTest CORE-16642 */
//         BOOL TopToBottomDone = FALSE;
//
//         if (bLeftToRight)
//         {
//           DPRINT("Flip is bLeftToRight.\n");
//
//           /* Allocate enough pixels for a row in WORD's */
//           WORD *store = ExAllocatePoolWithTag(NonPagedPool,
//             (BltInfo->DestRect.right - BltInfo->DestRect.left + 1) * 2, TAG_DIB);
//           if (store == NULL)
//           {
//             DPRINT1("Storage Allocation Failed.\n");
//             return FALSE;
//           }
//           WORD  Index;
//
//           /* This sets SourceBits to the bottom line */
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//             + ((BltInfo->SourcePoint.y + BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1)
//             * BltInfo->SourceSurface->lDelta) + 2 * BltInfo->SourcePoint.x;
//
//           /* Sets DestBits to the bottom line */
//           DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//            + (BltInfo->DestRect.bottom - 1) * BltInfo->DestSurface->lDelta
//            + 2 * BltInfo->DestRect.left + 2;
//
//           for (j = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= j; j--)
//           {
//             /* Set Dest32 to right pixel */
//             Dest32 = (WORD *) DestBits + (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//             Source32 = (WORD *) SourceBits;
//
//             Index = 0;
//
//             /* Store pixels from left to right */
//             for (i = BltInfo->DestRect.right - 1; BltInfo->DestRect.left <= i; i--)
//             {
//               store[Index] = (WORD)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *(WORD *)Source32++);
//               Index++;
//             }
//
//             Index = 0;
//
//             /* Copy stored data to pixels from right to left */
//             for (i = BltInfo->DestRect.right - 1; BltInfo->DestRect.left <= i; i--)
//             {
//               *(WORD *)Dest32-- = store[Index];
//               Index++;
//             }
//
//             SourceBits -= BltInfo->SourceSurface->lDelta;
//             DestBits -= BltInfo->DestSurface->lDelta;
//           }
//           ExFreePoolWithTag(store, TAG_DIB);
//           TopToBottomDone = TRUE;
//         }
//
//         if (bTopToBottom)
//         {
//           DPRINT("Flip is bTopToBottom.\n");
//
//           /* Allocate enough pixels for a column in WORD's */
//           WORD *store = ExAllocatePoolWithTag(NonPagedPool,
//             (BltInfo->DestRect.bottom - BltInfo->DestRect.top + 1) * 2, TAG_DIB);
//           if (store == NULL)
//           {
//             DPRINT1("Storage Allocation Failed.\n");
//             return FALSE;
//           }
//
//           /* This set SourceBits to the top line */
//           SourceBits = (PBYTE)BltInfo->SourceSurface->pvScan0
//             + (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta)
//             + 2 * BltInfo->SourcePoint.x;
//
//           /* This sets DestBits to the top line */
//           DestBits = (PBYTE)BltInfo->DestSurface->pvScan0
//            + ((BltInfo->DestRect.top) * BltInfo->DestSurface->lDelta)
//            + 2 * BltInfo->DestRect.left;
//
//           if ((BltInfo->SourceSurface->fjBitmap & BMF_TOPDOWN) == 0)
//           {
//             DestBits += BltInfo->DestSurface->lDelta;
//           }
//
//           if (bLeftToRight)
//           {
//             DPRINT("Adjusting DestBits for bLeftToRight.\n");
//             DestBits += 2;
//           }
//
//           /* The TopToBottomDone flag indicates that we are flipping for bTopToBottom and bLeftToRight
//            * and have already completed the bLeftToRight. So we will lose our first flip output
//            * unless we work with its output which is at the destination site. So in this case
//            * our new Source becomes the previous outputs Destination. */
//
//           if (TopToBottomDone)
//           {
//             /* This sets SourceBits to the top line */
//             SourceBits = DestBits;
//           }
//
//           for (j = BltInfo->DestRect.right - 1; BltInfo->DestRect.left <= j ; j--)
//           {
//             /* Set Dest32 to bottom pixel */
//             Dest32 = (WORD *) DestBits  + (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1)
//               * BltInfo->DestSurface->lDelta / 2;
//             Source32 = (WORD *) SourceBits;
//
//             Index = 0;
//
//             /* Store pixels from top to bottom */
//             for (i = BltInfo->DestRect.top; i <= BltInfo->DestRect.bottom - 1; i++)
//             {
//               store[Index] = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *Source32);
//               Source32 += BltInfo->SourceSurface->lDelta / 2;
//               Index++;
//             }
//
//             Index = 0;
//
//             /* Copy stored data to pixels from bottom to top */
//             for (i = BltInfo->DestRect.bottom - 1; BltInfo->DestRect.top <= i; i--)
//             {
//               *Dest32 = store[Index];
//               Dest32 -= BltInfo->DestSurface->lDelta / 2;
//               Index++;
//             }
//             SourceBits += 2;
//             DestBits += 2;
//           }
//           ExFreePoolWithTag(store, TAG_DIB);
//         }
//
//       }
//     }
//     break;
//
//   case BMF_24BPP:
//
//     DPRINT("BMF_24BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//            BltInfo->DestRect.left, BltInfo->DestRect.top,
//            BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     /* This sets SourceLine to the top line */
//     SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//       (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) +
//       3 * BltInfo->SourcePoint.x;
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceLine to the bottom line */
//       SourceLine += BltInfo->SourceSurface->lDelta * (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1);
//     }
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
//         SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1) * 3;
//       }
//       for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//       {
//         xColor = (*(SourceBits + 2) << 0x10) +
//           (*(SourceBits + 1) << 0x08) + (*(SourceBits));
//
//         *((WORD *)DestBits) = (WORD)XLATEOBJ_iXlate(
//           BltInfo->XlateSourceToDest, xColor);
//
//         DEC_OR_INC(SourceBits, bLeftToRight, 3);
//         DestBits += 2;
//       }
//       DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//       DestLine += BltInfo->DestSurface->lDelta;
//     }
//     break;
//
//   case BMF_32BPP:
//     DPRINT("32BPP Case Selected with DestRect Width of '%d'.\n",
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     SourceLine = (PBYTE)BltInfo->SourceSurface->pvScan0 +
//       (BltInfo->SourcePoint.y * BltInfo->SourceSurface->lDelta) +
//       4 * BltInfo->SourcePoint.x;
//
//     if (bTopToBottom)
//     {
//       /* This sets SourceLine to the bottom line */
//       SourceLine += BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1;
//     }
//     DestLine = DestBits;
//
//     for (j = BltInfo->DestRect.top; j < BltInfo->DestRect.bottom; j++)
//     {
//       SourceBits = SourceLine;
//       DestBits = DestLine;
//
//       if (bLeftToRight)
//       {
//         /* This sets SourceBits to the rightmost pixel */
//         SourceBits += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1) * 4;
//       }
//
//       for (i = BltInfo->DestRect.left; i < BltInfo->DestRect.right; i++)
//       {
//         *((WORD *)DestBits) = (WORD)XLATEOBJ_iXlate(
//           BltInfo->XlateSourceToDest,
//           *((PDWORD) SourceBits));
//         DEC_OR_INC(SourceBits, bLeftToRight, 4);
//         DestBits += 2;
//       }
//
//       DEC_OR_INC(SourceLine, bTopToBottom, BltInfo->SourceSurface->lDelta);
//       DestLine += BltInfo->DestSurface->lDelta;
//     }
//     break;
//
//   default:
//     DPRINT1("DIB_16BPP_BitBltSrcCopy: Unhandled Source BPP: %u\n",
//             BitsPerFormat(BltInfo->SourceSurface->iBitmapFormat));
//     return FALSE;
//   }
//
//   return TRUE;
// }
//
// /* Optimize for bitBlt */
// BOOLEAN
// DIB_16BPP_ColorFill(SURFOBJ* DestSurface, RECTL* DestRect, ULONG color)
// {
//   LONG DestY;
//
//   /* Make WellOrdered with top < bottom and left < right */
//   RECTL_vMakeWellOrdered(DestRect);
//
// #if defined(_M_IX86) && !defined(_MSC_VER)
//   /* This is about 10% faster than the generic C code below */
//   ULONG delta = DestSurface->lDelta;
//   ULONG width = (DestRect->right - DestRect->left) ;
//   PULONG pos =  (PULONG) ((PBYTE)DestSurface->pvScan0 + DestRect->top * delta + (DestRect->left<<1));
//   color = (color&0xffff);  /* If the color value is "abcd", put "abcdabcd" into color */
//   color += (color<<16);
//
//   for (DestY = DestRect->top; DestY< DestRect->bottom; DestY++)
//   {
//     __asm__ __volatile__ (
//       "cld\n\t"
//       "mov  %1,%%ebx\n\t"
//       "mov  %2,%%edi\n\t"
//       "test $0x03, %%edi\n\t"   /* Align to fullword boundary */
//       "jz   1f\n\t"
//       "stosw\n\t"
//       "dec  %%ebx\n\t"
//       "jz   2f\n"
//       "1:\n\t"
//       "mov  %%ebx,%%ecx\n\t"    /* Setup count of fullwords to fill */
//       "shr  $1,%%ecx\n\t"
//       "rep stosl\n\t"           /* The actual fill */
//       "test $0x01, %%ebx\n\t"   /* One left to do at the right side? */
//       "jz   2f\n\t"
//       "stosw\n"
//       "2:"
//       :
//       : "a" (color), "r" (width), "m" (pos)
//       : "%ecx", "%ebx", "%edi");
//     pos =(PULONG)((ULONG_PTR)pos + delta);
//   }
// #else /* _M_IX86 */
//
//   for (DestY = DestRect->top; DestY< DestRect->bottom; DestY++)
//   {
//     DIB_16BPP_HLine (DestSurface, DestRect->left, DestRect->right, DestY, color);
//   }
// #endif
//   return TRUE;
// }
//
// BOOLEAN
// DIB_16BPP_TransparentBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf,
//                          RECTL*  DestRect,  RECTL *SourceRect,
//                          XLATEOBJ *ColorTranslation, ULONG iTransColor)
// {
//   LONG RoundedRight, X, Y, SourceX = 0, SourceY = 0, wd;
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
//   RoundedRight = DestRect->right - ((DestRect->right - DestRect->left) & 0x1);
//   DestBits = (ULONG*)((PBYTE)DestSurf->pvScan0 +
//     (DestRect->left << 1) +
//     DestRect->top * DestSurf->lDelta);
//   wd = DestSurf->lDelta - ((DestRect->right - DestRect->left) << 1);
//
//   for(Y = DestRect->top; Y < DestRect->bottom; Y++)
//   {
//     SourceY = SourceRect->top+(Y - DestRect->top) * SrcHeight / DstHeight;
//     for(X = DestRect->left; X < RoundedRight; X += 2, DestBits++, SourceX += 2)
//     {
//       Dest = *DestBits;
//
//       SourceX = SourceRect->left+(X - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//         SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           Dest &= 0xFFFF0000;
//           Dest |= (XLATEOBJ_iXlate(ColorTranslation, Source) & 0xFFFF);
//         }
//       }
//
//       SourceX = SourceRect->left+(X+1 - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//         SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           Dest &= 0xFFFF;
//           Dest |= (XLATEOBJ_iXlate(ColorTranslation, Source) << 16);
//         }
//       }
//
//       *DestBits = Dest;
//     }
//
//     if(X < DestRect->right)
//     {
//       SourceX = SourceRect->left+(X - DestRect->left) * SrcWidth / DstWidth;
//       if (SourceX >= 0 && SourceY >= 0 &&
//         SourceSurf->sizlBitmap.cx > SourceX && SourceSurf->sizlBitmap.cy > SourceY)
//       {
//         Source = DIB_GetSourceIndex(SourceSurf, SourceX, SourceY);
//         if(Source != iTransColor)
//         {
//           *((USHORT*)DestBits) = (USHORT)XLATEOBJ_iXlate(ColorTranslation,
//             Source);
//         }
//       }
//
//       DestBits = (PULONG)((ULONG_PTR)DestBits + 2);
//     }
//
//     DestBits = (ULONG*)((ULONG_PTR)DestBits + wd);
//   }
//
//   return TRUE;
// }
//
// typedef union
// {
//   ULONG ul;
//   struct
//   {
//     UCHAR red;
//     UCHAR green;
//     UCHAR blue;
//     UCHAR alpha;
//   } col;
// } NICEPIXEL32;
//
// typedef union
// {
//   USHORT us;
//   struct
//   {
//     USHORT blue  :5;
//     USHORT green :6;
//     USHORT red   :5;
//   } col;
// } NICEPIXEL16_565;
//
// typedef union
// {
//   USHORT us;
//   struct
//   {
//     USHORT blue  :5;
//     USHORT green :5;
//     USHORT red   :5;
//     USHORT xxxx  :1;
//   } col;
// } NICEPIXEL16_555;
//
// static __inline UCHAR
// Clamp6(ULONG val)
// {
//    return (val > 63) ? 63 : (UCHAR)val;
// }
//
// static __inline UCHAR
// Clamp5(ULONG val)
// {
//    return (val > 31) ? 31 : (UCHAR)val;
// }
//
// BOOLEAN
// DIB_16BPP_AlphaBlend(SURFOBJ* Dest, SURFOBJ* Source, RECTL* DestRect,
//                      RECTL* SourceRect, CLIPOBJ* ClipRegion,
//                      XLATEOBJ* ColorTranslation, BLENDOBJ* BlendObj)
// {
//   INT DstX, DstY, SrcX, SrcY;
//   BLENDFUNCTION BlendFunc;
//   NICEPIXEL32 SrcPixel32;
//   UCHAR Alpha;
//   EXLATEOBJ* pexlo;
//   EXLATEOBJ exloSrcRGB;
//
//   DPRINT("DIB_16BPP_AlphaBlend: srcRect: (%d,%d)-(%d,%d), dstRect: (%d,%d)-(%d,%d)\n",
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
//           (BitsPerFormat(Source->iBitmapFormat) != 32))
//   {
//     DPRINT1("Source bitmap must be 32bpp when AC_SRC_ALPHA is set\n");
//     return FALSE;
//   }
//
//   if (!ColorTranslation)
//   {
//     DPRINT1("ColorTranslation must not be NULL!\n");
//     return FALSE;
//   }
//
//   pexlo = CONTAINING_RECORD(ColorTranslation, EXLATEOBJ, xlo);
//   EXLATEOBJ_vInitialize(&exloSrcRGB, pexlo->ppalSrc, &gpalRGB, 0, 0, 0);
//
//   if (pexlo->ppalDst->flFlags & PAL_RGB16_555)
//   {
//       NICEPIXEL16_555 DstPixel16;
//
//       SrcY = SourceRect->top;
//       DstY = DestRect->top;
//       while ( DstY < DestRect->bottom )
//       {
//         SrcX = SourceRect->left;
//         DstX = DestRect->left;
//         while(DstX < DestRect->right)
//         {
//           SrcPixel32.ul = DIB_GetSource(Source, SrcX, SrcY, &exloSrcRGB.xlo);
//           SrcPixel32.col.red = (SrcPixel32.col.red * BlendFunc.SourceConstantAlpha) / 255;
//           SrcPixel32.col.green = (SrcPixel32.col.green * BlendFunc.SourceConstantAlpha) / 255;
//           SrcPixel32.col.blue = (SrcPixel32.col.blue * BlendFunc.SourceConstantAlpha) / 255;
//
//           Alpha = ((BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0) ?
//                (SrcPixel32.col.alpha * BlendFunc.SourceConstantAlpha) / 255 :
//                BlendFunc.SourceConstantAlpha;
//
//           Alpha >>= 3;
//
//           DstPixel16.us = DIB_16BPP_GetPixel(Dest, DstX, DstY) & 0xFFFF;
//           /* Perform bit loss */
//           SrcPixel32.col.red >>= 3;
//           SrcPixel32.col.green >>= 3;
//           SrcPixel32.col.blue >>= 3;
//
//           /* Do the blend in the right bit depth */
//           DstPixel16.col.red = Clamp5((DstPixel16.col.red * (31 - Alpha)) / 31 + SrcPixel32.col.red);
//           DstPixel16.col.green = Clamp5((DstPixel16.col.green * (31 - Alpha)) / 31 + SrcPixel32.col.green);
//           DstPixel16.col.blue = Clamp5((DstPixel16.col.blue * (31 - Alpha)) / 31 + SrcPixel32.col.blue);
//
//           DIB_16BPP_PutPixel(Dest, DstX, DstY, DstPixel16.us);
//
//           DstX++;
//           SrcX = SourceRect->left + ((DstX-DestRect->left)*(SourceRect->right - SourceRect->left))
//                                                 /(DestRect->right-DestRect->left);
//         }
//         DstY++;
//         SrcY = SourceRect->top + ((DstY-DestRect->top)*(SourceRect->bottom - SourceRect->top))
//                                                 /(DestRect->bottom-DestRect->top);
//       }
//   }
//   else
//   {
//       NICEPIXEL16_565 DstPixel16;
//       UCHAR Alpha6, Alpha5;
//
//       SrcY = SourceRect->top;
//       DstY = DestRect->top;
//       while ( DstY < DestRect->bottom )
//       {
//         SrcX = SourceRect->left;
//         DstX = DestRect->left;
//         while(DstX < DestRect->right)
//         {
//           SrcPixel32.ul = DIB_GetSource(Source, SrcX, SrcY, &exloSrcRGB.xlo);
//           SrcPixel32.col.red = (SrcPixel32.col.red * BlendFunc.SourceConstantAlpha) / 255;
//           SrcPixel32.col.green = (SrcPixel32.col.green * BlendFunc.SourceConstantAlpha) / 255;
//           SrcPixel32.col.blue = (SrcPixel32.col.blue * BlendFunc.SourceConstantAlpha) / 255;
//
//           Alpha = ((BlendFunc.AlphaFormat as u32 & AC_SRC_ALPHA) != 0) ?
//                (SrcPixel32.col.alpha * BlendFunc.SourceConstantAlpha) / 255 :
//                BlendFunc.SourceConstantAlpha;
//
//           Alpha6 = Alpha >> 2;
//           Alpha5 = Alpha >> 3;
//
//           DstPixel16.us = DIB_16BPP_GetPixel(Dest, DstX, DstY) & 0xFFFF;
//           /* Perform bit loss */
//           SrcPixel32.col.red >>= 3;
//           SrcPixel32.col.green >>= 2;
//           SrcPixel32.col.blue >>= 3;
//
//           /* Do the blend in the right bit depth */
//           DstPixel16.col.red = Clamp5((DstPixel16.col.red * (31 - Alpha5)) / 31 + SrcPixel32.col.red);
//           DstPixel16.col.green = Clamp6((DstPixel16.col.green * (63 - Alpha6)) / 63 + SrcPixel32.col.green);
//           DstPixel16.col.blue = Clamp5((DstPixel16.col.blue * (31 - Alpha5)) / 31 + SrcPixel32.col.blue);
//
//           DIB_16BPP_PutPixel(Dest, DstX, DstY, DstPixel16.us);
//
//           DstX++;
//           SrcX = SourceRect->left + ((DstX-DestRect->left)*(SourceRect->right - SourceRect->left))
//                                                 /(DestRect->right-DestRect->left);
//         }
//         DstY++;
//         SrcY = SourceRect->top + ((DstY-DestRect->top)*(SourceRect->bottom - SourceRect->top))
//                                                 /(DestRect->bottom-DestRect->top);
//       }
//   }
//
//   EXLATEOBJ_vCleanup(&exloSrcRGB);
//
//   return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
