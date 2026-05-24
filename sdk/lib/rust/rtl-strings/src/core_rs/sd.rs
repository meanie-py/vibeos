//! Wave SD — pure decision kernels of `sdk/lib/rtl/sd.c`.
//!
//! `SECURITY_DESCRIPTOR_RELATIVE` is a fixed 20-byte record (header
//! u8/u8/u16 + four `ULONG` offsets) — identical layout on every
//! arch, so the byte-diff harness covers it directly.
//! `SECURITY_DESCRIPTOR` (absolute) bears pointers and so differs in
//! size by arch; the cabi shims operate on it via raw pointers cast
//! to a `#[repr(C)]` overlay, which is per-target by construction.
//!
//! No allocation, no `unsafe`. The `_SEH2_TRY` in `RtlValidSecurity-
//! Descriptor` is dead code under DUMMY_PSEH on all 3 kernel
//! arches; the diff harness drives only valid-ish (in-buffer) bytes
//! into it.

#![allow(dead_code)]

pub const SECURITY_DESCRIPTOR_REVISION: u32 = 1;
pub const SECURITY_DESCRIPTOR_REVISION1: u32 = 1;

// Control bits (SECURITY_DESCRIPTOR_CONTROL = USHORT).
pub const SE_OWNER_DEFAULTED: u16 = 0x0001;
pub const SE_GROUP_DEFAULTED: u16 = 0x0002;
pub const SE_DACL_PRESENT: u16 = 0x0004;
pub const SE_DACL_DEFAULTED: u16 = 0x0008;
pub const SE_SACL_PRESENT: u16 = 0x0010;
pub const SE_SACL_DEFAULTED: u16 = 0x0020;
pub const SE_DACL_UNTRUSTED: u16 = 0x0040;
pub const SE_SERVER_SECURITY: u16 = 0x0080;
pub const SE_DACL_AUTO_INHERIT_REQ: u16 = 0x0100;
pub const SE_SACL_AUTO_INHERIT_REQ: u16 = 0x0200;
pub const SE_DACL_AUTO_INHERITED: u16 = 0x0400;
pub const SE_SACL_AUTO_INHERITED: u16 = 0x0800;
pub const SE_DACL_PROTECTED: u16 = 0x1000;
pub const SE_SACL_PROTECTED: u16 = 0x2000;
pub const SE_RM_CONTROL_VALID: u16 = 0x4000;
pub const SE_SELF_RELATIVE: u16 = 0x8000;

/// Mask used by `RtlSetControlSecurityDescriptor` /
/// `RtlSetAttributesSecurityDescriptor` to reject non-attribute
/// control bits.
pub const SET_ATTRIBUTES_MASK: u16 = SE_DACL_UNTRUSTED
    | SE_SERVER_SECURITY
    | SE_DACL_AUTO_INHERIT_REQ
    | SE_SACL_AUTO_INHERIT_REQ
    | SE_DACL_AUTO_INHERITED
    | SE_SACL_AUTO_INHERITED
    | SE_DACL_PROTECTED
    | SE_SACL_PROTECTED;

// SECURITY_INFORMATION bits (subset used here).
pub const OWNER_SECURITY_INFORMATION: u32 = 0x00000001;
pub const GROUP_SECURITY_INFORMATION: u32 = 0x00000002;
pub const DACL_SECURITY_INFORMATION: u32 = 0x00000004;
pub const SACL_SECURITY_INFORMATION: u32 = 0x00000008;

// SECURITY_DESCRIPTOR_RELATIVE field offsets (flat, all arches):
pub const SDREL_OFF_REVISION: usize = 0;
pub const SDREL_OFF_SBZ1: usize = 1;
pub const SDREL_OFF_CONTROL: usize = 2;
pub const SDREL_OFF_OWNER: usize = 4;
pub const SDREL_OFF_GROUP: usize = 8;
pub const SDREL_OFF_SACL: usize = 12;
pub const SDREL_OFF_DACL: usize = 16;
pub const SIZEOF_SECURITY_DESCRIPTOR_RELATIVE: usize = 20;

/// `sizeof(SID)` per winnt.h (ANYSIZE_ARRAY = 1): 1+1+6+4 = 12.
pub const SIZEOF_SID_HEADER: u32 = 12;
/// `sizeof(ACL)`: 1+1+2+2+2 = 8.
pub const SIZEOF_ACL_HEADER: u32 = 8;

#[inline]
fn u16le(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}
#[inline]
fn u32le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// `RtlpValidateSDOffsetAndSize` — pure u32 helper. Returns
/// `Some(max_length_safe_to_read)` or `None` on rejection.
///
/// Verbatim port: rejects `offset < sizeof(SDREL)`, `offset >= length`,
/// `length - offset < min_length`, and ULONG-misaligned offsets.
#[inline]
pub fn validate_sd_offset_and_size(offset: u32, length: u32, min_length: u32) -> Option<u32> {
    if (offset as usize) < SIZEOF_SECURITY_DESCRIPTOR_RELATIVE {
        return None;
    }
    if offset >= length {
        return None;
    }
    if length - offset < min_length {
        return None;
    }
    if offset & 3 != 0 {
        return None;
    }
    Some(length - offset)
}

/// `RtlValidRelativeSecurityDescriptor` over the bytes of a
/// putatively self-relative SD. Verbatim port of the genuine
/// validator's sequence (calls `RtlValidSid` / `RtlValidAcl` via
/// the in-tree pure ports).
pub fn valid_relative_sd(sd: &[u8], required_info: u32) -> bool {
    use super::{acl as aclm, sid};

    if sd.len() < SIZEOF_SECURITY_DESCRIPTOR_RELATIVE {
        return false;
    }
    let revision = sd[SDREL_OFF_REVISION];
    let control = u16le(sd, SDREL_OFF_CONTROL);
    if revision as u32 != SECURITY_DESCRIPTOR_REVISION {
        return false;
    }
    if control & SE_SELF_RELATIVE == 0 {
        return false;
    }

    let owner_off = u32le(sd, SDREL_OFF_OWNER);
    let group_off = u32le(sd, SDREL_OFF_GROUP);
    let sacl_off = u32le(sd, SDREL_OFF_SACL);
    let dacl_off = u32le(sd, SDREL_OFF_DACL);
    let total_len = sd.len() as u32;

    // Owner
    if owner_off != 0 {
        let max = match validate_sd_offset_and_size(owner_off, total_len, SIZEOF_SID_HEADER) {
            Some(l) => l,
            None => return false,
        };
        let off = owner_off as usize;
        // RtlValidSid: read rev (off), sub_count (off+1), check rev+sub
        // bounds against the buffer. The C reads SID through a (PSID)cast
        // pointer so we just feed the trailing bytes to the in-tree port.
        if off + 2 > sd.len() {
            return false;
        }
        // For RtlValidSid we need the full SID; just call valid() on rev/count.
        // valid(rev,count,is_null) — is_null=false because we have a pointer.
        if !sid::valid(sd[off], sd[off + 1], false) {
            return false;
        }
        if max < sid::length(sd[off + 1]) {
            return false;
        }
    } else if required_info & OWNER_SECURITY_INFORMATION != 0 {
        return false;
    }

    // Group
    if group_off != 0 {
        let max = match validate_sd_offset_and_size(group_off, total_len, SIZEOF_SID_HEADER) {
            Some(l) => l,
            None => return false,
        };
        let off = group_off as usize;
        if off + 2 > sd.len() {
            return false;
        }
        if !sid::valid(sd[off], sd[off + 1], false) {
            return false;
        }
        if max < sid::length(sd[off + 1]) {
            return false;
        }
    } else if required_info & GROUP_SECURITY_INFORMATION != 0 {
        return false;
    }

    // DACL (note: only checked when the PRESENT bit is set; C does not
    // consult required_info for DACL/SACL — "Windows allows no DACL/
    // SACL even if RequiredInfo wants it").
    if control & SE_DACL_PRESENT != 0 {
        let max = match validate_sd_offset_and_size(dacl_off, total_len, SIZEOF_ACL_HEADER) {
            Some(l) => l,
            None => return false,
        };
        let off = dacl_off as usize;
        if off + 8 > sd.len() {
            return false;
        }
        // valid_acl wants the ACL bytes of length AclSize:
        let acl_size = u16le(sd, off + 2) as usize;
        if off + acl_size > sd.len() {
            return false;
        }
        if !aclm::valid_acl(&sd[off..off + acl_size]) {
            return false;
        }
        if (max as usize) < acl_size {
            return false;
        }
    }

    // SACL
    if control & SE_SACL_PRESENT != 0 {
        let max = match validate_sd_offset_and_size(sacl_off, total_len, SIZEOF_ACL_HEADER) {
            Some(l) => l,
            None => return false,
        };
        let off = sacl_off as usize;
        if off + 8 > sd.len() {
            return false;
        }
        let acl_size = u16le(sd, off + 2) as usize;
        if off + acl_size > sd.len() {
            return false;
        }
        if !aclm::valid_acl(&sd[off..off + acl_size]) {
            return false;
        }
        if (max as usize) < acl_size {
            return false;
        }
    }

    true
}
