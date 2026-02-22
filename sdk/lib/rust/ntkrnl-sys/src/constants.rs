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
pub const STATUS_REPLY_MESSAGE_MISMATCH: NTSTATUS = 0xC000002D_u32 as i32;
pub const STATUS_INVALID_PORT_HANDLE: NTSTATUS = 0xC0000042_u32 as i32;
pub const STATUS_PORT_MESSAGE_TOO_LONG: NTSTATUS = 0xC000002F_u32 as i32;
pub const STATUS_SERVER_SID_MISMATCH: NTSTATUS = 0xC00000A4_u32 as i32;
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
pub const SECURITY_DESCRIPTOR_MIN_LENGTH: ULONG = 20; // sizeof(SECURITY_DESCRIPTOR)

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

pub const TAG_SID: ULONG = u32::from_le_bytes(*b"Sid ");
pub const TAG_SID_AND_ATTRIBUTES: ULONG = u32::from_le_bytes(*b"SidA");
pub const TAG_PRIVILEGE_SET: ULONG = u32::from_le_bytes(*b"Priv");
pub const TAG_QOS: ULONG = u32::from_le_bytes(*b"SeQs");
pub const TAG_LUID: ULONG = u32::from_le_bytes(*b"SeLu");
pub const TAG_SE_HANDLES_TAB: ULONG = u32::from_le_bytes(*b"SeHt");
pub const TAG_SE_DIR_BUFFER: ULONG = u32::from_le_bytes(*b"SeDb");
pub const TAG_LOGON_SESSION: ULONG = u32::from_le_bytes(*b"SeLs");
pub const TAG_SID_VALIDATE: ULONG = u32::from_le_bytes(*b"SeSv");
pub const TAG_LOGON_NOTIFICATION: ULONG = u32::from_le_bytes(*b"SeLn");
pub const TAG_SEPA: ULONG = u32::from_le_bytes(*b"SePa");
pub const TAG_TOKEN: ULONG = u32::from_le_bytes(*b"Toke");
pub const TAG_ACL: ULONG = u32::from_le_bytes(*b"SeAl");

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
pub const STATUS_GENERIC_NOT_MAPPED: NTSTATUS = 0xC0000067_u32 as i32;
pub const STATUS_NOT_ALL_ASSIGNED: NTSTATUS = 0x00000106;
pub const STATUS_TOKEN_ALREADY_IN_USE: NTSTATUS = 0xC000015A_u32 as i32;
pub const STATUS_NO_SECURITY_ON_OBJECT: NTSTATUS = 0xC00000D7_u32 as i32;
pub const STATUS_NO_IMPERSONATION_TOKEN: NTSTATUS = 0xC000005C_u32 as i32;
pub const STATUS_LOGON_SESSION_EXISTS: NTSTATUS = 0xC0000068_u32 as i32;
pub const STATUS_BAD_LOGON_SESSION_STATE: NTSTATUS = 0xC0000104_u32 as i32;
pub const STATUS_ALLOTTED_SPACE_EXCEEDED: NTSTATUS = 0xC0000099_u32 as i32;
pub const STATUS_BAD_TOKEN_TYPE: NTSTATUS = 0xC00000A8_u32 as i32;
pub const STATUS_CANT_OPEN_ANONYMOUS: NTSTATUS = 0xC00000A6_u32 as i32;
pub const STATUS_CANT_ENABLE_DENY_ONLY: NTSTATUS = 0xC000025F_u32 as i32;
pub const STATUS_CANT_DISABLE_MANDATORY: NTSTATUS = 0xC0000260_u32 as i32;
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
pub const THREAD_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x03FF;

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

// ============================================================================
// Port access rights (from winnt.h)
// ============================================================================

pub const PORT_CONNECT: ULONG = 0x0001;
pub const PORT_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x0001;

// ============================================================================
// Debug object access rights (from winnt.h)
// ============================================================================

pub const DEBUG_OBJECT_ADD_REMOVE_PROCESS: ULONG = 0x0004;

// ============================================================================
// Additional NTSTATUS codes — PS subsystem (from ntstatus.h)
// ============================================================================

pub const STATUS_PENDING: NTSTATUS = 0x00000103;
pub const STATUS_ALERTED: NTSTATUS = 0x00000101;
pub const STATUS_NOTHING_TO_TERMINATE: NTSTATUS = 0xC0000187_u32 as i32;
pub const STATUS_THREAD_IS_TERMINATING: NTSTATUS = 0xC000004B_u32 as i32;
pub const STATUS_CANT_TERMINATE_SELF: NTSTATUS = 0xC00000DB_u32 as i32;
pub const STATUS_SYSTEM_SHUTDOWN: NTSTATUS = 0xC00002FE_u32 as i32;
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
pub const PROCESS_PRIORITY_CLASS_ABOVE_NORMAL: ULONG = 6;
pub const PROCESS_PRIORITY_CLASS_BELOW_NORMAL: ULONG = 5;
pub const PROCESS_PRIORITY_CLASS_HIGH: ULONG = 13;
pub const PROCESS_PRIORITY_CLASS_IDLE: ULONG = 4;
pub const PROCESS_PRIORITY_CLASS_REALTIME: ULONG = 24;

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
