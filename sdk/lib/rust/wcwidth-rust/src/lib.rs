//! Markus Kuhn's `wcwidth.c` (column widths of Unicode characters), ported once.
//!
//! The tree compiles the same file in two places: cmd.exe `#include`s
//! base/shell/cmd/wcwidth.c into console.c (its IS_FULL_WIDTH macro calls `mk_wcwidth_cjk`),
//! and consrv lists win32ss/user/winsrv/consrv/frontends/wcwidth.c. The two are
//! byte-identical; this is winsrv-rust's translation of it, moved here so both use one copy.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals)]

/// `wchar_t` is 16-bit on every Windows target.
pub type wchar_t = u16;

#[repr(C)]
struct interval {
    first: i32,
    last: i32,
}

pub(crate) unsafe fn bisearch(ucs: wchar_t, table: *const interval, mut max: i32) -> i32 {
    let mut min = 0i32;
    let mut mid: i32;

    if (ucs as i32) < (*table.add(0)).first || (ucs as i32) > (*table.add(max as usize)).last {
        return 0;
    }
    while max >= min {
        mid = (min + max) / 2;
        if (ucs as i32) > (*table.add(mid as usize)).last {
            min = mid + 1;
        } else if (ucs as i32) < (*table.add(mid as usize)).first {
            max = mid - 1;
        } else {
            return 1;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn mk_wcwidth(ucs: wchar_t) -> i32 {
    static combining: [interval; 142] = [
        interval { first: 0x0300, last: 0x036F }, interval { first: 0x0483, last: 0x0486 }, interval { first: 0x0488, last: 0x0489 },
        interval { first: 0x0591, last: 0x05BD }, interval { first: 0x05BF, last: 0x05BF }, interval { first: 0x05C1, last: 0x05C2 },
        interval { first: 0x05C4, last: 0x05C5 }, interval { first: 0x05C7, last: 0x05C7 }, interval { first: 0x0600, last: 0x0603 },
        interval { first: 0x0610, last: 0x0615 }, interval { first: 0x064B, last: 0x065E }, interval { first: 0x0670, last: 0x0670 },
        interval { first: 0x06D6, last: 0x06E4 }, interval { first: 0x06E7, last: 0x06E8 }, interval { first: 0x06EA, last: 0x06ED },
        interval { first: 0x070F, last: 0x070F }, interval { first: 0x0711, last: 0x0711 }, interval { first: 0x0730, last: 0x074A },
        interval { first: 0x07A6, last: 0x07B0 }, interval { first: 0x07EB, last: 0x07F3 }, interval { first: 0x0901, last: 0x0902 },
        interval { first: 0x093C, last: 0x093C }, interval { first: 0x0941, last: 0x0948 }, interval { first: 0x094D, last: 0x094D },
        interval { first: 0x0951, last: 0x0954 }, interval { first: 0x0962, last: 0x0963 }, interval { first: 0x0981, last: 0x0981 },
        interval { first: 0x09BC, last: 0x09BC }, interval { first: 0x09C1, last: 0x09C4 }, interval { first: 0x09CD, last: 0x09CD },
        interval { first: 0x09E2, last: 0x09E3 }, interval { first: 0x0A01, last: 0x0A02 }, interval { first: 0x0A3C, last: 0x0A3C },
        interval { first: 0x0A41, last: 0x0A42 }, interval { first: 0x0A47, last: 0x0A48 }, interval { first: 0x0A4B, last: 0x0A4D },
        interval { first: 0x0A70, last: 0x0A71 }, interval { first: 0x0A81, last: 0x0A82 }, interval { first: 0x0ABC, last: 0x0ABC },
        interval { first: 0x0AC1, last: 0x0AC5 }, interval { first: 0x0AC7, last: 0x0AC8 }, interval { first: 0x0ACD, last: 0x0ACD },
        interval { first: 0x0AE2, last: 0x0AE3 }, interval { first: 0x0B01, last: 0x0B01 }, interval { first: 0x0B3C, last: 0x0B3C },
        interval { first: 0x0B3F, last: 0x0B3F }, interval { first: 0x0B41, last: 0x0B43 }, interval { first: 0x0B4D, last: 0x0B4D },
        interval { first: 0x0B56, last: 0x0B56 }, interval { first: 0x0B82, last: 0x0B82 }, interval { first: 0x0BC0, last: 0x0BC0 },
        interval { first: 0x0BCD, last: 0x0BCD }, interval { first: 0x0C3E, last: 0x0C40 }, interval { first: 0x0C46, last: 0x0C48 },
        interval { first: 0x0C4A, last: 0x0C4D }, interval { first: 0x0C55, last: 0x0C56 }, interval { first: 0x0CBC, last: 0x0CBC },
        interval { first: 0x0CBF, last: 0x0CBF }, interval { first: 0x0CC6, last: 0x0CC6 }, interval { first: 0x0CCC, last: 0x0CCD },
        interval { first: 0x0CE2, last: 0x0CE3 }, interval { first: 0x0D41, last: 0x0D43 }, interval { first: 0x0D4D, last: 0x0D4D },
        interval { first: 0x0DCA, last: 0x0DCA }, interval { first: 0x0DD2, last: 0x0DD4 }, interval { first: 0x0DD6, last: 0x0DD6 },
        interval { first: 0x0E31, last: 0x0E31 }, interval { first: 0x0E34, last: 0x0E3A }, interval { first: 0x0E47, last: 0x0E4E },
        interval { first: 0x0EB1, last: 0x0EB1 }, interval { first: 0x0EB4, last: 0x0EB9 }, interval { first: 0x0EBB, last: 0x0EBC },
        interval { first: 0x0EC8, last: 0x0ECD }, interval { first: 0x0F18, last: 0x0F19 }, interval { first: 0x0F35, last: 0x0F35 },
        interval { first: 0x0F37, last: 0x0F37 }, interval { first: 0x0F39, last: 0x0F39 }, interval { first: 0x0F71, last: 0x0F7E },
        interval { first: 0x0F80, last: 0x0F84 }, interval { first: 0x0F86, last: 0x0F87 }, interval { first: 0x0F90, last: 0x0F97 },
        interval { first: 0x0F99, last: 0x0FBC }, interval { first: 0x0FC6, last: 0x0FC6 }, interval { first: 0x102D, last: 0x1030 },
        interval { first: 0x1032, last: 0x1032 }, interval { first: 0x1036, last: 0x1037 }, interval { first: 0x1039, last: 0x1039 },
        interval { first: 0x1058, last: 0x1059 }, interval { first: 0x1160, last: 0x11FF }, interval { first: 0x135F, last: 0x135F },
        interval { first: 0x1712, last: 0x1714 }, interval { first: 0x1732, last: 0x1734 }, interval { first: 0x1752, last: 0x1753 },
        interval { first: 0x1772, last: 0x1773 }, interval { first: 0x17B4, last: 0x17B5 }, interval { first: 0x17B7, last: 0x17BD },
        interval { first: 0x17C6, last: 0x17C6 }, interval { first: 0x17C9, last: 0x17D3 }, interval { first: 0x17DD, last: 0x17DD },
        interval { first: 0x180B, last: 0x180D }, interval { first: 0x18A9, last: 0x18A9 }, interval { first: 0x1920, last: 0x1922 },
        interval { first: 0x1927, last: 0x1928 }, interval { first: 0x1932, last: 0x1932 }, interval { first: 0x1939, last: 0x193B },
        interval { first: 0x1A17, last: 0x1A18 }, interval { first: 0x1B00, last: 0x1B03 }, interval { first: 0x1B34, last: 0x1B34 },
        interval { first: 0x1B36, last: 0x1B3A }, interval { first: 0x1B3C, last: 0x1B3C }, interval { first: 0x1B42, last: 0x1B42 },
        interval { first: 0x1B6B, last: 0x1B73 }, interval { first: 0x1DC0, last: 0x1DCA }, interval { first: 0x1DFE, last: 0x1DFF },
        interval { first: 0x200B, last: 0x200F }, interval { first: 0x202A, last: 0x202E }, interval { first: 0x2060, last: 0x2063 },
        interval { first: 0x206A, last: 0x206F }, interval { first: 0x20D0, last: 0x20EF }, interval { first: 0x302A, last: 0x302F },
        interval { first: 0x3099, last: 0x309A }, interval { first: 0xA806, last: 0xA806 }, interval { first: 0xA80B, last: 0xA80B },
        interval { first: 0xA825, last: 0xA826 }, interval { first: 0xFB1E, last: 0xFB1E }, interval { first: 0xFE00, last: 0xFE0F },
        interval { first: 0xFE20, last: 0xFE23 }, interval { first: 0xFEFF, last: 0xFEFF }, interval { first: 0xFFF9, last: 0xFFFB },
        interval { first: 0x10A01, last: 0x10A03 }, interval { first: 0x10A05, last: 0x10A06 }, interval { first: 0x10A0C, last: 0x10A0F },
        interval { first: 0x10A38, last: 0x10A3A }, interval { first: 0x10A3F, last: 0x10A3F }, interval { first: 0x1D167, last: 0x1D169 },
        interval { first: 0x1D173, last: 0x1D182 }, interval { first: 0x1D185, last: 0x1D18B }, interval { first: 0x1D1AA, last: 0x1D1AD },
        interval { first: 0x1D242, last: 0x1D244 }, interval { first: 0xE0001, last: 0xE0001 }, interval { first: 0xE0020, last: 0xE007F },
        interval { first: 0xE0100, last: 0xE01EF }
    ];

    if ucs == 0 {
        return 0;
    }
    if (ucs as u32) < 32 || ((ucs as u32) >= 0x7f && (ucs as u32) < 0xa0) {
        return -1;
    }

    if bisearch(ucs, combining.as_ptr(), (combining.len() - 1) as i32) != 0 {
        return 0;
    }

    1 + if (ucs as u32) >= 0x1100 &&
        ((ucs as u32) <= 0x115f ||
         ucs as u32 == 0x2329 || ucs as u32 == 0x232a ||
         ((ucs as u32) >= 0x2e80 && (ucs as u32) <= 0xa4cf &&
          ucs as u32 != 0x303f) ||
         ((ucs as u32) >= 0xac00 && (ucs as u32) <= 0xd7a3) ||
         ((ucs as u32) >= 0xf900 && (ucs as u32) <= 0xfaff) ||
         ((ucs as u32) >= 0xfe10 && (ucs as u32) <= 0xfe19) ||
         ((ucs as u32) >= 0xfe30 && (ucs as u32) <= 0xfe6f) ||
         ((ucs as u32) >= 0xff00 && (ucs as u32) <= 0xff60) ||
         ((ucs as u32) >= 0xffe0 && (ucs as u32) <= 0xffe6) ||
         (
             // Astral plane CJK ranges — sizeof(wchar_t) == 4 in C.
             // On Windows wchar_t is u16, so these are dead. Guard with
             // a runtime cfg-style flag so the parser stays valid.
             (core::mem::size_of::<wchar_t>() > 2) &&
             ( ((ucs as u32) >= 0x20000 && (ucs as u32) <= 0x2fffd) ||
               ((ucs as u32) >= 0x30000 && (ucs as u32) <= 0x3fffd) )
         )
        ) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn mk_wcswidth(mut pwcs: *const wchar_t, mut n: usize) -> i32 {
    let mut w: i32;
    let mut width = 0i32;

    while *pwcs != 0 && n > 0 {
        n = n.wrapping_sub(1);
        w = mk_wcwidth(*pwcs);
        if w < 0 {
            return -1;
        } else {
            width += w;
        }
        pwcs = pwcs.add(1);
    }

    width
}

#[no_mangle]
pub unsafe extern "C" fn mk_wcwidth_cjk(ucs: wchar_t) -> i32 {
    static ambiguous: [interval; 156] = [
        interval { first: 0x00A1, last: 0x00A1 }, interval { first: 0x00A4, last: 0x00A4 }, interval { first: 0x00A7, last: 0x00A8 },
        interval { first: 0x00AA, last: 0x00AA }, interval { first: 0x00AE, last: 0x00AE }, interval { first: 0x00B0, last: 0x00B4 },
        interval { first: 0x00B6, last: 0x00BA }, interval { first: 0x00BC, last: 0x00BF }, interval { first: 0x00C6, last: 0x00C6 },
        interval { first: 0x00D0, last: 0x00D0 }, interval { first: 0x00D7, last: 0x00D8 }, interval { first: 0x00DE, last: 0x00E1 },
        interval { first: 0x00E6, last: 0x00E6 }, interval { first: 0x00E8, last: 0x00EA }, interval { first: 0x00EC, last: 0x00ED },
        interval { first: 0x00F0, last: 0x00F0 }, interval { first: 0x00F2, last: 0x00F3 }, interval { first: 0x00F7, last: 0x00FA },
        interval { first: 0x00FC, last: 0x00FC }, interval { first: 0x00FE, last: 0x00FE }, interval { first: 0x0101, last: 0x0101 },
        interval { first: 0x0111, last: 0x0111 }, interval { first: 0x0113, last: 0x0113 }, interval { first: 0x011B, last: 0x011B },
        interval { first: 0x0126, last: 0x0127 }, interval { first: 0x012B, last: 0x012B }, interval { first: 0x0131, last: 0x0133 },
        interval { first: 0x0138, last: 0x0138 }, interval { first: 0x013F, last: 0x0142 }, interval { first: 0x0144, last: 0x0144 },
        interval { first: 0x0148, last: 0x014B }, interval { first: 0x014D, last: 0x014D }, interval { first: 0x0152, last: 0x0153 },
        interval { first: 0x0166, last: 0x0167 }, interval { first: 0x016B, last: 0x016B }, interval { first: 0x01CE, last: 0x01CE },
        interval { first: 0x01D0, last: 0x01D0 }, interval { first: 0x01D2, last: 0x01D2 }, interval { first: 0x01D4, last: 0x01D4 },
        interval { first: 0x01D6, last: 0x01D6 }, interval { first: 0x01D8, last: 0x01D8 }, interval { first: 0x01DA, last: 0x01DA },
        interval { first: 0x01DC, last: 0x01DC }, interval { first: 0x0251, last: 0x0251 }, interval { first: 0x0261, last: 0x0261 },
        interval { first: 0x02C4, last: 0x02C4 }, interval { first: 0x02C7, last: 0x02C7 }, interval { first: 0x02C9, last: 0x02CB },
        interval { first: 0x02CD, last: 0x02CD }, interval { first: 0x02D0, last: 0x02D0 }, interval { first: 0x02D8, last: 0x02DB },
        interval { first: 0x02DD, last: 0x02DD }, interval { first: 0x02DF, last: 0x02DF }, interval { first: 0x0391, last: 0x03A1 },
        interval { first: 0x03A3, last: 0x03A9 }, interval { first: 0x03B1, last: 0x03C1 }, interval { first: 0x03C3, last: 0x03C9 },
        interval { first: 0x0401, last: 0x0401 }, interval { first: 0x0410, last: 0x044F }, interval { first: 0x0451, last: 0x0451 },
        interval { first: 0x2010, last: 0x2010 }, interval { first: 0x2013, last: 0x2016 }, interval { first: 0x2018, last: 0x2019 },
        interval { first: 0x201C, last: 0x201D }, interval { first: 0x2020, last: 0x2022 }, interval { first: 0x2024, last: 0x2027 },
        interval { first: 0x2030, last: 0x2030 }, interval { first: 0x2032, last: 0x2033 }, interval { first: 0x2035, last: 0x2035 },
        interval { first: 0x203B, last: 0x203B }, interval { first: 0x203E, last: 0x203E }, interval { first: 0x2074, last: 0x2074 },
        interval { first: 0x207F, last: 0x207F }, interval { first: 0x2081, last: 0x2084 }, interval { first: 0x20AC, last: 0x20AC },
        interval { first: 0x2103, last: 0x2103 }, interval { first: 0x2105, last: 0x2105 }, interval { first: 0x2109, last: 0x2109 },
        interval { first: 0x2113, last: 0x2113 }, interval { first: 0x2116, last: 0x2116 }, interval { first: 0x2121, last: 0x2122 },
        interval { first: 0x2126, last: 0x2126 }, interval { first: 0x212B, last: 0x212B }, interval { first: 0x2153, last: 0x2154 },
        interval { first: 0x215B, last: 0x215E }, interval { first: 0x2160, last: 0x216B }, interval { first: 0x2170, last: 0x2179 },
        interval { first: 0x2190, last: 0x2199 }, interval { first: 0x21B8, last: 0x21B9 }, interval { first: 0x21D2, last: 0x21D2 },
        interval { first: 0x21D4, last: 0x21D4 }, interval { first: 0x21E7, last: 0x21E7 }, interval { first: 0x2200, last: 0x2200 },
        interval { first: 0x2202, last: 0x2203 }, interval { first: 0x2207, last: 0x2208 }, interval { first: 0x220B, last: 0x220B },
        interval { first: 0x220F, last: 0x220F }, interval { first: 0x2211, last: 0x2211 }, interval { first: 0x2215, last: 0x2215 },
        interval { first: 0x221A, last: 0x221A }, interval { first: 0x221D, last: 0x2220 }, interval { first: 0x2223, last: 0x2223 },
        interval { first: 0x2225, last: 0x2225 }, interval { first: 0x2227, last: 0x222C }, interval { first: 0x222E, last: 0x222E },
        interval { first: 0x2234, last: 0x2237 }, interval { first: 0x223C, last: 0x223D }, interval { first: 0x2248, last: 0x2248 },
        interval { first: 0x224C, last: 0x224C }, interval { first: 0x2252, last: 0x2252 }, interval { first: 0x2260, last: 0x2261 },
        interval { first: 0x2264, last: 0x2267 }, interval { first: 0x226A, last: 0x226B }, interval { first: 0x226E, last: 0x226F },
        interval { first: 0x2282, last: 0x2283 }, interval { first: 0x2286, last: 0x2287 }, interval { first: 0x2295, last: 0x2295 },
        interval { first: 0x2299, last: 0x2299 }, interval { first: 0x22A5, last: 0x22A5 }, interval { first: 0x22BF, last: 0x22BF },
        interval { first: 0x2312, last: 0x2312 }, interval { first: 0x2460, last: 0x24E9 }, interval { first: 0x24EB, last: 0x254B },
        interval { first: 0x2550, last: 0x2573 }, interval { first: 0x2580, last: 0x258F }, interval { first: 0x2592, last: 0x2595 },
        interval { first: 0x25A0, last: 0x25A1 }, interval { first: 0x25A3, last: 0x25A9 }, interval { first: 0x25B2, last: 0x25B3 },
        interval { first: 0x25B6, last: 0x25B7 }, interval { first: 0x25BC, last: 0x25BD }, interval { first: 0x25C0, last: 0x25C1 },
        interval { first: 0x25C6, last: 0x25C8 }, interval { first: 0x25CB, last: 0x25CB }, interval { first: 0x25CE, last: 0x25D1 },
        interval { first: 0x25E2, last: 0x25E5 }, interval { first: 0x25EF, last: 0x25EF }, interval { first: 0x2605, last: 0x2606 },
        interval { first: 0x2609, last: 0x2609 }, interval { first: 0x260E, last: 0x260F }, interval { first: 0x2614, last: 0x2615 },
        interval { first: 0x261C, last: 0x261C }, interval { first: 0x261E, last: 0x261E }, interval { first: 0x2640, last: 0x2640 },
        interval { first: 0x2642, last: 0x2642 }, interval { first: 0x2660, last: 0x2661 }, interval { first: 0x2663, last: 0x2665 },
        interval { first: 0x2667, last: 0x266A }, interval { first: 0x266C, last: 0x266D }, interval { first: 0x266F, last: 0x266F },
        interval { first: 0x273D, last: 0x273D }, interval { first: 0x2776, last: 0x277F }, interval { first: 0xE000, last: 0xF8FF },
        interval { first: 0xFFFD, last: 0xFFFD }, interval { first: 0xF0000, last: 0xFFFFD }, interval { first: 0x100000, last: 0x10FFFD }
    ];

    if bisearch(ucs, ambiguous.as_ptr(), (ambiguous.len() - 1) as i32) != 0 {
        return 2;
    }

    mk_wcwidth(ucs)
}

#[no_mangle]
pub unsafe extern "C" fn mk_wcswidth_cjk(mut pwcs: *const wchar_t, mut n: usize) -> i32 {
    let mut w: i32;
    let mut width = 0i32;

    while *pwcs != 0 && n > 0 {
        n = n.wrapping_sub(1);
        w = mk_wcwidth_cjk(*pwcs);
        if w < 0 {
            return -1;
        } else {
            width += w;
        }
        pwcs = pwcs.add(1);
    }

    width
}