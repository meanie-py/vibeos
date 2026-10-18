//! vibeos additions to install.c's LiveCD path, kept beside the translation rather than in
//! it: install.c has none of this, and verify_translation_literals reads a companion's
//! literals as the port's own, not as text the translation invented.
use crate::prelude::*;

/// vibeos: `HKLM\SYSTEM\Setup\VibeosRegistrationDone = 1` once RegistrationPhase2 and the type
/// libraries are in. winlogon starts the shell alongside this setup, so the shell's startup
/// programs run while COM classes are still being registered: modrun's first case saw
/// CLSID_ProgressDialog unregistered (REGDB_E_CLASSNOTREG) where the next process found it.
/// modrun waits for this value before its first case; nothing else reads it.
pub(crate) unsafe fn mark_registration_done() {
    let mut key: HKEY = core::ptr::null_mut();
    if RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SYSTEM\\Setup") as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        &mut key,
        core::ptr::null_mut(),
    ) as u32
        != ERROR_SUCCESS
    {
        DPRINT1!("[SYSSETUP] InstallLiveCD: cannot open SYSTEM\\Setup for the registration marker\n");
        return;
    }
    let done: DWORD = 1;
    RegSetValueExW(
        key,
        w!("VibeosRegistrationDone") as LPCWSTR,
        0,
        REG_DWORD,
        &done as *const DWORD as *const BYTE,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    RegCloseKey(key);
}
