//! The board's NVRAM file (`brcmfmac43455-sdio.txt`) as the firmware reads it.
//!
//! The file is `key=value` lines with `#` comments; the chip wants the pairs
//! NUL-separated, padded to a word with at least one NUL, then a length token:
//! the length in words in the low half and its complement in the high half. The
//! host writes that blob at the top of the chip's RAM. A port of OpenBSD's
//! `bwfm_nvram_convert` (bwfm.c, ISC; notice in chip.rs), without its FDT MAC
//! address, which the Pis' file does not need (the chip has its own OTP MAC).

/// The converted size can grow by the padding and the token; this much spare
/// covers both.
pub const SLACK: usize = 8;

/// Convert `src` into `out`, returning the blob's length (a multiple of 4), or
/// None when `out` is shorter than `src.len() + SLACK`.
pub fn convert(src: &[u8], out: &mut [u8]) -> Option<usize> {
    if out.len() < src.len() + SLACK {
        return None;
    }
    let mut n = 0;
    let (mut count, mut skip) = (0usize, false);
    for &c in src {
        if c == b'\n' {
            if count > 0 {
                out[n] = 0;
                n += 1;
            }
            count = 0;
            skip = false;
            continue;
        }
        if skip {
            continue;
        }
        if c == b'#' && count == 0 {
            skip = true;
            continue;
        }
        if c == b'\r' {
            continue;
        }
        out[n] = c;
        n += 1;
        count += 1;
    }
    // At least one NUL, then up to the next word.
    let pad = (n + 1).div_ceil(4) * 4 - n;
    out[n..n + pad].fill(0);
    n += pad;
    let words = (n / 4) as u32 & 0xFFFF;
    out[n..n + 4].copy_from_slice(&(words | !words << 16).to_le_bytes());
    Some(n + 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_go_lines_split_on_nul_and_the_token_counts_words() {
        let src = b"# board\r\nboardflags=0x00480201\r\n\n#x=1\nccode=X2 \nregrev=1";
        let mut out = [0xEEu8; 128];
        let n = convert(src, &mut out).unwrap();
        // The last line has no newline: it is still terminated by the padding.
        let body = b"boardflags=0x00480201\0ccode=X2 \0regrev=1\0";
        assert_eq!(&out[..body.len()], body);
        assert_eq!(n % 4, 0);
        let words = ((n - 4) / 4) as u32;
        assert_eq!(&out[n - 4..n], &(words | !words << 16).to_le_bytes());
        assert!(out[body.len()..n - 4].iter().all(|&b| b == 0));
    }

    #[test]
    fn an_exact_word_still_gets_a_terminating_nul() {
        let mut out = [0u8; 32];
        let n = convert(b"ab=1\n", &mut out).unwrap(); // "ab=1\0" is 5 bytes
        assert_eq!(n, 8 + 4);
        let n = convert(b"abc", &mut out).unwrap(); // 3 bytes, then one NUL
        assert_eq!((n, &out[..4]), (8, &b"abc\0"[..]));
    }

    #[test]
    fn too_small_an_output_is_refused() {
        assert_eq!(convert(b"a=1", &mut [0u8; 4]), None);
    }
}
