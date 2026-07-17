use win32k_sys::win32k::*;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "C" fn DIB_24BPP_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32) {
    let mut addr = ((*SurfObj).pvScan0 as PBYTE)
        .offset((y * (*SurfObj).lDelta) as isize)
        .offset(((x1 << 1) + x1) as isize);
    let mut Count = (x2 - x1) as u32;

    if x1 >= x2 {
        return;
    }

    if Count < 8 {
        while Count > 0 {
            Count -= 1;
            *(addr as *mut u16) = c as u16;
            addr = addr.offset(2);
            *addr = (c >> 16) as u8;
            addr = addr.offset(1);
        }
    } else {
        let mut Fill: [u32; 3] = [0; 3];
        let mut MultiCount: u32;

        while (addr as usize) & 0x3 != 0 {
            *(addr as *mut u16) = c as u16;
            addr = addr.offset(2);
            *addr = (c >> 16) as u8;
            addr = addr.offset(1);
            Count -= 1;
        }

        let c = c & 0xffffff;
        Fill[0] = c | (c << 24);
        Fill[1] = (c >> 8) | (c << 16);
        Fill[2] = (c << 8) | (c >> 16);
        MultiCount = Count / 4;
        loop {
            *(addr as *mut u32) = Fill[0];
            addr = addr.offset(4);
            *(addr as *mut u32) = Fill[1];
            addr = addr.offset(4);
            *(addr as *mut u32) = Fill[2];
            addr = addr.offset(4);
            MultiCount -= 1;
            if MultiCount == 0 {
                break;
            }
        }

        Count = Count & 0x03;
        while Count != 0 {
            Count -= 1;
            *(addr as *mut u16) = c as u16;
            addr = addr.offset(2);
            *addr = (c >> 16) as u8;
            addr = addr.offset(1);
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Win32 subsystem
//  * LICENSE:         See COPYING in the top level directory
//  * FILE:            win32ss/gdi/dib/dib24bppc.c
//  * PURPOSE:         C language equivalents of asm optimised 24bpp functions
//  * PROGRAMMERS:     Jason Filby
//  *                  Magnus Olsen
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// VOID
// DIB_24BPP_HLine(SURFOBJ *SurfObj, LONG x1, LONG x2, LONG y, ULONG c)
// {
//   PBYTE addr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta + (x1 << 1) + x1;
//   ULONG Count = x2 - x1;
//
//   if (x1 >= x2)
//     return;
//
//   if (Count < 8)
//   {
//     /* For small fills, don't bother doing anything fancy */
//     while (Count--)
//     {
//       *(PUSHORT)(addr) = c;
//       addr += 2;
//       *(addr) = c >> 16;
//       addr += 1;
//     }
//   }
//   else
//   {
//     ULONG Fill[3];
//     ULONG MultiCount;
//
//     /* Align to 4-byte address */
//     while (0 != ((ULONG_PTR) addr & 0x3))
//     {
//       *(PUSHORT)(addr) = c;
//       addr += 2;
//       *(addr) = c >> 16;
//       addr += 1;
//       Count--;
//     }
//     /* If the color we need to fill with is 0ABC, then the final mem pattern
//     * (note little-endianness) would be:
//     *
//     * |C.B.A|C.B.A|C.B.A|C.B.A|   <- pixel borders
//     * |C.B.A.C|B.A.C.B|A.C.B.A|   <- ULONG borders
//     *
//     * So, taking endianness into account again, we need to fill with these
//     * ULONGs: CABC BCAB ABCA */
//
//     c = c & 0xffffff;                /* 0ABC */
//     Fill[0] = c | (c << 24);         /* CABC */
//     Fill[1] = (c >> 8) | (c << 16);  /* BCAB */
//     Fill[2] = (c << 8) | (c >> 16);  /* ABCA */
//     MultiCount = Count / 4;
//     do
//     {
//       *(PULONG)addr = Fill[0];
//       addr += 4;
//       *(PULONG)addr = Fill[1];
//       addr += 4;
//       *(PULONG)addr = Fill[2];
//       addr += 4;
//     }
//     while (0 != --MultiCount);
//
//     Count = Count & 0x03;
//     while (0 != Count--)
//     {
//       *(PUSHORT)(addr) = c;
//       addr += 2;
//       *(addr) = c >> 16;
//       addr += 1;
//     }
//   }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
