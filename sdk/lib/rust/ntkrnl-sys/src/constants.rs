//! Constants from Windows/ReactOS kernel headers.
//!
//! These are C `#define` constants that bindgen cannot capture.
//! Organized by subsystem for easy reference.

use crate::base::*;

// ============================================================================
// NTSTATUS codes (from ntstatus.h)
// ============================================================================

pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001_u32 as i32;
pub const STATUS_INVALID_HANDLE: NTSTATUS = 0xC0000008_u32 as i32;
pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as i32;
pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022_u32 as i32;
pub const STATUS_OBJECT_TYPE_MISMATCH: NTSTATUS = 0xC0000024_u32 as i32;
pub const STATUS_OBJECT_NAME_COLLISION: NTSTATUS = 0xC0000035_u32 as i32;
pub const STATUS_QUOTA_EXCEEDED: NTSTATUS = 0xC0000044_u32 as i32;
pub const STATUS_PROCESS_IS_TERMINATING: NTSTATUS = 0xC000010A_u32 as i32;
pub const STATUS_HANDLE_NOT_CLOSABLE: NTSTATUS = 0xC0000235_u32 as i32;
pub const STATUS_OBJECT_NAME_EXISTS: NTSTATUS = 0x40000000_u32 as i32;

// ============================================================================
// OBJ_* — Object attribute flags (from obtypes.h, ob.h)
// ============================================================================

pub const OBJ_INHERIT: ULONG = 0x00000002;
pub const OBJ_PERMANENT: ULONG = 0x00000010;
pub const OBJ_EXCLUSIVE: ULONG = 0x00000020;
pub const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
pub const OBJ_OPENIF: ULONG = 0x00000080;
pub const OBJ_KERNEL_HANDLE: ULONG = 0x00000200;
pub const OBJ_FORCE_ACCESS_CHECK: ULONG = 0x00000400;
pub const OBJ_OPENLINK: ULONG = 0x00000100;
pub const OBJ_KERNEL_EXCLUSIVE: ULONG = 0x00010000;

// Internal handle entry attribute bits (from ob.h)
pub const OBJ_PROTECT_CLOSE: ULONG = 0x01;
pub const OBJ_AUDIT_OBJECT_CLOSE: ULONG = 0x04;
pub const OBJ_HANDLE_ATTRIBUTES: ULONG = OBJ_PROTECT_CLOSE | OBJ_INHERIT | OBJ_AUDIT_OBJECT_CLOSE;

// ============================================================================
// OB_FLAG_* — Object header flags (from obtypes.h)
// ============================================================================

pub const OB_FLAG_CREATE_INFO: u8 = 0x01;
pub const OB_FLAG_KERNEL_MODE: u8 = 0x02;
pub const OB_FLAG_CREATOR_INFO: u8 = 0x04;
pub const OB_FLAG_EXCLUSIVE: u8 = 0x08;
pub const OB_FLAG_PERMANENT: u8 = 0x10;
pub const OB_FLAG_SECURITY: u8 = 0x20;
pub const OB_FLAG_SINGLE_PROCESS: u8 = 0x40;
pub const OB_FLAG_DEFER_DELETE: u8 = 0x80;

// ============================================================================
// OB internal constants (from ob.h)
// ============================================================================

/// Access bit used to mark handles as protect-from-close.
pub const ObpAccessProtectCloseBit: ULONG = 0x02000000;

/// Sentinel value for system process quota (not a real pointer).
pub const OBP_SYSTEM_PROCESS_QUOTA: ULONG_PTR = 1;

/// Kernel-only flag for name info QueryReferences.
pub const OB_FLAG_KERNEL_EXCLUSIVE: ULONG = 0x40000000;

// ============================================================================
// Access mask constants (from setypes.h, winnt.h)
// ============================================================================

pub const GENERIC_READ: ULONG = 0x80000000;
pub const GENERIC_WRITE: ULONG = 0x40000000;
pub const GENERIC_EXECUTE: ULONG = 0x20000000;
pub const GENERIC_ALL: ULONG = 0x10000000;
pub const GENERIC_ACCESS: ULONG = GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE | GENERIC_ALL;
pub const MAXIMUM_ALLOWED: ULONG = 0x02000000;
pub const ACCESS_SYSTEM_SECURITY: ULONG = 0x01000000;

// ============================================================================
// Process access rights (from pstypes.h)
// ============================================================================

pub const PROCESS_DUP_HANDLE: ULONG = 0x0040;

// ============================================================================
// Duplicate handle flags (from obtypes.h)
// ============================================================================

pub const DUPLICATE_CLOSE_SOURCE: ULONG = 0x00000001;
pub const DUPLICATE_SAME_ACCESS: ULONG = 0x00000002;
pub const DUPLICATE_SAME_ATTRIBUTES: ULONG = 0x00000004;

// ============================================================================
// Security descriptor flags (from setypes.h)
// ============================================================================

pub const SE_SACL_PRESENT: u16 = 0x0010;

// ============================================================================
// Bugcheck codes (from bugcodes.mc)
// ============================================================================

pub const INVALID_KERNEL_HANDLE: ULONG = 0x93;

// ============================================================================
// Pool tags — OB subsystem (from ob.h)
// ============================================================================

pub const TAG_OB_HANDLE: ULONG = u32::from_le_bytes(*b"ObHd");
pub const TAG_OB_TEMP_STORAGE: ULONG = u32::from_le_bytes(*b"ObSt");

// ============================================================================
// Kernel/User mode constants
// ============================================================================

pub const KernelMode: i8 = 0;
pub const UserMode: i8 = 1;

// ============================================================================
// Additional NTSTATUS codes (from ntstatus.h)
// ============================================================================

pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034_u32 as i32;
pub const STATUS_OBJECT_NAME_INVALID: NTSTATUS = 0xC0000033_u32 as i32;
pub const STATUS_OBJECT_PATH_NOT_FOUND: NTSTATUS = 0xC000003A_u32 as i32;
pub const STATUS_OBJECT_PATH_INVALID: NTSTATUS = 0xC0000039_u32 as i32;
pub const STATUS_OBJECT_PATH_SYNTAX_BAD: NTSTATUS = 0xC000003B_u32 as i32;
pub const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004_u32 as i32;
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023_u32 as i32;
pub const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x80000005_u32 as i32;
pub const STATUS_PRIVILEGE_NOT_HELD: NTSTATUS = 0xC0000061_u32 as i32;
pub const STATUS_REPARSE: NTSTATUS = 0x00000104;
pub const STATUS_REPARSE_OBJECT: NTSTATUS = 0x00000118;
pub const STATUS_NO_MORE_ENTRIES: NTSTATUS = 0x8000001A_u32 as i32;
pub const STATUS_INVALID_INFO_CLASS: NTSTATUS = 0xC0000003_u32 as i32;
pub const STATUS_INVALID_PARAMETER_1: NTSTATUS = 0xC00000EF_u32 as i32;
pub const STATUS_BAD_IMPERSONATION_LEVEL: NTSTATUS = 0xC00000A5_u32 as i32;

// ============================================================================
// Additional OBJ_* flags (from obtypes.h)
// ============================================================================

pub const OBJ_VALID_KERNEL_ATTRIBUTES: ULONG = OBJ_VALID_ATTRIBUTES | OBJ_KERNEL_EXCLUSIVE;

pub const OBJ_VALID_ATTRIBUTES: ULONG = 0x000007F2;

// ============================================================================
// OB pool tags (from ob.h)
// ============================================================================

pub const OB_NAME_TAG: ULONG = u32::from_le_bytes(*b"ObNm");
pub const OB_DIR_TAG: ULONG = u32::from_le_bytes(*b"ObDi");
pub const TAG_SD: ULONG = u32::from_le_bytes(*b"ObSd");
pub const TAG_SEC_QUERY: ULONG = u32::from_le_bytes(*b"ObSq");
pub const TAG_DACL: ULONG = u32::from_le_bytes(*b"SeAc");
pub const OB_LINK_TAG: ULONG = u32::from_le_bytes(*b"ObSy");
pub const TAG_OB_WAIT: ULONG = u32::from_le_bytes(*b"ObWt");

// ============================================================================
// OB internal constants (from ob.h)
// ============================================================================

pub const OBP_NAME_LOOKASIDE_MAX_SIZE: ULONG = 248;
pub const OBP_CREATE_INFO_LOOKASIDE_MAX_SIZE: ULONG = 0; // placeholder
pub const OBJ_NAME_PATH_SEPARATOR: WCHAR = b'\\' as WCHAR;

// ============================================================================
// Access mask constants (from winnt.h)
// ============================================================================

pub const DELETE: ULONG = 0x00010000;
pub const READ_CONTROL: ULONG = 0x00020000;
pub const WRITE_DAC: ULONG = 0x00040000;
pub const WRITE_OWNER: ULONG = 0x00080000;
pub const SYNCHRONIZE: ULONG = 0x00100000;
pub const STANDARD_RIGHTS_REQUIRED: ULONG = DELETE | READ_CONTROL | WRITE_DAC | WRITE_OWNER;
pub const STANDARD_RIGHTS_READ: ULONG = READ_CONTROL;
pub const STANDARD_RIGHTS_WRITE: ULONG = READ_CONTROL;
pub const STANDARD_RIGHTS_EXECUTE: ULONG = READ_CONTROL;
pub const STANDARD_RIGHTS_ALL: ULONG = DELETE | READ_CONTROL | WRITE_DAC | WRITE_OWNER | SYNCHRONIZE;
pub const SPECIFIC_RIGHTS_ALL: ULONG = 0x0000FFFF;

// ============================================================================
// Directory access rights (from obtypes.h)
// ============================================================================

pub const DIRECTORY_QUERY: ULONG = 0x0001;
pub const DIRECTORY_TRAVERSE: ULONG = 0x0002;
pub const DIRECTORY_CREATE_OBJECT: ULONG = 0x0004;
pub const DIRECTORY_CREATE_SUBDIRECTORY: ULONG = 0x0008;
pub const DIRECTORY_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | 0x000F;

// ============================================================================
// Symbolic link access rights (from obtypes.h)
// ============================================================================

pub const SYMBOLIC_LINK_QUERY: ULONG = 0x0001;
pub const SYMBOLIC_LINK_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | 0x0001;

// ============================================================================
// Security constants (from setypes.h)
// ============================================================================

pub const ACL_REVISION: ULONG = 2;
pub const ACL_REVISION_DS: ULONG = 4;
pub const SECURITY_DESCRIPTOR_REVISION: ULONG = 1;
pub const SE_DACL_PRESENT: u16 = 0x0004;
pub const SE_SELF_RELATIVE: u16 = 0x8000;
pub const ACCESS_ALLOWED_ACE_TYPE: UCHAR = 0;

// Security information (from winnt.h)
pub const OWNER_SECURITY_INFORMATION: ULONG = 0x00000001;
pub const GROUP_SECURITY_INFORMATION: ULONG = 0x00000002;
pub const DACL_SECURITY_INFORMATION: ULONG = 0x00000004;
pub const SACL_SECURITY_INFORMATION: ULONG = 0x00000008;
pub const LABEL_SECURITY_INFORMATION: ULONG = 0x00000010;

// ============================================================================
// Lookaside list constants
// ============================================================================

pub const LookasideCreateInfoList: ULONG = 3;
pub const LookasideNameBufferList: ULONG = 4;
