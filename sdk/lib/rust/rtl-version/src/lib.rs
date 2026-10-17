//! Rust translation of `sdk/lib/rtl/version.c` — the OS-version query
//! helpers: `RtlVerifyVersionInfo` and `VerSetConditionMask` (plus the
//! file-local `RtlpVerCompare` / `RtlpVerGetCondition`).
//!
//! Self-contained leaf: the `OSVERSIONINFO*` structs and `VER_*` /
//! `STATUS_*` constants are fixed-layout winnt.h values, identical on all
//! three arches, so they're defined locally rather than pulled from
//! ntkrnl-sys. The only external symbol is `RtlGetVersion` (defined
//! per-module: ntoskrnl/rtl/misc.c kernel-side, ntdll user-side).
//!
//! Faithful to ReactOS version.c (Filip Navara, Hermès Belusca-Maïto).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_upper_case_globals)]

pub type NTSTATUS = i32;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000D_u32 as i32;
const STATUS_REVISION_MISMATCH: NTSTATUS = 0xC000_0059_u32 as i32;

// --- VER_* condition operators (winnt.h), UCHAR-domain values. ---
const VER_EQUAL: u8 = 1;
const VER_GREATER: u8 = 2;
const VER_GREATER_EQUAL: u8 = 3;
const VER_LESS: u8 = 4;
const VER_LESS_EQUAL: u8 = 5;
const VER_AND: u8 = 6;
const VER_OR: u8 = 7;
const VER_CONDITION_MASK: u8 = 7;
const VER_NUM_BITS_PER_CONDITION_MASK: u32 = 3;

// --- VER_* type-mask flags (winnt.h), ULONG-domain values. ---
const VER_MINORVERSION: u32 = 0x0000001;
const VER_MAJORVERSION: u32 = 0x0000002;
const VER_BUILDNUMBER: u32 = 0x0000004;
const VER_PLATFORMID: u32 = 0x0000008;
const VER_SERVICEPACKMINOR: u32 = 0x0000010;
const VER_SERVICEPACKMAJOR: u32 = 0x0000020;
const VER_SUITENAME: u32 = 0x0000040;
const VER_PRODUCT_TYPE: u32 = 0x0000080;

/// `RTL_OSVERSIONINFOW` (winnt.h) — base struct passed to RtlGetVersion.
/// Fixed layout, identical on all arches.
#[repr(C)]
pub struct OSVERSIONINFOW {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
    pub szCSDVersion: [u16; 128],
}

/// `RTL_OSVERSIONINFOEXW` (winnt.h) — the base struct plus the
/// service-pack / suite / product-type tail.
#[repr(C)]
pub struct OSVERSIONINFOEXW {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
    pub szCSDVersion: [u16; 128],
    pub wServicePackMajor: u16,
    pub wServicePackMinor: u16,
    pub wSuiteMask: u16,
    pub wProductType: u8,
    pub wReserved: u8,
}

extern "system" {
    fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> NTSTATUS;
}

/// `RtlpVerCompare` — apply one VER_* comparison operator. File-local.
fn RtlpVerCompare(left: u32, right: u32, condition: u8) -> bool {
    match condition {
        VER_EQUAL => left == right,
        VER_GREATER => left > right,
        VER_GREATER_EQUAL => left >= right,
        VER_LESS => left < right,
        VER_LESS_EQUAL => left <= right,
        _ => false,
    }
}

/// `RtlpVerGetCondition` — extract the 3-bit condition for `TypeMask`
/// from the packed `ConditionMask`. File-local. The else-if order is
/// load-bearing (lexicographical Major;Minor;SPMajor;SPMinor selection),
/// so it is preserved exactly from the C.
fn RtlpVerGetCondition(condition_mask: u64, type_mask: u32) -> u8 {
    let mut condition: u8 = 0;

    if type_mask & VER_PRODUCT_TYPE != 0 {
        condition |= (condition_mask >> (7 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    } else if type_mask & VER_SUITENAME != 0 {
        condition |= (condition_mask >> (6 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    } else if type_mask & VER_PLATFORMID != 0 {
        condition |= (condition_mask >> (3 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    } else if type_mask & VER_BUILDNUMBER != 0 {
        condition |= (condition_mask >> (2 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    } else if type_mask & VER_MAJORVERSION != 0 {
        condition |= (condition_mask >> (1 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    } else if type_mask & VER_MINORVERSION != 0 {
        condition |= (condition_mask >> (0 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    } else if type_mask & VER_SERVICEPACKMAJOR != 0 {
        condition |= (condition_mask >> (5 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    } else if type_mask & VER_SERVICEPACKMINOR != 0 {
        condition |= (condition_mask >> (4 * VER_NUM_BITS_PER_CONDITION_MASK)) as u8;
    }

    condition & VER_CONDITION_MASK
}

/// `NTSTATUS NTAPI RtlVerifyVersionInfo(PRTL_OSVERSIONINFOEXW, ULONG, ULONGLONG)`.
#[cfg_attr(feature = "exports", no_mangle)]
pub unsafe extern "system" fn RtlVerifyVersionInfo(
    VersionInfo: *mut OSVERSIONINFOEXW,
    TypeMask: u32,
    ConditionMask: u64,
) -> NTSTATUS {
    let mut Version: OSVERSIONINFOEXW = core::mem::zeroed();
    Version.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as u32;

    let Status = RtlGetVersion(core::ptr::addr_of_mut!(Version) as *mut OSVERSIONINFOW);
    if Status != STATUS_SUCCESS {
        return Status;
    }
    verify_against(&Version, VersionInfo, TypeMask, ConditionMask)
}

/// RtlVerifyVersionInfo's comparison against a given current version (`Version`), so a
/// caller that reports a different version (kernel32's manifest rule) checks against that.
pub unsafe fn verify_against(
    Version: &OSVERSIONINFOEXW,
    VersionInfo: *mut OSVERSIONINFOEXW,
    TypeMask: u32,
    ConditionMask: u64,
) -> NTSTATUS {
    let mut Comparison: bool;
    let mut DoNextCheck: bool;
    let mut Condition: u8;

    if TypeMask == 0 || ConditionMask == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if TypeMask & VER_PRODUCT_TYPE != 0 {
        Comparison = RtlpVerCompare(
            Version.wProductType as u32,
            (*VersionInfo).wProductType as u32,
            RtlpVerGetCondition(ConditionMask, VER_PRODUCT_TYPE),
        );
        if !Comparison {
            return STATUS_REVISION_MISMATCH;
        }
    }

    if TypeMask & VER_SUITENAME != 0 {
        match RtlpVerGetCondition(ConditionMask, VER_SUITENAME) {
            VER_AND => {
                if ((*VersionInfo).wSuiteMask & Version.wSuiteMask) != (*VersionInfo).wSuiteMask {
                    return STATUS_REVISION_MISMATCH;
                }
            }
            VER_OR => {
                if ((*VersionInfo).wSuiteMask & Version.wSuiteMask) == 0
                    && (*VersionInfo).wSuiteMask != 0
                {
                    return STATUS_REVISION_MISMATCH;
                }
            }
            _ => {
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    if TypeMask & VER_PLATFORMID != 0 {
        Comparison = RtlpVerCompare(
            Version.dwPlatformId,
            (*VersionInfo).dwPlatformId,
            RtlpVerGetCondition(ConditionMask, VER_PLATFORMID),
        );
        if !Comparison {
            return STATUS_REVISION_MISMATCH;
        }
    }

    if TypeMask & VER_BUILDNUMBER != 0 {
        Comparison = RtlpVerCompare(
            Version.dwBuildNumber,
            (*VersionInfo).dwBuildNumber,
            RtlpVerGetCondition(ConditionMask, VER_BUILDNUMBER),
        );
        if !Comparison {
            return STATUS_REVISION_MISMATCH;
        }
    }

    DoNextCheck = true;
    Condition = VER_EQUAL;

    if TypeMask & VER_MAJORVERSION != 0 {
        Condition = RtlpVerGetCondition(ConditionMask, VER_MAJORVERSION);
        DoNextCheck = (*VersionInfo).dwMajorVersion == Version.dwMajorVersion;
        Comparison = RtlpVerCompare(
            Version.dwMajorVersion,
            (*VersionInfo).dwMajorVersion,
            Condition,
        );

        if !Comparison && !DoNextCheck {
            return STATUS_REVISION_MISMATCH;
        }
    }

    if DoNextCheck {
        if TypeMask & VER_MINORVERSION != 0 {
            if Condition == VER_EQUAL {
                Condition = RtlpVerGetCondition(ConditionMask, VER_MINORVERSION);
            }

            DoNextCheck = (*VersionInfo).dwMinorVersion == Version.dwMinorVersion;
            Comparison = RtlpVerCompare(
                Version.dwMinorVersion,
                (*VersionInfo).dwMinorVersion,
                Condition,
            );

            if !Comparison && !DoNextCheck {
                return STATUS_REVISION_MISMATCH;
            }
        }

        if DoNextCheck && (TypeMask & VER_SERVICEPACKMAJOR != 0) {
            if Condition == VER_EQUAL {
                Condition = RtlpVerGetCondition(ConditionMask, VER_SERVICEPACKMAJOR);
            }

            DoNextCheck = (*VersionInfo).wServicePackMajor == Version.wServicePackMajor;
            Comparison = RtlpVerCompare(
                Version.wServicePackMajor as u32,
                (*VersionInfo).wServicePackMajor as u32,
                Condition,
            );

            if !Comparison && !DoNextCheck {
                return STATUS_REVISION_MISMATCH;
            }

            if DoNextCheck && (TypeMask & VER_SERVICEPACKMINOR != 0) {
                if Condition == VER_EQUAL {
                    Condition = RtlpVerGetCondition(ConditionMask, VER_SERVICEPACKMINOR);
                }

                Comparison = RtlpVerCompare(
                    Version.wServicePackMinor as u32,
                    (*VersionInfo).wServicePackMinor as u32,
                    Condition,
                );

                if !Comparison {
                    return STATUS_REVISION_MISMATCH;
                }
            }
        }
    }

    STATUS_SUCCESS
}

/// `ULONGLONG NTAPI VerSetConditionMask(ULONGLONG, ULONG, UCHAR)`.
#[cfg_attr(feature = "exports", no_mangle)]
pub unsafe extern "system" fn VerSetConditionMask(
    mut ConditionMask: u64,
    TypeMask: u32,
    Condition: u8,
) -> u64 {
    if TypeMask == 0 {
        return ConditionMask;
    }

    let Condition = Condition & VER_CONDITION_MASK;

    if Condition == 0 {
        return ConditionMask;
    }

    let ullCondMask = Condition as u64;
    if TypeMask & VER_PRODUCT_TYPE != 0 {
        ConditionMask |= ullCondMask << (7 * VER_NUM_BITS_PER_CONDITION_MASK);
    } else if TypeMask & VER_SUITENAME != 0 {
        ConditionMask |= ullCondMask << (6 * VER_NUM_BITS_PER_CONDITION_MASK);
    } else if TypeMask & VER_SERVICEPACKMAJOR != 0 {
        ConditionMask |= ullCondMask << (5 * VER_NUM_BITS_PER_CONDITION_MASK);
    } else if TypeMask & VER_SERVICEPACKMINOR != 0 {
        ConditionMask |= ullCondMask << (4 * VER_NUM_BITS_PER_CONDITION_MASK);
    } else if TypeMask & VER_PLATFORMID != 0 {
        ConditionMask |= ullCondMask << (3 * VER_NUM_BITS_PER_CONDITION_MASK);
    } else if TypeMask & VER_BUILDNUMBER != 0 {
        ConditionMask |= ullCondMask << (2 * VER_NUM_BITS_PER_CONDITION_MASK);
    } else if TypeMask & VER_MAJORVERSION != 0 {
        ConditionMask |= ullCondMask << (1 * VER_NUM_BITS_PER_CONDITION_MASK);
    } else if TypeMask & VER_MINORVERSION != 0 {
        ConditionMask |= ullCondMask << (0 * VER_NUM_BITS_PER_CONDITION_MASK);
    }

    ConditionMask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_roundtrip() {
        // VerSetConditionMask then RtlpVerGetCondition recovers the op.
        let m = unsafe { VerSetConditionMask(0, VER_MAJORVERSION, VER_GREATER_EQUAL) };
        assert_eq!(RtlpVerGetCondition(m, VER_MAJORVERSION), VER_GREATER_EQUAL);
        let m = unsafe { VerSetConditionMask(m, VER_MINORVERSION, VER_LESS) };
        assert_eq!(RtlpVerGetCondition(m, VER_MINORVERSION), VER_LESS);
        assert_eq!(RtlpVerGetCondition(m, VER_MAJORVERSION), VER_GREATER_EQUAL);
    }

    #[test]
    fn product_type_slot() {
        let m = unsafe { VerSetConditionMask(0, VER_PRODUCT_TYPE, VER_EQUAL) };
        assert_eq!(RtlpVerGetCondition(m, VER_PRODUCT_TYPE), VER_EQUAL);
    }

    #[test]
    fn compare_ops() {
        assert!(RtlpVerCompare(6, 6, VER_EQUAL));
        assert!(RtlpVerCompare(7, 6, VER_GREATER));
        assert!(RtlpVerCompare(6, 6, VER_GREATER_EQUAL));
        assert!(!RtlpVerCompare(5, 6, VER_GREATER));
        assert!(RtlpVerCompare(5, 6, VER_LESS));
        assert!(!RtlpVerCompare(6, 6, VER_LESS));
        assert!(!RtlpVerCompare(6, 6, 0)); // unknown op → false
    }
}
