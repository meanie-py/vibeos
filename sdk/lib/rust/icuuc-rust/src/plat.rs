//! Platform seam. Everything that needs kernel32 goes through `Platform` so the state
//! machine in engine.rs can be unit tested on the host with `Fake`.

#[derive(Clone, Copy)]
pub struct CpInfo {
    #[allow(dead_code)]
    pub max_char: u8,
    /// First byte of CPINFOEX.DefaultChar (the code page's substitution byte).
    pub default_char: u8,
}

pub trait Platform {
    fn acp() -> u32;
    fn cp_info(cp: u32) -> Option<CpInfo>;
    fn is_lead(cp: u32, b: u8) -> bool;
    /// Strict decode of exactly one complete character (1..=4 bytes). None = invalid/unmapped.
    fn mb_to_wc(cp: u32, src: &[u8], dst: &mut [u16; 4]) -> Option<usize>;
    /// Encode one scalar given as 1 or 2 UTF-16 units. None = no mapping (would use default char).
    fn wc_to_mb(cp: u32, src: &[u16], dst: &mut [u8; 8]) -> Option<usize>;
}

#[cfg(not(test))]
pub type Plat = win::Win;
#[cfg(test)]
pub type Plat = fake::Fake;

#[cfg(not(test))]
pub mod win {
    use super::{CpInfo, Platform};
    use core::mem::MaybeUninit;

    #[repr(C)]
    struct CpInfoExW {
        max_char_size: u32,
        default_char: [u8; 2],
        lead_byte: [u8; 12],
        unicode_default_char: u16,
        code_page: u32,
        code_page_name: [u16; 260],
    }

    extern "system" {
        fn GetACP() -> u32;
        fn GetCPInfoExW(cp: u32, flags: u32, info: *mut CpInfoExW) -> i32;
        fn IsDBCSLeadByteEx(cp: u32, b: u8) -> i32;
        fn MultiByteToWideChar(cp: u32, flags: u32, src: *const u8, cb: i32, dst: *mut u16, cch: i32) -> i32;
        fn WideCharToMultiByte(
            cp: u32, flags: u32, src: *const u16, cch: i32, dst: *mut u8, cb: i32,
            def: *const u8, used: *mut i32,
        ) -> i32;
        fn GetLastError() -> u32;
    }

    const MB_ERR_INVALID_CHARS: u32 = 8;
    const WC_NO_BEST_FIT_CHARS: u32 = 0x400;
    const ERROR_INVALID_FLAGS: u32 = 1004;

    /// Code pages for which WideCharToMultiByte requires dwFlags == 0 and NULL default-char pointers.
    fn plain_wc(cp: u32) -> bool {
        matches!(cp, 42 | 52936 | 54936 | 65000 | 65001 | 50220..=50229 | 57002..=57011)
    }

    pub struct Win;
    impl Platform for Win {
        fn acp() -> u32 {
            unsafe { GetACP() }
        }
        fn cp_info(cp: u32) -> Option<CpInfo> {
            unsafe {
                let mut i = MaybeUninit::<CpInfoExW>::uninit();
                if GetCPInfoExW(cp, 0, i.as_mut_ptr()) == 0 {
                    return None;
                }
                let i = i.assume_init();
                Some(CpInfo { max_char: i.max_char_size as u8, default_char: i.default_char[0] })
            }
        }
        fn is_lead(cp: u32, b: u8) -> bool {
            unsafe { IsDBCSLeadByteEx(cp, b) != 0 }
        }
        fn mb_to_wc(cp: u32, src: &[u8], dst: &mut [u16; 4]) -> Option<usize> {
            unsafe {
                let mut n = MultiByteToWideChar(cp, MB_ERR_INVALID_CHARS, src.as_ptr(), src.len() as i32, dst.as_mut_ptr(), 4);
                if n == 0 && GetLastError() == ERROR_INVALID_FLAGS {
                    n = MultiByteToWideChar(cp, 0, src.as_ptr(), src.len() as i32, dst.as_mut_ptr(), 4);
                }
                if n <= 0 { None } else { Some(n as usize) }
            }
        }
        fn wc_to_mb(cp: u32, src: &[u16], dst: &mut [u8; 8]) -> Option<usize> {
            unsafe {
                let n;
                if plain_wc(cp) {
                    n = WideCharToMultiByte(cp, 0, src.as_ptr(), src.len() as i32, dst.as_mut_ptr(), 8,
                                            core::ptr::null(), core::ptr::null_mut());
                } else {
                    let mut used = 0i32;
                    n = WideCharToMultiByte(cp, WC_NO_BEST_FIT_CHARS, src.as_ptr(), src.len() as i32,
                                            dst.as_mut_ptr(), 8, core::ptr::null(), &mut used);
                    if used != 0 {
                        return None;
                    }
                }
                if n <= 0 { None } else { Some(n as usize) }
            }
        }
    }
}

#[cfg(test)]
pub mod fake {
    use super::{CpInfo, Platform};

    pub const DBCS: u32 = 0xFFF1;
    pub const SBCS: u32 = 0xFFF2;

    /// Toy platform. fake-dbcs: 0x00-0x7F ASCII, 0xA1-0xDF -> U+FF61.., lead bytes 0x81-0x9F with
    /// trail 0x40-0x7E -> U+4E00 + ((lead-0x81)<<6) + (trail-0x40). fake-sbcs: 0x00-0x7F ASCII,
    /// 0x81 undefined, other 0x80-0xFF -> U+0100 + (b-0x80). Every other code page is ASCII-only.
    pub struct Fake;
    impl Platform for Fake {
        fn acp() -> u32 { 28591 }
        fn cp_info(_cp: u32) -> Option<CpInfo> { Some(CpInfo { max_char: 2, default_char: b'?' }) }
        fn is_lead(cp: u32, b: u8) -> bool { cp == DBCS && (0x81..=0x9F).contains(&b) }
        fn mb_to_wc(cp: u32, src: &[u8], dst: &mut [u16; 4]) -> Option<usize> {
            let u = match (cp, src) {
                (_, [b]) if *b < 0x80 => *b as u16,
                (DBCS, [b]) if (0xA1..=0xDF).contains(b) => 0xFF61 + (*b as u16 - 0xA1),
                (DBCS, [l, t]) if (0x81..=0x9F).contains(l) && (0x40..=0x7E).contains(t) =>
                    0x4E00 + (((*l as u16) - 0x81) << 6) + (*t as u16 - 0x40),
                (SBCS, [b]) if *b != 0x81 => 0x100 + (*b as u16 - 0x80),
                _ => return None,
            };
            dst[0] = u;
            Some(1)
        }
        fn wc_to_mb(cp: u32, src: &[u16], dst: &mut [u8; 8]) -> Option<usize> {
            if src.len() != 1 { return None; }
            let u = src[0];
            if u < 0x80 { dst[0] = u as u8; return Some(1); }
            match cp {
                DBCS if (0xFF61..=0xFF9F).contains(&u) => { dst[0] = 0xA1 + (u - 0xFF61) as u8; Some(1) }
                DBCS if (0x4E00..0x4E00 + 0x1F * 0x40).contains(&u) => {
                    let o = u - 0x4E00;
                    dst[0] = 0x81 + (o >> 6) as u8;
                    dst[1] = 0x40 + (o & 0x3F) as u8;
                    Some(2)
                }
                SBCS if (0x100..0x180).contains(&u) && u != 0x101 => { dst[0] = 0x80 + (u - 0x100) as u8; Some(1) }
                _ => None,
            }
        }
    }
}
