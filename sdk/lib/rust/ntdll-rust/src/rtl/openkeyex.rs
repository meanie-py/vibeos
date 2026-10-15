//! `NtOpenKeyEx` (Windows 7): `NtOpenKey` plus open options. Our kernel has no
//! NtOpenKeyEx system service, so it is built in ntdll over NtOpenKey, the way Wine's
//! ntdll treats the options it understands. It was arm64/amd64-only while it lived
//! in the WOW64 shim; every arch needs it (Steam imports it on i386).

#![allow(non_snake_case)]

use ntdll_sys::nt::*;

const REG_OPTION_OPEN_LINK_: ULONG = 0x0000_0008;
const OBJ_OPENLINK_: ULONG = 0x0000_0100;

/// REG_OPTION_OPEN_LINK opens a symbolic-link key itself rather than its target, which
/// is what OBJ_OPENLINK asks NtOpenKey for. The other option Windows accepts here,
/// REG_OPTION_BACKUP_RESTORE, needs SeBackupPrivilege semantics our configuration
/// manager does not implement; the open proceeds as a plain open, as in Wine.
#[no_mangle]
pub unsafe extern "system" fn NtOpenKeyEx(
    KeyHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    OpenOptions: ULONG,
) -> NTSTATUS {
    if OpenOptions & REG_OPTION_OPEN_LINK_ != 0 && !ObjectAttributes.is_null() {
        let mut attr = *ObjectAttributes;
        attr.Attributes |= OBJ_OPENLINK_;
        return NtOpenKey(KeyHandle, DesiredAccess, &mut attr);
    }
    NtOpenKey(KeyHandle, DesiredAccess, ObjectAttributes)
}
