//! Wave ACL — pure decision kernels of `sdk/lib/rtl/acl.c`. ACL/ACE
//! are fixed binary records (all fields u8/u16/u32 — no pointers, so
//! the layout is identical on every arch):
//!
//! ```text
//! ACL          off0 AclRevision:u8  off1 Sbz1:u8  off2 AclSize:u16
//!              off4 AceCount:u16     off6 Sbz2:u16            (=8)
//! ACE_HEADER   off0 AceType:u8  off1 AceFlags:u8  off2 AceSize:u16 (=4)
//! ACE          ACE_HEADER + AccessMask:u32                    (=8)
//! KNOWN_ACE    ACE_HEADER + Mask:u32 + SidStart           (off8 SID)
//! KNOWN_OBJ_ACE ACE_HEADER + Mask:u32 + Flags:u32 + SidStart (off12)
//! ```
//!
//! No allocation, no `unsafe`. Faithful to acl.c incl. the dead
//! `_SEH2` in `RtlValidAcl` (DUMMY_PSEH: body runs, EXCEPT dead — a
//! wild OOB read there is genuine UB, never fuzzed).

use super::sid;

pub const ACL_HDR: usize = 8;
pub const ACE_HDR: usize = 4;
pub const SIZEOF_ACE: usize = 8;
pub const SIZEOF_KNOWN_OBJECT_ACE: usize = 16;
pub const GUID_SZ: usize = 16;

pub const MIN_ACL_REVISION: u8 = 2;
pub const MAX_ACL_REVISION: u8 = 4;
pub const ACL_REVISION3: u32 = 3;
pub const ACL_REVISION4: u32 = 4;

pub const ACCESS_ALLOWED_ACE_TYPE: u8 = 0x0;
pub const ACCESS_DENIED_ACE_TYPE: u8 = 0x1;
pub const SYSTEM_AUDIT_ACE_TYPE: u8 = 0x2;
pub const ACCESS_MAX_MS_V2_ACE_TYPE: u8 = 0x3;
pub const ACCESS_ALLOWED_COMPOUND_ACE_TYPE: u8 = 0x4;
pub const ACCESS_MAX_MS_V3_ACE_TYPE: u8 = 0x4;
pub const ACCESS_MIN_MS_OBJECT_ACE_TYPE: u8 = 0x5;
pub const ACCESS_ALLOWED_OBJECT_ACE_TYPE: u8 = 0x5;
pub const ACCESS_DENIED_OBJECT_ACE_TYPE: u8 = 0x6;
pub const SYSTEM_AUDIT_OBJECT_ACE_TYPE: u8 = 0x7;
pub const ACCESS_MAX_MS_OBJECT_ACE_TYPE: u8 = 0x8;
pub const ACCESS_MAX_MS_ACE_TYPE: u8 = 0x8;
pub const SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE: u8 = 0xF;

pub const VALID_INHERIT_FLAGS: u32 = 0x1F;
pub const SUCCESSFUL_ACCESS_ACE_FLAG: u32 = 0x40;
pub const FAILED_ACCESS_ACE_FLAG: u32 = 0x80;
pub const ACE_OBJECT_TYPE_PRESENT: u32 = 0x1;
pub const ACE_INHERITED_OBJECT_TYPE_PRESENT: u32 = 0x2;

pub const ACL_REVISION_INFORMATION: u32 = 1;
pub const ACL_SIZE_INFORMATION: u32 = 2;

#[inline]
pub fn round_down(n: u32, align: u32) -> u32 {
    n & !(align - 1)
}
#[inline]
pub fn round_up(n: u32, align: u32) -> u32 {
    round_down(n.wrapping_add(align - 1), align)
}

#[inline]
fn u16le(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}
#[inline]
fn u32le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// `RtlValidAcl` over the ACL bytes (`acl.len()` MUST equal the
/// header `AclSize`; the cabi shim slices it so). Verbatim port of
/// the genuine validator's check sequence. Reads of SID fields that
/// the C does ahead of a full bounds check are kept in the same
/// order — for fuzzed valid-ish inputs they stay in-slice; the
/// wild-OOB case is the genuine `_SEH2` UB (unfuzzed).
pub fn valid_acl(acl: &[u8]) -> bool {
    if acl.len() < ACL_HDR {
        return false;
    }
    let revision = acl[0] as u8;
    let acl_size = u16le(acl, 2) as usize;
    let ace_count = u16le(acl, 4) as usize;

    if revision < MIN_ACL_REVISION || revision > MAX_ACL_REVISION {
        return false;
    }
    // USHORT-aligned.
    if round_down(acl_size as u32, 2) != acl_size as u32 {
        return false;
    }
    if acl_size < ACL_HDR {
        return false;
    }

    let mut ace = ACL_HDR; // offset of the first ACE
    for _ in 0..ace_count {
        // Space for the ACE header.
        if ace + ACE_HDR >= acl_size {
            return false;
        }
        if ace + 4 > acl.len() {
            return false;
        }
        let ace_type = acl[ace];
        let ace_size = u16le(acl, ace + 2) as usize;

        // ACE length USHORT-aligned.
        if round_down(ace_size as u32, 2) != ace_size as u32 {
            return false;
        }
        // Space for the entire ACE.
        if ace + ace_size > acl_size {
            return false;
        }

        if ace_type <= ACCESS_MAX_MS_V2_ACE_TYPE {
            if round_down(ace_size as u32, 4) != ace_size as u32 {
                return false;
            }
            if ace_size < ACE_HDR {
                return false;
            }
            // SID at &KNOWN_ACE.SidStart = ace + 8.
            let sid_off = ace + 8;
            if sid_off + 2 > acl.len() {
                return false;
            }
            if acl[sid_off] != sid::SID_REVISION {
                return false;
            }
            let sub = acl[sid_off + 1];
            if sub > sid::SID_MAX_SUB_AUTHORITIES {
                return false;
            }
            if ace_size < ACE_HDR + sid::length(sub) as usize {
                return false;
            }
        } else if ace_type == ACCESS_ALLOWED_OBJECT_ACE_TYPE
            || ace_type == ACCESS_DENIED_OBJECT_ACE_TYPE
        {
            if revision < ACL_REVISION4 as u8 {
                return false;
            }
            if round_down(ace_size as u32, 4) != ace_size as u32 {
                return false;
            }
            if ace_size < SIZEOF_KNOWN_OBJECT_ACE {
                return false;
            }
            // Flags at KNOWN_OBJECT_ACE.Flags = ace + 8.
            let flags = u32le(acl, ace + 8);
            let mut guid_size = 0usize;
            if flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                guid_size += GUID_SZ;
            }
            if flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                guid_size += GUID_SZ;
            }
            // SID at &KNOWN_OBJECT_ACE.SidStart (ace+12) + guid_size.
            let sid_off = ace + 12 + guid_size;
            if sid_off + 2 > acl.len() {
                return false;
            }
            if acl[sid_off] != sid::SID_REVISION {
                return false;
            }
            let sub = acl[sid_off + 1];
            if sub > sid::SID_MAX_SUB_AUTHORITIES {
                return false;
            }
            let required =
                (SIZEOF_KNOWN_OBJECT_ACE - 4) + guid_size + sid::length(sub) as usize;
            if ace_size < required {
                return false;
            }
        } else if ace_type == ACCESS_ALLOWED_COMPOUND_ACE_TYPE {
            // "Unsupported in ReactOS, assuming valid"
        } else if ace_type >= ACCESS_MIN_MS_OBJECT_ACE_TYPE
            && ace_type <= ACCESS_MAX_MS_OBJECT_ACE_TYPE
        {
            // "Unsupported in ReactOS, assuming valid"
        } else {
            if ace_size < ACE_HDR {
                return false;
            }
        }

        ace += ace_size;
    }
    true
}
