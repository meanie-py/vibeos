use win32k_sys::win32k::*;

// ---- file-scope preamble (translated) ----
pub const RLE_EOL: u32 = 0;
pub const RLE_END: u32 = 1;
pub const RLE_DELTA: u32 = 2;

#[no_mangle]
pub unsafe extern "C" fn DecompressBitmap(
    Size: SIZEL,
    CompressedBits: *mut u8,
    UncompressedBits: *mut u8,
    Delta: i32,
    Format: u32,
    cjSizeImage: u32,
) {
    let mut x: i32 = 0;
    let mut y: i32 = Size.cy - 1;
    let mut i: i32;
    let mut c: i32 = 0;
    let mut c2: i32;
    let mut length: i32;
    let width: i32 = Size.cx;
    let height: i32 = y;
    let begin: *mut u8 = CompressedBits;
    let mut bits: *mut u8 = CompressedBits;
    let mut temp: *mut u8;
    let mut is4bpp: u8 = 0u8;

    if (Format == BMF_4RLE) || (Format == BMF_4BPP) {
        is4bpp = 1u8;
    } else if (Format != BMF_8RLE) && (Format != BMF_8BPP) {
        return;
    }

    {
        while y >= 0 && (bits.offset_from(begin) as u32) <= cjSizeImage {
            length = *bits as i32;
            bits = bits.add(1);
            if length != 0 {
                c = *bits as i32;
                bits = bits.add(1);
                i = 0;
                while i < length {
                    if x >= width {
                        break;
                    }
                    temp = UncompressedBits.offset(((height - y) * Delta) as isize);
                    if is4bpp != 0 {
                        temp = temp.offset((x / 2) as isize);
                        if (i & 1) != 0 {
                            c2 = c & 0x0F;
                        } else {
                            c2 = c >> 4;
                        }
                        if (x & 1) != 0 {
                            *temp |= c2 as u8;
                        } else {
                            *temp |= (c2 << 4) as u8;
                        }
                    } else {
                        temp = temp.offset(x as isize);
                        *temp = c as u8;
                    }
                    x += 1;
                    i += 1;
                }
            } else {
                length = *bits as i32;
                bits = bits.add(1);
                match length {
                    0 => {
                        x = 0;
                        y -= 1;
                    }
                    1 => {
                        return;
                    }
                    2 => {
                        x += *bits as i32;
                        bits = bits.add(1);
                        y -= *bits as i32;
                        bits = bits.add(1);
                    }
                    _ => {
                        i = 0;
                        while i < length {
                            if !((is4bpp != 0) && (i & 1) != 0) {
                                c = *bits as i32;
                                bits = bits.add(1);
                            }

                            if x < width {
                                temp = UncompressedBits.offset(((height - y) * Delta) as isize);
                                if is4bpp != 0 {
                                    temp = temp.offset((x / 2) as isize);
                                    if (i & 1) != 0 {
                                        c2 = c & 0x0F;
                                    } else {
                                        c2 = c >> 4;
                                    }
                                    if (x & 1) != 0 {
                                        *temp |= c2 as u8;
                                    } else {
                                        *temp |= (c2 << 4) as u8;
                                    }
                                } else {
                                    temp = temp.offset(x as isize);
                                    *temp = c as u8;
                                }
                                x += 1;
                            }
                            i += 1;
                        }
                        if (bits.offset_from(begin) as isize & 1) != 0 {
                            bits = bits.add(1);
                        }
                    }
                }
            }
        }
    }

    return;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           RLE compression
//  * FILE:              win32ss/gdi/eng/rlecomp.c
//  * PROGRAMER:         Jason Filby
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// enum Rle_EscapeCodes
// {
//     RLE_EOL   = 0, /* End of line */
//     RLE_END   = 1, /* End of bitmap */
//     RLE_DELTA = 2  /* Delta */
// };
//
// VOID DecompressBitmap(SIZEL Size, BYTE *CompressedBits, BYTE *UncompressedBits,
//                       LONG Delta, ULONG Format, ULONG cjSizeImage)
// {
//     INT x = 0, y = Size.cy - 1;
//     INT i, c, c2, length;
//     INT width = Size.cx, height = y;
//     BYTE *begin = CompressedBits;
//     BYTE *bits = CompressedBits;
//     BYTE *temp;
//     BOOL is4bpp = FALSE;
//
//     if ((Format == BMF_4RLE) || (Format == BMF_4BPP))
//         is4bpp = TRUE;
//     else if ((Format != BMF_8RLE) && (Format != BMF_8BPP))
//         return;
//
//     _SEH2_TRY
//     {
//         while (y >= 0 && (bits - begin) <= cjSizeImage)
//         {
//             length = *bits++;
//             if (length)
//             {
//                 c = *bits++;
//                 for (i = 0; i < length; i++)
//                 {
//                     if (x >= width) break;
//                     temp = UncompressedBits + (height - y) * Delta;
//                     if (is4bpp)
//                     {
//                         temp += x / 2;
//                         if (i & 1)
//                             c2 = c & 0x0F;
//                         else
//                             c2 = c >> 4;
//                         if (x & 1)
//                             *temp |= c2;
//                         else
//                             *temp |= c2 << 4;
//                     }
//                     else
//                     {
//                         temp += x;
//                         *temp = c;
//                     }
//                     x++;
//                 }
//             }
//             else
//             {
//                 length = *bits++;
//                 switch (length)
//                 {
//                 case RLE_EOL:
//                     x = 0;
//                     y--;
//                     break;
//                 case RLE_END:
//                     _SEH2_YIELD(return);
//                 case RLE_DELTA:
//                     x += *bits++;
//                     y -= *bits++;
//                     break;
//                 default:
//                     for (i = 0; i < length; i++)
//                     {
//                         if (!(is4bpp && i & 1))
//                             c = *bits++;
//
//                         if (x < width)
//                         {
//                             temp = UncompressedBits + (height - y) * Delta;
//                             if (is4bpp)
//                             {
//                                 temp += x / 2;
//                                 if (i & 1)
//                                     c2 = c & 0x0F;
//                                 else
//                                     c2 = c >> 4;
//                                 if (x & 1)
//                                     *temp |= c2;
//                                 else
//                                     *temp |= c2 << 4;
//                             }
//                             else
//                             {
//                                 temp += x;
//                                 *temp = c;
//                             }
//                             x++;
//                         }
//                     }
//                     if ((bits - begin) & 1)
//                     {
//                         bits++;
//                     }
//                 }
//             }
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DPRINT1("Decoding error\n");
//     }
//     _SEH2_END;
//
//     return;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
