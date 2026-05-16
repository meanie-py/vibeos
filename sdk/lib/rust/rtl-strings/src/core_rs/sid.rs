//! Wave SID — the pure decision kernels of `sdk/lib/rtl/sid.c`.
//! No allocation, no `unsafe`. A `SID` is a fixed binary record:
//!
//! ```text
//! off 0  Revision           : u8
//! off 1  SubAuthorityCount   : u8
//! off 2  IdentifierAuthority : [u8; 6]
//! off 8  SubAuthority        : [u32; SubAuthorityCount]   (LE)
//! ```
//!
//! `FIELD_OFFSET(SID, SubAuthority[n]) == 8 + 4*n` — both
//! `RtlLengthRequiredSid` and `RtlLengthSid` are exactly that (ULONG
//! arithmetic, wraps).
//!
//! Faithful misfeatures pinned here:
//!  - `RtlValidSid(NULL)` ⇒ **TRUE**: genuine is
//!    `if ((Sid) && (bad)) return FALSE; return TRUE;` — the `&&`
//!    short-circuits on NULL and falls through to the `TRUE`. (The
//!    `_SEH2` wrapper only guarded a bad *non-NULL* pointer; under
//!    DUMMY_PSEH that path faults = genuine UB, excluded from fuzz.)
//!  - `RtlEqualPrefixSid`: revision **and** all 6 identifier-authority
//!    bytes **and** SubAuthorityCount must match; then every
//!    subauthority **but the last** (`(i+1) < count`); `count == 0`
//!    after the authority match ⇒ TRUE.

pub const SID_REVISION: u8 = 1;
pub const SID_MAX_SUB_AUTHORITIES: u8 = 15;

/// `FIELD_OFFSET(SID, SubAuthority[count])` — ULONG, wraps like C.
#[inline]
pub fn length_required(sub_authority_count: u32) -> u32 {
    8u32.wrapping_add(4u32.wrapping_mul(sub_authority_count))
}

/// `RtlLengthSid` = `8 + 4*SubAuthorityCount` (count is the u8 field).
#[inline]
pub fn length(sub_authority_count: u8) -> u32 {
    8 + 4 * (sub_authority_count as u32)
}

/// `RtlValidSid`. `is_null` carries the NULL-pointer case (genuine
/// `&&` short-circuit ⇒ TRUE). For a non-NULL SID: valid iff
/// `(Revision & 0xF) == SID_REVISION` and
/// `SubAuthorityCount <= SID_MAX_SUB_AUTHORITIES`.
#[inline]
pub fn valid(revision: u8, sub_authority_count: u8, is_null: bool) -> bool {
    if is_null {
        return true;
    }
    !((revision & 0xF) != SID_REVISION || sub_authority_count > SID_MAX_SUB_AUTHORITIES)
}

/// `RtlEqualPrefixSid` over the two parsed records. `auth` = the 6
/// identifier-authority bytes; `subs` = the subauthority words.
pub fn equal_prefix(
    rev1: u8,
    auth1: &[u8; 6],
    subs1: &[u32],
    rev2: u8,
    auth2: &[u8; 6],
    subs2: &[u32],
) -> bool {
    if rev1 != rev2 {
        return false;
    }
    if auth1 != auth2 {
        return false;
    }
    if subs1.len() != subs2.len() {
        return false;
    }
    if subs1.is_empty() {
        return true;
    }
    // All subauthorities BUT the last one ((i+1) < count).
    let n = subs1.len() - 1;
    subs1[..n] == subs2[..n]
}

/// `RtlConvertSidToUnicodeString` text builder — the exact genuine
/// `swprintf` chain into the WCHAR Buffer[256]:
///
/// ```text
/// "S-1-"
/// IdentifierAuthority[0]==0 && [1]==0  → "%lu"   of
///     (V[2]<<24 | V[3]<<16 | V[4]<<8 | V[5])   (u32, no pad)
/// else                                 → "0x%02hx"×6  (lowercase)
/// for each subauthority                → "-%u"  (u32, no pad)
/// ```
///
/// Returns the count of u16 written (NOT incl NUL) = the genuine
/// `wcs - Buffer`. `out` must hold ≥ 256 (genuine stack buffer).
pub fn convert_text(auth: &[u8; 6], subs: &[u32], out: &mut [u16]) -> usize {
    let mut n = 0usize;
    let put = |out: &mut [u16], n: &mut usize, b: u8| {
        out[*n] = b as u16;
        *n += 1;
    };
    for &b in b"S-1-" {
        put(out, &mut n, b);
    }

    if auth[0] == 0 && auth[1] == 0 {
        let v = (auth[2] as u32) << 24
            | (auth[3] as u32) << 16
            | (auth[4] as u32) << 8
            | (auth[5] as u32);
        write_u32_dec(out, &mut n, v);
    } else {
        put(out, &mut n, b'0');
        put(out, &mut n, b'x');
        for &a in auth.iter() {
            put(out, &mut n, hex_lc(a >> 4));
            put(out, &mut n, hex_lc(a & 0xF));
        }
    }

    for &s in subs {
        put(out, &mut n, b'-');
        write_u32_dec(out, &mut n, s);
    }
    n
}

#[inline]
fn hex_lc(nib: u8) -> u8 {
    if nib < 10 {
        b'0' + nib
    } else {
        b'a' + (nib - 10)
    }
}

fn write_u32_dec(out: &mut [u16], n: &mut usize, v: u32) {
    if v == 0 {
        out[*n] = b'0' as u16;
        *n += 1;
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0usize;
    let mut x = v;
    while x > 0 {
        buf[i] = b'0' + (x % 10) as u8;
        x /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        out[*n] = buf[i] as u16;
        *n += 1;
    }
}
