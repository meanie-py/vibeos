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
pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034_u32 as i32;
pub const STATUS_PORT_ALREADY_SET: NTSTATUS = 0xC0000048_u32 as i32;
pub const STATUS_PORT_DISCONNECTED: NTSTATUS = 0xC0000037_u32 as i32;
pub const STATUS_REPLY_MESSAGE_MISMATCH: NTSTATUS = 0xC000021F_u32 as i32;
pub const STATUS_INVALID_PORT_HANDLE: NTSTATUS = 0xC0000042_u32 as i32;
pub const STATUS_PORT_MESSAGE_TOO_LONG: NTSTATUS = 0xC000002F_u32 as i32;
pub const STATUS_SERVER_SID_MISMATCH: NTSTATUS = 0xC00002A0_u32 as i32;
pub const STATUS_PORT_CONNECTION_REFUSED: NTSTATUS = 0xC0000041_u32 as i32;
pub const STATUS_PORT_NOT_SET: NTSTATUS = 0xC0000353_u32 as i32;
pub const STATUS_INVALID_PARAMETER_3: NTSTATUS = 0xC00000F1_u32 as i32;
pub const STATUS_INVALID_PARAMETER_4: NTSTATUS = 0xC00000F2_u32 as i32;
pub const STATUS_LPC_REPLY_LOST: NTSTATUS = 0xC0000253_u32 as i32;

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

pub const STATUS_OBJECT_NAME_INVALID: NTSTATUS = 0xC0000033_u32 as i32;
pub const STATUS_OBJECT_PATH_NOT_FOUND: NTSTATUS = 0xC000003A_u32 as i32;
pub const STATUS_OBJECT_PATH_INVALID: NTSTATUS = 0xC0000039_u32 as i32;
pub const STATUS_OBJECT_PATH_SYNTAX_BAD: NTSTATUS = 0xC000003B_u32 as i32;
pub const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004_u32 as i32;
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023_u32 as i32;
pub const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x80000005_u32 as i32;
pub const STATUS_NO_DATA_DETECTED: NTSTATUS = 0x80000022_u32 as i32;
pub const STATUS_PRIVILEGE_NOT_HELD: NTSTATUS = 0xC0000061_u32 as i32;
pub const STATUS_REPARSE: NTSTATUS = 0x00000104;
pub const STATUS_REPARSE_OBJECT: NTSTATUS = 0x00000118;
pub const STATUS_NO_MORE_ENTRIES: NTSTATUS = 0x8000001A_u32 as i32;
pub const STATUS_INVALID_INFO_CLASS: NTSTATUS = 0xC0000003_u32 as i32;
pub const STATUS_INVALID_PARAMETER_1: NTSTATUS = 0xC00000EF_u32 as i32;
pub const STATUS_INVALID_PARAMETER_2: NTSTATUS = 0xC00000F0_u32 as i32;
pub const STATUS_REVISION_MISMATCH: NTSTATUS = 0xC0000059_u32 as i32;
pub const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC00000BB_u32 as i32;
pub const STATUS_CANCELLED: NTSTATUS = 0xC0000120_u32 as i32;
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
pub const TAG_SD: ULONG = u32::from_be_bytes(*b"dSeS"); // C literal 'dSeS' -- ntoskrnl/include/internal/tag.h
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

// ============================================================================
// Security — SE_* descriptor control flags (from setypes.h / winnt.h)
// ============================================================================

pub const SE_OWNER_DEFAULTED: u16 = 0x0001;
pub const SE_GROUP_DEFAULTED: u16 = 0x0002;
// SE_DACL_PRESENT = 0x0004 already defined above
pub const SE_DACL_DEFAULTED: u16 = 0x0008;
// SE_SACL_PRESENT = 0x0010 already defined above
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
// SE_SELF_RELATIVE = 0x8000 already defined above

// ============================================================================
// Security — SE_GROUP_* flags (from setypes.h)
// ============================================================================

pub const SE_GROUP_MANDATORY: ULONG = 0x00000001;
pub const SE_GROUP_ENABLED_BY_DEFAULT: ULONG = 0x00000002;
pub const SE_GROUP_ENABLED: ULONG = 0x00000004;
pub const SE_GROUP_OWNER: ULONG = 0x00000008;
pub const SE_GROUP_USE_FOR_DENY_ONLY: ULONG = 0x00000010;
pub const SE_GROUP_INTEGRITY: ULONG = 0x00000020;
pub const SE_GROUP_INTEGRITY_ENABLED: ULONG = 0x00000040;
pub const SE_GROUP_LOGON_ID: ULONG = 0xC0000000;
pub const SE_GROUP_RESOURCE: ULONG = 0x20000000;

// ============================================================================
// Security — SE_PRIVILEGE_* flags (from setypes.h)
// ============================================================================

pub const SE_PRIVILEGE_ENABLED_BY_DEFAULT: ULONG = 0x00000001;
pub const SE_PRIVILEGE_ENABLED: ULONG = 0x00000002;
pub const SE_PRIVILEGE_REMOVED: ULONG = 0x00000004;
pub const SE_PRIVILEGE_USED_FOR_ACCESS: ULONG = 0x80000000;
pub const PRIVILEGE_SET_ALL_NECESSARY: ULONG = 1;

// ============================================================================
// Security — ACE types (from winnt.h)
// ============================================================================

// ACCESS_ALLOWED_ACE_TYPE = 0 already defined above
pub const ACCESS_DENIED_ACE_TYPE: UCHAR = 1;
pub const SYSTEM_AUDIT_ACE_TYPE: UCHAR = 2;
pub const SYSTEM_ALARM_ACE_TYPE: UCHAR = 3;
pub const ACCESS_ALLOWED_COMPOUND_ACE_TYPE: UCHAR = 4;
pub const ACCESS_ALLOWED_OBJECT_ACE_TYPE: UCHAR = 5;
pub const ACCESS_DENIED_OBJECT_ACE_TYPE: UCHAR = 6;
pub const SYSTEM_AUDIT_OBJECT_ACE_TYPE: UCHAR = 7;
pub const SYSTEM_ALARM_OBJECT_ACE_TYPE: UCHAR = 8;
pub const ACCESS_ALLOWED_CALLBACK_ACE_TYPE: UCHAR = 9;
pub const ACCESS_DENIED_CALLBACK_ACE_TYPE: UCHAR = 10;
pub const ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE: UCHAR = 11;
pub const ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE: UCHAR = 12;
pub const SYSTEM_AUDIT_CALLBACK_ACE_TYPE: UCHAR = 13;
pub const SYSTEM_ALARM_CALLBACK_ACE_TYPE: UCHAR = 14;
pub const SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE: UCHAR = 15;
pub const SYSTEM_ALARM_CALLBACK_OBJECT_ACE_TYPE: UCHAR = 16;
pub const SYSTEM_MANDATORY_LABEL_ACE_TYPE: UCHAR = 17;

// ============================================================================
// Security — ACE inheritance flags (from winnt.h)
// ============================================================================

pub const OBJECT_INHERIT_ACE: UCHAR = 0x01;
pub const CONTAINER_INHERIT_ACE: UCHAR = 0x02;
pub const NO_PROPAGATE_INHERIT_ACE: UCHAR = 0x04;
pub const INHERIT_ONLY_ACE: UCHAR = 0x08;
pub const INHERITED_ACE: UCHAR = 0x10;
pub const VALID_INHERIT_FLAGS: UCHAR = 0x1F;

// ACE object type present flags
pub const ACE_OBJECT_TYPE_PRESENT: ULONG = 0x1;
pub const ACE_INHERITED_OBJECT_TYPE_PRESENT: ULONG = 0x2;

// ============================================================================
// Security — SID revision and RID constants (from winnt.h / ntseapi.h)
// ============================================================================

pub const SID_REVISION: ULONG = 1;
pub const SID_MAX_SUB_AUTHORITIES: ULONG = 15;
pub const SID_RECOMMENDED_SUB_AUTHORITIES: ULONG = 1;

pub const SECURITY_NULL_RID: ULONG = 0x00000000;
pub const SECURITY_WORLD_RID: ULONG = 0x00000000;
pub const SECURITY_LOCAL_RID: ULONG = 0x00000000;
pub const SECURITY_LOCAL_LOGON_RID: ULONG = 0x00000001;
pub const SECURITY_CREATOR_OWNER_RID: ULONG = 0x00000000;
pub const SECURITY_CREATOR_GROUP_RID: ULONG = 0x00000001;
pub const SECURITY_CREATOR_OWNER_SERVER_RID: ULONG = 0x00000002;
pub const SECURITY_CREATOR_GROUP_SERVER_RID: ULONG = 0x00000003;
pub const SECURITY_CREATOR_OWNER_RIGHTS_RID: ULONG = 0x00000004;

pub const SECURITY_DIALUP_RID: ULONG = 0x00000001;
pub const SECURITY_NETWORK_RID: ULONG = 0x00000002;
pub const SECURITY_BATCH_RID: ULONG = 0x00000003;
pub const SECURITY_INTERACTIVE_RID: ULONG = 0x00000004;
pub const SECURITY_LOGON_IDS_RID: ULONG = 0x00000005;
pub const SECURITY_SERVICE_RID: ULONG = 0x00000006;
pub const SECURITY_ANONYMOUS_LOGON_RID: ULONG = 0x00000007;
pub const SECURITY_PROXY_RID: ULONG = 0x00000008;
pub const SECURITY_ENTERPRISE_CONTROLLERS_RID: ULONG = 0x00000009;
pub const SECURITY_PRINCIPAL_SELF_RID: ULONG = 0x0000000A;
pub const SECURITY_AUTHENTICATED_USER_RID: ULONG = 0x0000000B;
pub const SECURITY_RESTRICTED_CODE_RID: ULONG = 0x0000000C;
pub const SECURITY_TERMINAL_SERVER_RID: ULONG = 0x0000000D;
pub const SECURITY_REMOTE_LOGON_RID: ULONG = 0x0000000E;
pub const SECURITY_THIS_ORGANIZATION_RID: ULONG = 0x0000000F;
pub const SECURITY_LOCAL_SYSTEM_RID: ULONG = 0x00000012;
pub const SECURITY_LOCAL_SERVICE_RID: ULONG = 0x00000013;
pub const SECURITY_NETWORK_SERVICE_RID: ULONG = 0x00000014;

pub const SECURITY_BUILTIN_DOMAIN_RID: ULONG = 0x00000020;

pub const DOMAIN_ALIAS_RID_ADMINS: ULONG = 0x00000220;
pub const DOMAIN_ALIAS_RID_USERS: ULONG = 0x00000221;
pub const DOMAIN_ALIAS_RID_GUESTS: ULONG = 0x00000222;
pub const DOMAIN_ALIAS_RID_POWER_USERS: ULONG = 0x00000223;
pub const DOMAIN_ALIAS_RID_ACCOUNT_OPS: ULONG = 0x00000224;
pub const DOMAIN_ALIAS_RID_SYSTEM_OPS: ULONG = 0x00000225;
pub const DOMAIN_ALIAS_RID_PRINT_OPS: ULONG = 0x00000226;
pub const DOMAIN_ALIAS_RID_BACKUP_OPS: ULONG = 0x00000227;

// ============================================================================
// Security descriptor revision (from winnt.h)
// ============================================================================

pub const SECURITY_DESCRIPTOR_REVISION1: ULONG = 1;
// `sizeof(SECURITY_DESCRIPTOR)` (absolute form): 20 bytes on 32-bit (4-byte
// header + four 4-byte pointers), 40 bytes on 64-bit (4-byte header + 4 pad +
// four 8-byte pointers). Hardcoding 20 on 64-bit undersizes SD allocations and
// the capture-length guard.
#[cfg(target_pointer_width = "32")]
pub const SECURITY_DESCRIPTOR_MIN_LENGTH: ULONG = 20;
#[cfg(target_pointer_width = "64")]
pub const SECURITY_DESCRIPTOR_MIN_LENGTH: ULONG = 40;

// ============================================================================
// Token access rights (from winnt.h)
// ============================================================================

pub const TOKEN_ASSIGN_PRIMARY: ULONG = 0x0001;
pub const TOKEN_DUPLICATE: ULONG = 0x0002;
pub const TOKEN_IMPERSONATE: ULONG = 0x0004;
pub const TOKEN_QUERY: ULONG = 0x0008;
pub const TOKEN_QUERY_SOURCE: ULONG = 0x0010;
pub const TOKEN_ADJUST_PRIVILEGES: ULONG = 0x0020;
pub const TOKEN_ADJUST_GROUPS: ULONG = 0x0040;
pub const TOKEN_ADJUST_DEFAULT: ULONG = 0x0080;
pub const TOKEN_ADJUST_SESSIONID: ULONG = 0x0100;
pub const TOKEN_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | 0x01FF;

// ============================================================================
// QOSS / Impersonation (from winnt.h)
// ============================================================================

pub const SECURITY_STATIC_TRACKING: UCHAR = 0;
pub const SECURITY_DYNAMIC_TRACKING: UCHAR = 1;

// ============================================================================
// SEF_* flags (from se.h / ntseapi.h)
// ============================================================================

pub const SEF_DACL_AUTO_INHERIT: ULONG = 0x01;
pub const SEF_SACL_AUTO_INHERIT: ULONG = 0x02;
pub const SEF_DEFAULT_DESCRIPTOR_FOR_OBJECT: ULONG = 0x04;
pub const SEF_AVOID_PRIVILEGE_CHECK: ULONG = 0x08;
pub const SEF_AVOID_OWNER_CHECK: ULONG = 0x10;
pub const SEF_DEFAULT_OWNER_FROM_PARENT: ULONG = 0x20;
pub const SEF_DEFAULT_GROUP_FROM_PARENT: ULONG = 0x40;
pub const SEF_MACL_NO_WRITE_UP: ULONG = 0x100;
pub const SEF_MACL_NO_READ_UP: ULONG = 0x200;
pub const SEF_MACL_NO_EXECUTE_UP: ULONG = 0x400;
pub const SEF_AVOID_OWNER_RESTRICTION: ULONG = 0x1000;

// ============================================================================
// SE pool tags (from se.h / sepg.h)
// ============================================================================

pub const TAG_SID: ULONG = u32::from_be_bytes(*b"iSeS"); // nolint: const-contradiction ('iSeS' -- ntoskrnl/include/internal/tag.h:151; rtl's TAG_SID is 'diSp')
pub const TAG_SID_AND_ATTRIBUTES: ULONG = u32::from_le_bytes(*b"SidA");
pub const TAG_PRIVILEGE_SET: ULONG = u32::from_be_bytes(*b"rPeS"); // C literal 'rPeS' -- ntoskrnl/include/internal/tag.h
pub const TAG_QOS: ULONG = u32::from_le_bytes(*b"SeQs");
pub const TAG_LUID: ULONG = u32::from_le_bytes(*b"SeLu");
pub const TAG_SE_HANDLES_TAB: ULONG = u32::from_le_bytes(*b"SeHt");
pub const TAG_SE_DIR_BUFFER: ULONG = u32::from_le_bytes(*b"SeDb");
pub const TAG_LOGON_SESSION: ULONG = u32::from_le_bytes(*b"SeLs");
pub const TAG_SID_VALIDATE: ULONG = u32::from_le_bytes(*b"SeSv");
pub const TAG_LOGON_NOTIFICATION: ULONG = u32::from_le_bytes(*b"SeLn");
pub const TAG_SEPA: ULONG = u32::from_le_bytes(*b"SePa");
pub const TAG_TOKEN: ULONG = u32::from_le_bytes(*b"Toke");
pub const TAG_ACL: ULONG = u32::from_be_bytes(*b"cAeS"); // C literal 'cAeS' -- ntoskrnl/include/internal/tag.h

// ============================================================================
// SE internal constants (from se.h)
// ============================================================================

pub const INITIAL_PRIVILEGE_COUNT: ULONG = 3;
pub const TOKEN_HAS_TRAVERSE_PRIVILEGE: ULONG = 0x0001;
pub const TOKEN_HAS_BACKUP_PRIVILEGE: ULONG = 0x0002;
pub const TOKEN_HAS_RESTORE_PRIVILEGE: ULONG = 0x0004;
pub const TOKEN_WRITE_RESTRICTED: ULONG = 0x0008;
pub const TOKEN_IS_RESTRICTED: ULONG = 0x0010;
pub const TOKEN_SESSION_NOT_REFERENCED: ULONG = 0x0020;
pub const TOKEN_SANDBOX_INERT: ULONG = 0x0040;
pub const TOKEN_HAS_IMPERSONATE_PRIVILEGE: ULONG = 0x0080;

// ============================================================================
// Additional NTSTATUS codes — SE subsystem (from ntstatus.h)
// ============================================================================

pub const STATUS_NOT_FOUND: NTSTATUS = 0xC0000225_u32 as i32;
pub const STATUS_NO_TOKEN: NTSTATUS = 0xC000007C_u32 as i32;
pub const STATUS_NO_SUCH_LOGON_SESSION: NTSTATUS = 0xC000005F_u32 as i32;
pub const STATUS_INVALID_SID: NTSTATUS = 0xC0000078_u32 as i32;
pub const STATUS_INVALID_OWNER: NTSTATUS = 0xC000005A_u32 as i32;
pub const STATUS_INVALID_PRIMARY_GROUP: NTSTATUS = 0xC000005B_u32 as i32;
pub const STATUS_INVALID_ACL: NTSTATUS = 0xC0000077_u32 as i32;
pub const STATUS_INVALID_SECURITY_DESCR: NTSTATUS = 0xC0000079_u32 as i32;
pub const STATUS_UNKNOWN_REVISION: NTSTATUS = 0xC0000058_u32 as i32;
pub const STATUS_GENERIC_NOT_MAPPED: NTSTATUS = 0xC00000E6_u32 as i32;
pub const STATUS_NOT_ALL_ASSIGNED: NTSTATUS = 0x00000106;
pub const STATUS_TOKEN_ALREADY_IN_USE: NTSTATUS = 0xC000012B_u32 as i32;
pub const STATUS_NO_SECURITY_ON_OBJECT: NTSTATUS = 0xC00000D7_u32 as i32;
pub const STATUS_NO_IMPERSONATION_TOKEN: NTSTATUS = 0xC000005C_u32 as i32;
pub const STATUS_LOGON_SESSION_EXISTS: NTSTATUS = 0xC00000EE_u32 as i32;
pub const STATUS_BAD_LOGON_SESSION_STATE: NTSTATUS = 0xC0000104_u32 as i32;
pub const STATUS_ALLOTTED_SPACE_EXCEEDED: NTSTATUS = 0xC0000099_u32 as i32;
pub const STATUS_BAD_TOKEN_TYPE: NTSTATUS = 0xC00000A8_u32 as i32;
pub const STATUS_CANT_OPEN_ANONYMOUS: NTSTATUS = 0xC00000A6_u32 as i32;
pub const STATUS_CANT_ENABLE_DENY_ONLY: NTSTATUS = 0xC00002B3_u32 as i32;
pub const STATUS_CANT_DISABLE_MANDATORY: NTSTATUS = 0xC000005D_u32 as i32;
// STATUS_INSUFFICIENT_RESOURCES is in base.rs

// ============================================================================
// Registry constants (from winnt.h)
// ============================================================================

pub const REG_NONE: ULONG = 0;
pub const REG_SZ: ULONG = 1;
pub const REG_EXPAND_SZ: ULONG = 2;
pub const REG_BINARY: ULONG = 3;
pub const REG_DWORD: ULONG = 4;
pub const REG_DWORD_BIG_ENDIAN: ULONG = 5;
pub const REG_LINK: ULONG = 6;
pub const REG_MULTI_SZ: ULONG = 7;

pub const KEY_QUERY_VALUE: ULONG = 0x0001;
pub const KEY_SET_VALUE: ULONG = 0x0002;
pub const KEY_CREATE_SUB_KEY: ULONG = 0x0004;
pub const KEY_ENUMERATE_SUB_KEYS: ULONG = 0x0008;
pub const KEY_NOTIFY: ULONG = 0x0010;
pub const KEY_CREATE_LINK: ULONG = 0x0020;
pub const KEY_READ: ULONG = STANDARD_RIGHTS_READ | KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS | KEY_NOTIFY;

// ============================================================================
// Object access rights — SECTION, THREAD (from winnt.h)
// ============================================================================

pub const SECTION_QUERY: ULONG = 0x0001;
pub const SECTION_MAP_WRITE: ULONG = 0x0002;
pub const SECTION_MAP_READ: ULONG = 0x0004;
pub const SECTION_MAP_EXECUTE: ULONG = 0x0008;
pub const SECTION_EXTEND_SIZE: ULONG = 0x0010;
pub const SECTION_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | 0x001F;

pub const THREAD_TERMINATE: ULONG = 0x0001;
pub const THREAD_SET_INFORMATION: ULONG = 0x0020;
pub const THREAD_SET_LIMITED_INFORMATION: ULONG = 0x0400;
pub const THREAD_QUERY_LIMITED_INFORMATION: ULONG = 0x0800;
pub const THREAD_RESUME: ULONG = 0x1000;
// The NT6 mask. sdk/include/xdk/pstypes.h selects it under __REACTOS__ too since upstream
// 4b4063a01d6, so the C and the Rust agree on it again (ps-rust's THREAD_ALL_ACCESS_NT6).
pub const THREAD_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFFF;

// ============================================================================
// Audit constants (from setypes.h / se.h)
// ============================================================================

pub const AUDIT_ALLOW_NO_PRIVILEGE: ULONG = 0x1;

// ============================================================================
// Process access rights (from pstypes.h / winnt.h)
// ============================================================================

pub const PROCESS_TERMINATE: ULONG = 0x0001;
pub const PROCESS_CREATE_THREAD: ULONG = 0x0002;
pub const PROCESS_SET_SESSIONID: ULONG = 0x0004;
pub const PROCESS_VM_OPERATION: ULONG = 0x0008;
pub const PROCESS_VM_READ: ULONG = 0x0010;
pub const PROCESS_VM_WRITE: ULONG = 0x0020;
// PROCESS_DUP_HANDLE = 0x0040 already defined above
pub const PROCESS_CREATE_PROCESS: ULONG = 0x0080;
pub const PROCESS_SET_QUOTA: ULONG = 0x0100;
pub const PROCESS_SET_INFORMATION: ULONG = 0x0200;
pub const PROCESS_QUERY_INFORMATION: ULONG = 0x0400;
pub const PROCESS_SUSPEND_RESUME: ULONG = 0x0800;
pub const PROCESS_QUERY_LIMITED_INFORMATION: ULONG = 0x1000;
pub const PROCESS_SET_LIMITED_INFORMATION: ULONG = 0x2000;
pub const PROCESS_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFFF;

// ============================================================================
// Thread access rights (from winnt.h) — additional
// ============================================================================

pub const THREAD_SUSPEND_RESUME: ULONG = 0x0002;
pub const THREAD_GET_CONTEXT: ULONG = 0x0008;
pub const THREAD_SET_CONTEXT: ULONG = 0x0010;
pub const THREAD_QUERY_INFORMATION: ULONG = 0x0040;
pub const THREAD_SET_THREAD_TOKEN: ULONG = 0x0080;
pub const THREAD_IMPERSONATE: ULONG = 0x0100;
pub const THREAD_DIRECT_IMPERSONATION: ULONG = 0x0200;

// ============================================================================
// Job object access rights (from winnt.h)
// ============================================================================

pub const JOB_OBJECT_ASSIGN_PROCESS: ULONG = 0x0001;
pub const JOB_OBJECT_SET_ATTRIBUTES: ULONG = 0x0002;
pub const JOB_OBJECT_QUERY: ULONG = 0x0004;
pub const JOB_OBJECT_TERMINATE: ULONG = 0x0008;
pub const JOB_OBJECT_SET_SECURITY_ATTRIBUTES: ULONG = 0x0010;
pub const JOB_OBJECT_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x001F;

// ndk/pstypes.h: job limit flags (JOBOBJECT_BASIC_LIMIT_INFORMATION.LimitFlags).
pub const JOB_OBJECT_LIMIT_WORKINGSET: ULONG = 0x1;
pub const JOB_OBJECT_LIMIT_PROCESS_TIME: ULONG = 0x2;
pub const JOB_OBJECT_LIMIT_JOB_TIME: ULONG = 0x4;
pub const JOB_OBJECT_LIMIT_ACTIVE_PROCESS: ULONG = 0x8;
pub const JOB_OBJECT_LIMIT_AFFINITY: ULONG = 0x10;
pub const JOB_OBJECT_LIMIT_PRIORITY_CLASS: ULONG = 0x20;
pub const JOB_OBJECT_LIMIT_PRESERVE_JOB_TIME: ULONG = 0x40;
pub const JOB_OBJECT_LIMIT_SCHEDULING_CLASS: ULONG = 0x80;
pub const JOB_OBJECT_LIMIT_PROCESS_MEMORY: ULONG = 0x100;
pub const JOB_OBJECT_LIMIT_JOB_MEMORY: ULONG = 0x200;
pub const JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION: ULONG = 0x400;
pub const JOB_OBJECT_LIMIT_BREAKAWAY_OK: ULONG = 0x800;
pub const JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK: ULONG = 0x1000;
pub const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: ULONG = 0x2000;
// ndk/pstypes.h: JOBOBJECT_BASIC_UI_RESTRICTIONS.UIRestrictionsClass.
pub const JOB_OBJECT_UILIMIT_ALL: ULONG = 0x00FF;
// ndk/pstypes.h: job completion-port messages.
pub const JOB_OBJECT_MSG_END_OF_JOB_TIME: ULONG = 1;
pub const JOB_OBJECT_MSG_END_OF_PROCESS_TIME: ULONG = 2;
pub const JOB_OBJECT_MSG_ACTIVE_PROCESS_LIMIT: ULONG = 3;
pub const JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO: ULONG = 4;
pub const JOB_OBJECT_MSG_NEW_PROCESS: ULONG = 6;
pub const JOB_OBJECT_MSG_EXIT_PROCESS: ULONG = 7;
pub const JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS: ULONG = 8;
pub const JOB_OBJECT_MSG_PROCESS_MEMORY_LIMIT: ULONG = 9;
pub const JOB_OBJECT_MSG_JOB_MEMORY_LIMIT: ULONG = 10;
// ndk/iotypes.h
pub const IO_COMPLETION_MODIFY_STATE: ULONG = 0x0002;

// ============================================================================
// Port access rights (from winnt.h)
// ============================================================================

pub const PORT_CONNECT: ULONG = 0x0001;
pub const PORT_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x0001;

// ============================================================================
// Debug object access rights (from ndk/dbgktypes.h:31-34)
// ============================================================================

// 0x0002, not 0x0004 (that is DEBUG_OBJECT_SET_INFORMATION).
pub const DEBUG_OBJECT_ADD_REMOVE_PROCESS: ULONG = 0x0002;

// ============================================================================
// Additional NTSTATUS codes — PS subsystem (from ntstatus.h)
// ============================================================================

pub const STATUS_PENDING: NTSTATUS = 0x00000103;
pub const STATUS_ALERTED: NTSTATUS = 0x00000101;
pub const STATUS_NOTHING_TO_TERMINATE: NTSTATUS = 0x0122;
pub const STATUS_THREAD_IS_TERMINATING: NTSTATUS = 0xC000004B_u32 as i32;
pub const STATUS_CANT_TERMINATE_SELF: NTSTATUS = 0xC00000DB_u32 as i32;
pub const STATUS_SYSTEM_SHUTDOWN: NTSTATUS = 0xC00002EB_u32 as i32;
pub const STATUS_PROCESS_IN_JOB: NTSTATUS = 0x00000124;
pub const STATUS_PROCESS_NOT_IN_JOB: NTSTATUS = 0x00000123;
pub const STATUS_INVALID_CID: NTSTATUS = 0xC000000B_u32 as i32;
pub const STATUS_INVALID_PARAMETER_MIX: NTSTATUS = 0xC0000030_u32 as i32;
pub const STATUS_PROCEDURE_NOT_FOUND: NTSTATUS = 0xC000007A_u32 as i32;
pub const STATUS_PAGEFILE_QUOTA_EXCEEDED: NTSTATUS = 0xC000012C_u32 as i32;

// ============================================================================
// Process priority (from pstypes.h / winbase.h)
// ============================================================================

pub const PROCESS_PRIORITY_NORMAL: ULONG = 8;
pub const PROCESS_PRIORITY_CLASS_INVALID: ULONG = 0;
pub const PROCESS_PRIORITY_CLASS_ABOVE_NORMAL: ULONG = 6;
pub const PROCESS_PRIORITY_CLASS_BELOW_NORMAL: ULONG = 5;
pub const PROCESS_PRIORITY_CLASS_HIGH: ULONG = 0x03;
pub const PROCESS_PRIORITY_CLASS_IDLE: ULONG = 0x01;
pub const PROCESS_PRIORITY_CLASS_REALTIME: ULONG = 0x04;

// ============================================================================
// Thread base priority (from winnt.h)
// ============================================================================

pub const THREAD_BASE_PRIORITY_LOWRT: i32 = 15;
pub const THREAD_BASE_PRIORITY_MAX: i32 = 2;
pub const THREAD_BASE_PRIORITY_MIN: i32 = -2;
pub const THREAD_BASE_PRIORITY_IDLE: i32 = -15;

// ============================================================================
// Memory priority / Scheduler priority (from ntoskrnl internal)
// ============================================================================

pub const HIGH_PRIORITY: ULONG = 31;
pub const LOW_PRIORITY: ULONG = 0;
pub const LOW_REALTIME_PRIORITY: ULONG = 16;
pub const MEMORY_PRIORITY_BACKGROUND: ULONG = 0;
pub const MEMORY_PRIORITY_FOREGROUND: ULONG = 2;

// ============================================================================
// Quota limits (from winnt.h)
// ============================================================================

pub const QUOTA_LIMITS_HARDWS_MIN_ENABLE: ULONG = 0x00000001;
pub const QUOTA_LIMITS_HARDWS_MIN_DISABLE: ULONG = 0x00000002;
pub const QUOTA_LIMITS_HARDWS_MAX_ENABLE: ULONG = 0x00000004;
pub const QUOTA_LIMITS_HARDWS_MAX_DISABLE: ULONG = 0x00000008;

// ============================================================================
// PS internal flags and constants (from ps.h / pstypes.h)
// ============================================================================

pub const PSF_BREAK_ON_TERMINATION_BIT: ULONG = 0x2000;
pub const PSF_SESSION_CREATION_UNDERWAY_BIT: ULONG = 0x4000;
pub const PSF_NO_DEBUG_INHERIT_BIT: ULONG = 0x2;
pub const PSF_VDM_ALLOWED_BIT: ULONG = 0x1000000;
pub const CT_BREAK_ON_TERMINATION_BIT: ULONG = 0x40;
pub const CT_HIDE_FROM_DEBUGGER_BIT: ULONG = 0x4;

pub const PROCESS_LUID_DOSDEVICES_ONLY: ULONG = 0x00000001;

pub const SEM_NOALIGNMENTFAULTEXCEPT: ULONG = 0x0004;

pub const SYSTEM_THREAD_EXCEPTION_NOT_HANDLED: ULONG = 0x7E;
pub const QUOTA_UNDERFLOW: ULONG = 0x21;

pub const ICIF_PROBE_READ: ULONG = 0x0010;

// ============================================================================
// PS pool tags (from ps.h)
// ============================================================================

pub const TAG_SHIM: ULONG = u32::from_le_bytes(*b"Shim");

// ============================================================================
// Processor constants (from ntdef.h / ke.h)
// ============================================================================

#[cfg(target_arch = "x86")]
pub const MAXIMUM_PROCESSORS: ULONG = 32;
#[cfg(not(target_arch = "x86"))]
pub const MAXIMUM_PROCESSORS: ULONG = 64;

// ============================================================================
// Additional NTSTATUS codes — Tier 2 subsystems (CC, FSRTL, KD64)
// ============================================================================

pub const STATUS_END_OF_FILE: NTSTATUS = 0xC0000011_u32 as i32;
pub const STATUS_MEDIA_WRITE_PROTECTED: NTSTATUS = 0xC00000A2_u32 as i32;
pub const STATUS_FILE_LOCK_CONFLICT: NTSTATUS = 0xC0000054_u32 as i32;
pub const STATUS_RANGE_NOT_LOCKED: NTSTATUS = 0xC000007E_u32 as i32;
pub const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC0000010_u32 as i32;
pub const STATUS_CRC_ERROR: NTSTATUS = 0xC000003F_u32 as i32;
pub const STATUS_DEVICE_DATA_ERROR: NTSTATUS = 0xC000009C_u32 as i32;
pub const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0x80000002_u32 as i32;
pub const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as i32;
pub const STATUS_ILLEGAL_INSTRUCTION: NTSTATUS = 0xC000001D_u32 as i32;
pub const STATUS_INSTRUCTION_MISALIGNMENT: NTSTATUS = 0xC00000AA_u32 as i32;
pub const STATUS_DELETE_PENDING: NTSTATUS = 0xC0000056_u32 as i32;
pub const STATUS_NOTIFY_ENUM_DIR: NTSTATUS = 0x0000010C;
pub const STATUS_NOTIFY_CLEANUP: NTSTATUS = 0x0000010B;
pub const STATUS_INVALID_OPLOCK_PROTOCOL: NTSTATUS = 0xC00000E3_u32 as i32;
pub const STATUS_OPLOCK_NOT_GRANTED: NTSTATUS = 0xC00000E2_u32 as i32;
pub const STATUS_OPLOCK_BREAK_IN_PROGRESS: NTSTATUS = 0x00000108;
pub const STATUS_BREAKPOINT: NTSTATUS = 0x80000003_u32 as i32;
pub const STATUS_SINGLE_STEP: NTSTATUS = 0x80000004_u32 as i32;
pub const STATUS_DEBUGGER_INACTIVE: NTSTATUS = 0xC0000354_u32 as i32;
pub const STATUS_ASSERTION_FAILURE: NTSTATUS = 0xC0000420_u32 as i32;
pub const STATUS_DEVICE_NOT_CONNECTED: NTSTATUS = 0xC000009D_u32 as i32;
pub const STATUS_LOCK_NOT_GRANTED: NTSTATUS = 0xC0000055_u32 as i32;
pub const STATUS_NO_SUCH_FILE: NTSTATUS = 0xC000000F_u32 as i32;
pub const STATUS_TIMEOUT: NTSTATUS = 0x00000102;
pub const STATUS_MORE_ENTRIES: NTSTATUS = 0x00000105;
pub const STATUS_SHARING_VIOLATION: NTSTATUS = 0xC0000043_u32 as i32;
pub const STATUS_UNABLE_TO_DELETE_SECTION: NTSTATUS = 0xC000001B_u32 as i32;
pub const STATUS_SECTION_NOT_EXTENDED: NTSTATUS = 0xC0000087_u32 as i32;
pub const STATUS_MAPPED_FILE_SIZE_ZERO: NTSTATUS = 0xC000011E_u32 as i32;
pub const STATUS_CANNOT_DELETE: NTSTATUS = 0xC0000121_u32 as i32;
pub const STATUS_USER_MAPPED_FILE: NTSTATUS = 0xC0000243_u32 as i32;
pub const STATUS_KEY_DELETED: NTSTATUS = 0xC000017C_u32 as i32;
pub const STATUS_DATA_OVERRUN: NTSTATUS = 0xC000003C_u32 as i32;
pub const STATUS_IMAGE_ALREADY_LOADED: NTSTATUS = 0xC000010E_u32 as i32;
pub const STATUS_LOGON_FAILURE: NTSTATUS = 0xC000006D_u32 as i32;

// ============================================================================
// File access / sharing / disposition constants (from winnt.h / iotypes.h)
// ============================================================================

pub const FILE_READ_DATA: ULONG = 0x00000001;
pub const FILE_WRITE_DATA: ULONG = 0x00000002;
pub const FILE_APPEND_DATA: ULONG = 0x00000004;
pub const FILE_READ_EA: ULONG = 0x00000008;
pub const FILE_WRITE_EA: ULONG = 0x00000010;
pub const FILE_EXECUTE: ULONG = 0x00000020;
pub const FILE_READ_ATTRIBUTES: ULONG = 0x00000080;
pub const FILE_WRITE_ATTRIBUTES: ULONG = 0x00000100;
pub const FILE_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x1FF;
pub const FILE_ATTRIBUTE_NORMAL: ULONG = 0x00000080;
pub const FILE_SHARE_READ: ULONG = 0x00000001;
pub const FILE_SHARE_WRITE: ULONG = 0x00000002;
pub const FILE_SHARE_DELETE: ULONG = 0x00000004;
pub const FILE_OPEN: ULONG = 0x00000001;

// File create options (from iotypes.h / ntdef.h)
pub const FILE_COMPLETE_IF_OPLOCKED: ULONG = 0x00000100;
pub const FILE_RESERVE_OPFILTER: ULONG = 0x00100000;

// Oplock break info values (from iotypes.h)
pub const FILE_OPLOCK_BROKEN_TO_LEVEL_2: ULONG = 0x00000007;
pub const FILE_OPLOCK_BROKEN_TO_NONE: ULONG = 0x00000008;

// ============================================================================
// I/O constants (from wdm.h / iotypes.h)
// ============================================================================

pub const IO_NO_INCREMENT: i8 = 0;
pub const IO_DISK_INCREMENT: i8 = 1;
pub const IO_KEYBOARD_INCREMENT: i8 = 6;
pub const IO_MOUSE_INCREMENT: i8 = 6;
pub const SL_FAIL_IMMEDIATELY: UCHAR = 0x01;
pub const SL_EXCLUSIVE_LOCK: UCHAR = 0x02;
pub const INVALID_HANDLE_VALUE: isize = -1;
pub const MAXULONG: ULONG = 0xFFFFFFFF;

// ============================================================================
// Keyboard/Mouse input constants (from ntddkbd.h / ntddmou.h)
// ============================================================================

pub const KEY_MAKE: USHORT = 0;
pub const KEY_BREAK: USHORT = 1;
pub const MOUSE_LEFT_BUTTON_DOWN: USHORT = 0x0001;
pub const MOUSE_LEFT_BUTTON_UP: USHORT = 0x0002;
pub const MOUSE_RIGHT_BUTTON_DOWN: USHORT = 0x0004;
pub const MOUSE_RIGHT_BUTTON_UP: USHORT = 0x0008;
pub const MOUSE_MIDDLE_BUTTON_DOWN: USHORT = 0x0010;
pub const MOUSE_MIDDLE_BUTTON_UP: USHORT = 0x0020;
pub const MOUSE_MOVE_ABSOLUTE: USHORT = 1;
pub const MOUSE_SERIAL_HARDWARE: ULONG = 0x0004;

// ============================================================================
// IRP major function codes (from iotypes.h)
// ============================================================================

pub const IRP_MJ_CREATE: UCHAR = 0x00;
pub const IRP_MJ_CLOSE: UCHAR = 0x02;
pub const IRP_MJ_READ: UCHAR = 0x03;
pub const IRP_MJ_WRITE: UCHAR = 0x04;
pub const IRP_MJ_QUERY_INFORMATION: UCHAR = 0x05;
pub const IRP_MJ_SET_INFORMATION: UCHAR = 0x06;
pub const IRP_MJ_FLUSH_BUFFERS: UCHAR = 0x09;
pub const IRP_MJ_FILE_SYSTEM_CONTROL: UCHAR = 0x0D;
pub const IRP_MJ_LOCK_CONTROL: UCHAR = 0x11;
pub const IRP_MJ_CLEANUP: UCHAR = 0x12;
pub const IRP_MJ_MAXIMUM_FUNCTION: UCHAR = 0x1B;

// ============================================================================
// IRQL constants (from wdm.h / ke.h — architecture-dependent)
// ============================================================================

// i386: IRQLs 0-31, APIC-based with PIC emulation
// amd64/arm64: IRQLs 0-15, CR8/ICC-based
#[cfg(target_arch = "x86")]
pub const HIGH_LEVEL: UCHAR = 31;
#[cfg(target_arch = "x86")]
pub const IPI_LEVEL: UCHAR = 29;

#[cfg(not(target_arch = "x86"))]
pub const HIGH_LEVEL: UCHAR = 15;
#[cfg(not(target_arch = "x86"))]
pub const PROFILE_LEVEL: UCHAR = 15;
#[cfg(not(target_arch = "x86"))]
pub const CLOCK_LEVEL: UCHAR = 13;
#[cfg(not(target_arch = "x86"))]
pub const IPI_LEVEL: UCHAR = 14;

// ============================================================================
// Pool allocation flags (from ntoskrnl)
// ============================================================================

pub const POOL_COLD_ALLOCATION: ULONG = 256;

// ============================================================================
// Pool tags — Tier 2 subsystems (from tag.h)
// ============================================================================

// NOTE: C multi-char literal 'orSF' = 0x6F725346 (GCC big-endian convention).
// In Rust: u32::from_le_bytes stores bytes at ascending addresses, so the
// LE byte representation of 0x6F725346 is [0x46, 0x53, 0x72, 0x6F] = "FSro".
pub const TAG_OPLOCK: ULONG = u32::from_le_bytes(*b"FSro");  // 'orSF' = 0x6F725346
pub const TAG_UNC: ULONG = u32::from_le_bytes(*b"FSun");     // 'nuSF' = 0x6E755346
pub const TAG_TABLE: ULONG = u32::from_le_bytes(*b"LTAB");   // 'BATL' = 0x4241544C
pub const TAG_RANGE: ULONG = u32::from_le_bytes(*b"FSRA");   // 'ARSF' = 0x41525346
pub const TAG_FLOCK: ULONG = u32::from_le_bytes(*b"FLCK");   // 'KCLF' = 0x4B434C46

// Cache Manager pool tags (from tag.h)
pub const TAG_CC: ULONG = u32::from_le_bytes(*b"Cc  ");              // '  cC' = 0x20206343
pub const TAG_VACB: ULONG = u32::from_le_bytes(*b"CcVa");            // 'aVcC' = 0x61566343
pub const TAG_SHARED_CACHE_MAP: ULONG = u32::from_le_bytes(*b"CcSc"); // 'cScC' = 0x63536343
pub const TAG_PRIVATE_CACHE_MAP: ULONG = u32::from_le_bytes(*b"CcPc"); // 'cPcC' = 0x63506343
pub const TAG_BCB: ULONG = u32::from_le_bytes(*b"CcBc");             // 'cBcC' = 0x63426343

// Kernel Debugger pool tags (from tag.h)
pub const TAG_KDBS: ULONG = u32::from_le_bytes(*b"KDBS");  // 'SBDK' = 0x5342444B
pub const TAG_KDBG: ULONG = u32::from_le_bytes(*b"KDBG");  // 'GBDK' = 0x4742444B

// ============================================================================
// FSCTL constants (from iotypes.h / devioctl.h)
// ============================================================================

pub const FSCTL_MUP_REGISTER_PROVIDER: ULONG = 0x00390014;

// Oplock-related FSCTLs — computed via CTL_CODE(FILE_DEVICE_FILE_SYSTEM, N, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_REQUEST_OPLOCK_LEVEL_1: ULONG = 0x00090000;
pub const FSCTL_REQUEST_OPLOCK_LEVEL_2: ULONG = 0x00090004;
pub const FSCTL_REQUEST_BATCH_OPLOCK: ULONG = 0x00090008;
pub const FSCTL_OPLOCK_BREAK_ACKNOWLEDGE: ULONG = 0x0009000C;
pub const FSCTL_OPBATCH_ACK_CLOSE_PENDING: ULONG = 0x00090010;
pub const FSCTL_OPLOCK_BREAK_NOTIFY: ULONG = 0x00090014;
pub const FSCTL_OPLOCK_BREAK_ACK_NO_2: ULONG = 0x00090050;
pub const FSCTL_REQUEST_FILTER_OPLOCK: ULONG = 0x0009005C;

// Volume-control FSCTLs (xdk/iotypes.h) — cdfs dispatches on these. Values
// COMPUTED by tools/gen_ioctl_consts.py, not hand-typed (§9.113); the same
// run verifies the 9 constants above still match the header.
pub const FSCTL_LOCK_VOLUME: ULONG = 0x00090018; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 6, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_UNLOCK_VOLUME: ULONG = 0x0009001c; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 7, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_DISMOUNT_VOLUME: ULONG = 0x00090020; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 8, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_IS_VOLUME_DIRTY: ULONG = 0x00090078; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 30, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_IS_VOLUME_MOUNTED: ULONG = 0x00090028; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 10, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_IS_PATHNAME_VALID: ULONG = 0x0009002c; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 11, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_INVALIDATE_VOLUMES: ULONG = 0x00090054; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 21, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_ALLOW_EXTENDED_DASD_IO: ULONG = 0x00090083; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 32, METHOD_NEITHER, FILE_ANY_ACCESS)

// ============================================================================
// ANSI_DOS wildcard characters (from fsrtl.h)
// ============================================================================

pub const ANSI_DOS_STAR: UCHAR = b'<';
pub const ANSI_DOS_QM: UCHAR = b'>';
pub const ANSI_DOS_DOT: UCHAR = b'"';

// ============================================================================
// FSRTL internal constants (from fsrtl.h)
// ============================================================================

pub const FSRTL_MAX_RESOURCES: ULONG = 16;

// FSRTL notification flags (from fsrtl.h)
pub const WATCH_TREE: ULONG = 0x01;
pub const NOTIFY_IMMEDIATELY: ULONG = 0x02;
pub const CLEANUP_IN_PROCESS: ULONG = 0x04;
pub const NOTIFY_LATER: ULONG = 0x08;
pub const WATCH_ROOT: ULONG = 0x10;
pub const DELETE_IN_PROCESS: ULONG = 0x20;

// ============================================================================
// Oplock internal flags (from ntoskrnl/fsrtl/oplock.c)
// ============================================================================

pub const NO_OPLOCK: ULONG = 0x1;
pub const LEVEL_1_OPLOCK: ULONG = 0x2;
pub const BATCH_OPLOCK: ULONG = 0x4;
pub const FILTER_OPLOCK: ULONG = 0x8;
pub const LEVEL_2_OPLOCK: ULONG = 0x10;
pub const EXCLUSIVE_LOCK: ULONG = 0x40;
pub const PENDING_LOCK: ULONG = 0x80;
pub const BROKEN_TO_LEVEL_2: ULONG = 0x100;
pub const BROKEN_TO_NONE: ULONG = 0x200;
pub const BROKEN_TO_NONE_FROM_LEVEL_2: ULONG = 0x400;
pub const BROKEN_TO_CLOSE_PENDING: ULONG = 0x800;
pub const BROKEN_ANY: ULONG = BROKEN_TO_LEVEL_2 | BROKEN_TO_NONE | BROKEN_TO_NONE_FROM_LEVEL_2 | BROKEN_TO_CLOSE_PENDING;

// ============================================================================
// Tunnel flags (from ntoskrnl/fsrtl/tunnel.c)
// ============================================================================

pub const TUNNEL_FLAG_KEY_SHORT_NAME: ULONG = 0x1;
pub const TUNNEL_FLAG_POOL: ULONG = 0x2;

// ============================================================================
// Object name constants (from obtypes.h)
// ============================================================================

pub const MAXIMUM_FILENAME_LENGTH: ULONG = 256;

// ============================================================================
// Bugcheck codes — Tier 2 (from bugcodes.mc)
// ============================================================================

pub const CACHE_MANAGER: ULONG = 0x00000034;

// ============================================================================
// Cache Manager internal constants (from cc.h)
// ============================================================================

pub const CC_API_DEBUG: ULONG = 0x01;
pub const READAHEAD_DISABLED: ULONG = 0x1;
pub const WRITEBEHIND_DISABLED: ULONG = 0x2;
pub const SHARED_CACHE_MAP_IN_CREATION: ULONG = 0x4;
pub const SHARED_CACHE_MAP_IN_LAZYWRITE: ULONG = 0x8;
pub const NODE_TYPE_DEFERRED_WRITE: u16 = 0x02FC;
pub const NODE_TYPE_PRIVATE_MAP: u16 = 0x02FE;
pub const NODE_TYPE_SHARED_MAP: u16 = 0x02FF;

// VACB constants (from cctypes.h)
pub const VACB_MAPPING_GRANULARITY: ULONG = 0x40000; // 256 KB

// Private cache map flags (from ndk/cctypes.h)
pub const PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE: ULONG = 1 << 16; // 0x10000

// Deferred write retry constant (from ntoskrnl/cc/copy.c — enum value)
pub const RetryForceCheckPerFile: ULONG = 254;

// ============================================================================
// Memory Manager constants for CC (from mm.h)
// ============================================================================

// MI_USAGE_CACHE is enum value 18 (0-indexed from MI_USAGE_NOT_SET=0)
pub const MI_USAGE_CACHE: ULONG = 18;
pub const MC_SYSTEM: ULONG = 1;

// ============================================================================
// KD64 constants (from ntoskrnl/include/internal/kd64.h)
// ============================================================================

pub const KD_BREAKPOINT_MAX: ULONG = 32;
#[cfg(target_arch = "aarch64")]
pub const KD_BREAKPOINT_SIZE: ULONG = 4; // sizeof(ULONG) on ARM64
#[cfg(not(target_arch = "aarch64"))]
pub const KD_BREAKPOINT_SIZE: ULONG = 1; // sizeof(UCHAR) on x86/amd64
pub const KD_BREAKPOINT_ACTIVE: ULONG = 0x01;
pub const KD_BREAKPOINT_PENDING: ULONG = 0x02;
pub const KD_BREAKPOINT_SUSPENDED: ULONG = 0x04;
pub const KD_BREAKPOINT_EXPIRED: ULONG = 0x08;

#[cfg(debug_assertions)]
pub const KD_DEFAULT_LOG_BUFFER_SIZE: ULONG = 0x8000;
#[cfg(not(debug_assertions))]
pub const KD_DEFAULT_LOG_BUFFER_SIZE: ULONG = 0x1000;

pub const KDP_MSG_BUFFER_SIZE: ULONG = 0x1000;

// DPFLTR_ENDOFTABLE_ID = 155 (enum value after DPFLTR_FSLIB_ID = 154)
pub const DPFLTR_ENDOFTABLE_ID: ULONG = 155;
pub const MAX_KD_COMPONENT_TABLE_ENTRIES: ULONG = DPFLTR_ENDOFTABLE_ID + 1;

// MMDBG flags: defined in mm.rs (not here, to avoid ambiguity)

// ============================================================================
// KD Packet protocol constants (from windbgkd.h)
// ============================================================================

// Packet sizes
pub const PACKET_MAX_SIZE: ULONG = 4000;
pub const DBGKD_MAXSTREAM: ULONG = 16;

// Magic packet IDs
pub const INITIAL_PACKET_ID: ULONG = 0x80800000;
pub const SYNC_PACKET_ID: ULONG = 0x00000800;

// Magic packet bytes
pub const BREAKIN_PACKET: ULONG = 0x62626262;
pub const BREAKIN_PACKET_BYTE: u8 = 0x62;
pub const PACKET_LEADER: ULONG = 0x30303030;
pub const PACKET_LEADER_BYTE: u8 = 0x30;
pub const CONTROL_PACKET_LEADER: ULONG = 0x69696969;
pub const CONTROL_PACKET_LEADER_BYTE: u8 = 0x69;
pub const PACKET_TRAILING_BYTE: u8 = 0xAA;

// Packet types
pub const PACKET_TYPE_UNUSED: ULONG = 0;
pub const PACKET_TYPE_KD_STATE_CHANGE32: ULONG = 1;
pub const PACKET_TYPE_KD_STATE_MANIPULATE: ULONG = 2;
pub const PACKET_TYPE_KD_DEBUG_IO: ULONG = 3;
pub const PACKET_TYPE_KD_ACKNOWLEDGE: ULONG = 4;
pub const PACKET_TYPE_KD_RESEND: ULONG = 5;
pub const PACKET_TYPE_KD_RESET: ULONG = 6;
pub const PACKET_TYPE_KD_STATE_CHANGE64: ULONG = 7;
pub const PACKET_TYPE_KD_POLL_BREAKIN: ULONG = 8;
pub const PACKET_TYPE_KD_TRACE_IO: ULONG = 9;
pub const PACKET_TYPE_KD_CONTROL_REQUEST: ULONG = 10;
pub const PACKET_TYPE_KD_FILE_IO: ULONG = 11;
pub const PACKET_TYPE_MAX: ULONG = 12;

// Wait state change types
pub const DbgKdMinimumStateChange: ULONG = 0x00003030;
pub const DbgKdExceptionStateChange: ULONG = 0x00003030;
pub const DbgKdLoadSymbolsStateChange: ULONG = 0x00003031;
pub const DbgKdCommandStringStateChange: ULONG = 0x00003032;
pub const DbgKdMaximumStateChange: ULONG = 0x00003033;

// Manipulate types
pub const DbgKdMinimumManipulate: ULONG = 0x00003130;
pub const DbgKdReadVirtualMemoryApi: ULONG = 0x00003130;
pub const DbgKdWriteVirtualMemoryApi: ULONG = 0x00003131;
pub const DbgKdGetContextApi: ULONG = 0x00003132;
pub const DbgKdSetContextApi: ULONG = 0x00003133;
pub const DbgKdWriteBreakPointApi: ULONG = 0x00003134;
pub const DbgKdRestoreBreakPointApi: ULONG = 0x00003135;
pub const DbgKdContinueApi: ULONG = 0x00003136;
pub const DbgKdReadControlSpaceApi: ULONG = 0x00003137;
pub const DbgKdWriteControlSpaceApi: ULONG = 0x00003138;
pub const DbgKdReadIoSpaceApi: ULONG = 0x00003139;
pub const DbgKdWriteIoSpaceApi: ULONG = 0x0000313A;
pub const DbgKdRebootApi: ULONG = 0x0000313B;
pub const DbgKdContinueApi2: ULONG = 0x0000313C;
pub const DbgKdReadPhysicalMemoryApi: ULONG = 0x0000313D;
pub const DbgKdWritePhysicalMemoryApi: ULONG = 0x0000313E;
pub const DbgKdQuerySpecialCallsApi: ULONG = 0x0000313F;
pub const DbgKdSetSpecialCallApi: ULONG = 0x00003140;
pub const DbgKdClearSpecialCallsApi: ULONG = 0x00003141;
pub const DbgKdSetInternalBreakPointApi: ULONG = 0x00003142;
pub const DbgKdGetInternalBreakPointApi: ULONG = 0x00003143;
pub const DbgKdReadIoSpaceExtendedApi: ULONG = 0x00003144;
pub const DbgKdWriteIoSpaceExtendedApi: ULONG = 0x00003145;
pub const DbgKdGetVersionApi: ULONG = 0x00003146;
pub const DbgKdWriteBreakPointExApi: ULONG = 0x00003147;
pub const DbgKdRestoreBreakPointExApi: ULONG = 0x00003148;
pub const DbgKdCauseBugCheckApi: ULONG = 0x00003149;
pub const DbgKdSwitchProcessor: ULONG = 0x00003150;
pub const DbgKdPageInApi: ULONG = 0x00003151;
pub const DbgKdReadMachineSpecificRegister: ULONG = 0x00003152;
pub const DbgKdWriteMachineSpecificRegister: ULONG = 0x00003153;
pub const DbgKdSearchMemoryApi: ULONG = 0x00003156;
pub const DbgKdGetBusDataApi: ULONG = 0x00003157;
pub const DbgKdSetBusDataApi: ULONG = 0x00003158;
pub const DbgKdCheckLowMemoryApi: ULONG = 0x00003159;
pub const DbgKdClearAllInternalBreakpointsApi: ULONG = 0x0000315A;
pub const DbgKdFillMemoryApi: ULONG = 0x0000315B;
pub const DbgKdQueryMemoryApi: ULONG = 0x0000315C;
pub const DbgKdSwitchPartition: ULONG = 0x0000315D;
pub const DbgKdWriteCustomBreakpointApi: ULONG = 0x0000315E;
pub const DbgKdGetContextExApi: ULONG = 0x0000315F;
pub const DbgKdSetContextExApi: ULONG = 0x00003160;
pub const DbgKdMaximumManipulate: ULONG = 0x00003161;

// Debug I/O types
pub const DbgKdPrintStringApi: ULONG = 0x00003230;
pub const DbgKdGetStringApi: ULONG = 0x00003231;

// Trace I/O types
pub const DbgKdPrintTraceApi: ULONG = 0x00003330;

// File I/O types
pub const DbgKdCreateFileApi: ULONG = 0x00003430;
pub const DbgKdReadFileApi: ULONG = 0x00003431;
pub const DbgKdWriteFileApi: ULONG = 0x00003432;
pub const DbgKdCloseFileApi: ULONG = 0x00003433;

// Control report flags
pub const REPORT_INCLUDES_SEGS: ULONG = 0x0001;
pub const REPORT_STANDARD_CS: ULONG = 0x0002;

// Protocol versions
pub const DBGKD_64BIT_PROTOCOL_VERSION1: ULONG = 5;
pub const DBGKD_64BIT_PROTOCOL_VERSION2: ULONG = 6;

// DBGKD version flags (from wdbgexts.h)
pub const DBGKD_VERS_FLAG_MP: u16 = 0x0001;
pub const DBGKD_VERS_FLAG_DATA: u16 = 0x0002;
pub const DBGKD_VERS_FLAG_PTR64: u16 = 0x0004;
pub const DBGKD_VERS_FLAG_NOMM: u16 = 0x0008;
pub const DBGKD_VERS_FLAG_HSS: u16 = 0x0010;
pub const DBGKD_VERS_FLAG_PARTITIONS: u16 = 0x0020;

// DBGKD major types (from wdbgexts.h — enum, DBGKD_MAJOR_NT = 0)
pub const DBGKD_MAJOR_NT: ULONG = 0;

// Query memory address spaces (from windbgkd.h)
pub const DBGKD_QUERY_MEMORY_VIRTUAL: ULONG = 0;
pub const DBGKD_QUERY_MEMORY_PROCESS: ULONG = 0;
pub const DBGKD_QUERY_MEMORY_SESSION: ULONG = 1;
pub const DBGKD_QUERY_MEMORY_KERNEL: ULONG = 2;

// Query memory flags
pub const DBGKD_QUERY_MEMORY_READ: ULONG = 0x01;
pub const DBGKD_QUERY_MEMORY_WRITE: ULONG = 0x02;
pub const DBGKD_QUERY_MEMORY_EXECUTE: ULONG = 0x04;
pub const DBGKD_QUERY_MEMORY_FIXED: ULONG = 0x08;

// Internal breakpoint flags
pub const DBGKD_INTERNAL_BP_FLAG_COUNTONLY: ULONG = 0x01;
pub const DBGKD_INTERNAL_BP_FLAG_INVALID: ULONG = 0x02;
pub const DBGKD_INTERNAL_BP_FLAG_SUSPENDED: ULONG = 0x04;
pub const DBGKD_INTERNAL_BP_FLAG_DYING: ULONG = 0x08;

// Fill memory flags
pub const DBGKD_FILL_MEMORY_VIRTUAL: ULONG = 0x01;
pub const DBGKD_FILL_MEMORY_PHYSICAL: ULONG = 0x02;

// Physical memory caching flags
pub const DBGKD_CACHING_DEFAULT: ULONG = 0;
pub const DBGKD_CACHING_CACHED: ULONG = 1;
pub const DBGKD_CACHING_UNCACHED: ULONG = 2;
pub const DBGKD_CACHING_WRITE_COMBINED: ULONG = 3;

// Partition switch flags
pub const DBGKD_PARTITION_DEFAULT: ULONG = 0x00;
pub const DBGKD_PARTITION_ALTERNATE: ULONG = 0x01;

// ============================================================================
// Exception constants (from rtltypes.h)
// ============================================================================

pub const EXCEPTION_MAXIMUM_PARAMETERS: ULONG = 15;

// ============================================================================
// Section / Memory constants (from mmtypes.h / winnt.h)
// ============================================================================

// PAGE_READWRITE = 0x04 — defined in base.rs (re-exported at crate root)
pub const SEC_COMMIT: ULONG = 0x08000000;
pub const SEC_BASED: ULONG = 0x00200000;
pub const SEC_IMAGE: ULONG = 0x01000000;
pub const SEC_LARGE_PAGES: ULONG = 0x80000000;
pub const SEC_NOCACHE: ULONG = 0x10000000;
pub const SEC_NO_CHANGE: ULONG = 0x00400000;
pub const SEC_RESERVE: ULONG = 0x04000000;
pub const SEC_WRITECOMBINE: ULONG = 0x40000000;
pub const SEC_FILE: ULONG = 0x00800000;
pub const SEC_PROTECTED_IMAGE: ULONG = 0x02000000;

// Memory allocation type flags (from mmtypes.h)
pub const MEM_FREE: ULONG = 0x00010000;
pub const MEM_PRIVATE: ULONG = 0x00020000;
pub const MEM_MAPPED: ULONG = 0x00040000;
pub const MEM_IMAGE: ULONG = 0x01000000;

// Additional NTSTATUS codes — MM subsystem
pub const STATUS_INVALID_ADDRESS: NTSTATUS = 0xC0000141_u32 as i32;
pub const STATUS_INVALID_FILE_FOR_SECTION: NTSTATUS = 0xC0000020_u32 as i32;
pub const STATUS_INVALID_PARAMETER_5: NTSTATUS = 0xC00000F3_u32 as i32;
pub const STATUS_INVALID_PARAMETER_6: NTSTATUS = 0xC00000F4_u32 as i32;
pub const STATUS_INVALID_PARAMETER_9: NTSTATUS = 0xC00000F7_u32 as i32;
pub const STATUS_INVALID_VIEW_SIZE: NTSTATUS = 0xC000001F_u32 as i32;
pub const STATUS_NOT_MAPPED_VIEW: NTSTATUS = 0xC0000019_u32 as i32;
pub const STATUS_SECTION_NOT_IMAGE: NTSTATUS = 0xC0000049_u32 as i32;
pub const STATUS_SECTION_TOO_BIG: NTSTATUS = 0xC0000040_u32 as i32;
pub const STATUS_SECTION_PROTECTION: NTSTATUS = 0xC000004E_u32 as i32;
pub const STATUS_UNABLE_TO_FREE_VM: NTSTATUS = 0xC000001A_u32 as i32;
pub const STATUS_FREE_VM_NOT_AT_BASE: NTSTATUS = 0xC000009F_u32 as i32;
pub const STATUS_MEMORY_NOT_ALLOCATED: NTSTATUS = 0xC00000A0_u32 as i32;
pub const STATUS_INCOMPATIBLE_FILE_MAP: NTSTATUS = 0xC000004D_u32 as i32;
pub const STATUS_IMAGE_NOT_AT_BASE: NTSTATUS = 0x40000003_u32 as i32;

// Bugcheck codes — MM subsystem
pub const DRIVER_UNMAPPING_INVALID_VIEW: ULONG = 0xD7;

// MAXULONG_PTR
pub const MAXULONG_PTR: ULONG_PTR = !0usize;
