//! security.c — window-station / desktop security descriptors + per-session
//! access grants. On the boot/logon path: wlx.c builds the winsta/desktop
//! SDs at startup and sas.c calls AllowAccessOnSession at logon, so this is
//! runtime-exercised by smoke (unlike shutdown/screensaver).
//!
//! Translated by rustconv (Opus 4.8), then hand-reconciled: dropped the
//! bogus `use ntdll_sys::nt::*`; added `#[no_mangle]` to all 8 exports
//! (rustconv omitted it → the C callers in wlx.c/sas.c wouldn't link);
//! RtlGetProcessHeap() and the undocumented SetWindowStationUser() provided
//! via preamble; SIZE_T casts use `as _`; FALSE/TRUE shadowed as BOOL;
//! `&mut NtAuthority` → addr_of_mut! (static_mut_refs).

use winlogon_sys::nt::*;
use crate::preamble::*;

// `BOOL` is i32; bindgen `FALSE`/`TRUE` are u32 — shadow them as BOOL so
// the i32 return/assign/compare sites and BOOL API args line up.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

const DESKTOP_ALL: DWORD = DESKTOP_READOBJECTS | DESKTOP_CREATEWINDOW |
    DESKTOP_CREATEMENU | DESKTOP_HOOKCONTROL | DESKTOP_JOURNALRECORD |
    DESKTOP_JOURNALPLAYBACK | DESKTOP_ENUMERATE | DESKTOP_WRITEOBJECTS |
    DESKTOP_SWITCHDESKTOP | STANDARD_RIGHTS_REQUIRED;

const DESKTOP_ADMINS_LIMITED: DWORD = DESKTOP_WRITEOBJECTS | DESKTOP_READOBJECTS |
    DESKTOP_CREATEWINDOW | DESKTOP_CREATEMENU | DESKTOP_ENUMERATE;

const DESKTOP_INTERACTIVE_LIMITED: DWORD = STANDARD_RIGHTS_READ | DESKTOP_ENUMERATE |
    DESKTOP_READOBJECTS | DESKTOP_CREATEWINDOW;

const DESKTOP_WINLOGON_ADMINS_LIMITED: DWORD = STANDARD_RIGHTS_REQUIRED | DESKTOP_ENUMERATE;

const WINSTA_ALL: DWORD = WINSTA_ENUMDESKTOPS | WINSTA_READATTRIBUTES |
    WINSTA_ACCESSCLIPBOARD | WINSTA_CREATEDESKTOP |
    WINSTA_WRITEATTRIBUTES | WINSTA_ACCESSGLOBALATOMS |
    WINSTA_EXITWINDOWS | WINSTA_ENUMERATE | WINSTA_READSCREEN |
    STANDARD_RIGHTS_REQUIRED;

const WINSTA_ADMINS_LIMITED: DWORD = WINSTA_READATTRIBUTES | WINSTA_ENUMERATE;

const GENERIC_ACCESS: DWORD = GENERIC_READ | GENERIC_WRITE |
    GENERIC_EXECUTE | GENERIC_ALL;

static mut NtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 5],
};

#[no_mangle]
pub unsafe extern "C" fn ConvertToSelfRelative(
    AbsoluteSd: PSECURITY_DESCRIPTOR,
) -> PSECURITY_DESCRIPTOR {
    let RelativeSd: PSECURITY_DESCRIPTOR;
    let mut DescriptorLength: DWORD = 0;

    /* Determine the size for allocating our buffer */
    if MakeSelfRelativeSD(AbsoluteSd, core::ptr::null_mut(), &mut DescriptorLength) != 0 ||
        (GetLastError() != ERROR_INSUFFICIENT_BUFFER)
    {
        return core::ptr::null_mut();
    }

    /* Allocate the buffer now */
    RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(),
                                 HEAP_ZERO_MEMORY,
                                 DescriptorLength as _);
    if RelativeSd.is_null() {
        return core::ptr::null_mut();
    }

    /* Convert the security descriptor now */
    if MakeSelfRelativeSD(AbsoluteSd, RelativeSd, &mut DescriptorLength) == 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        return core::ptr::null_mut();
    }

    RelativeSd
}

#[no_mangle]
pub unsafe extern "C" fn CreateWinstaSecurity(
    WinstaSd: *mut PSECURITY_DESCRIPTOR,
) -> BOOL {
    let mut Success: BOOL = FALSE;
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut WinlogonSid: PSID = core::ptr::null_mut();
    let mut AdminsSid: PSID = core::ptr::null_mut();
    let mut NetworkServiceSid: PSID = core::ptr::null_mut();
    let DaclSize: DWORD;
    let mut Dacl: PACL = core::ptr::null_mut();

    /* Create the Winlogon SID */
    if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                1,
                                SECURITY_LOCAL_SYSTEM_RID,
                                0, 0, 0, 0, 0, 0, 0,
                                &mut WinlogonSid) == 0
    {
        return FALSE;
    }

    'quit: loop {
        /* Create the admins SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    2,
                                    SECURITY_BUILTIN_DOMAIN_RID,
                                    DOMAIN_ALIAS_RID_ADMINS,
                                    0, 0, 0, 0, 0, 0,
                                    &mut AdminsSid) == 0
        {
            break 'quit;
        }

        /* HACK: Create the network service SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    1,
                                    SECURITY_NETWORK_SERVICE_RID,
                                    0, 0, 0, 0, 0, 0, 0,
                                    &mut NetworkServiceSid) == 0
        {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as DWORD +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(NetworkServiceSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(NetworkServiceSid);

        /* Allocate the DACL now */
        Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY,
                               DaclSize as _) as PACL;
        if Dacl.is_null() {
            break 'quit;
        }

        /* Initialize it */
        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            break 'quit;
        }

        /* First ACE -- give full winsta access to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ALL,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Second ACE -- give full generic access to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 GENERIC_ACCESS,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Third ACE -- give limited winsta access to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* Fourth ACE -- give limited desktop access to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 DESKTOP_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* HACK: Fifth ACE -- give full access to network services */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ALL,
                                 NetworkServiceSid) == 0
        {
            break 'quit;
        }

        /* HACK: Sixth ACE -- give full generic access to network services */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 GENERIC_ACCESS,
                                 NetworkServiceSid) == 0
        {
            break 'quit;
        }

        /* Initialize the security descriptor */
        if InitializeSecurityDescriptor(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION) == 0 {
            break 'quit;
        }

        /* Set the DACL to the descriptor */
        if SetSecurityDescriptorDacl(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, TRUE, Dacl, FALSE) == 0 {
            break 'quit;
        }

        /* Convert it to self-relative format */
        RelativeSd = ConvertToSelfRelative(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR);
        if RelativeSd.is_null() {
            break 'quit;
        }

        /* Give the descriptor to the caller */
        *WinstaSd = RelativeSd;
        Success = TRUE;

        break 'quit;
    }

    if !WinlogonSid.is_null() {
        FreeSid(WinlogonSid);
    }

    if !AdminsSid.is_null() {
        FreeSid(AdminsSid);
    }

    /* HACK */
    if !NetworkServiceSid.is_null() {
        FreeSid(NetworkServiceSid);
    }
    /* END HACK */

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if Success == FALSE {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        }
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn CreateApplicationDesktopSecurity(
    ApplicationDesktopSd: *mut PSECURITY_DESCRIPTOR,
) -> BOOL {
    let mut Success: BOOL = FALSE;
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut WinlogonSid: PSID = core::ptr::null_mut();
    let mut AdminsSid: PSID = core::ptr::null_mut();
    let mut NetworkServiceSid: PSID = core::ptr::null_mut();
    let DaclSize: DWORD;
    let mut Dacl: PACL = core::ptr::null_mut();

    /* Create the Winlogon SID */
    if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                1,
                                SECURITY_LOCAL_SYSTEM_RID,
                                0, 0, 0, 0, 0, 0, 0,
                                &mut WinlogonSid) == 0
    {
        return FALSE;
    }

    'quit: loop {
        /* Create the admins SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    2,
                                    SECURITY_BUILTIN_DOMAIN_RID,
                                    DOMAIN_ALIAS_RID_ADMINS,
                                    0, 0, 0, 0, 0, 0,
                                    &mut AdminsSid) == 0
        {
            break 'quit;
        }

        /* HACK: Create the network service SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    1,
                                    SECURITY_NETWORK_SERVICE_RID,
                                    0, 0, 0, 0, 0, 0, 0,
                                    &mut NetworkServiceSid) == 0
        {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as DWORD +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(NetworkServiceSid);

        /* Allocate the DACL now */
        Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY,
                               DaclSize as _) as PACL;
        if Dacl.is_null() {
            break 'quit;
        }

        /* Initialize it */
        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            break 'quit;
        }

        /* First ACE -- Give full desktop power to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Second ACE -- Give limited desktop power to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* HACK: Third ACE -- Give full desktop power to network services */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 NetworkServiceSid) == 0
        {
            break 'quit;
        }

        /* Initialize the security descriptor */
        if InitializeSecurityDescriptor(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION) == 0 {
            break 'quit;
        }

        /* Set the DACL to the descriptor */
        if SetSecurityDescriptorDacl(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, TRUE, Dacl, FALSE) == 0 {
            break 'quit;
        }

        /* Conver it to self-relative format */
        RelativeSd = ConvertToSelfRelative(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR);
        if RelativeSd.is_null() {
            break 'quit;
        }

        /* Give the descriptor to the caller */
        *ApplicationDesktopSd = RelativeSd;
        Success = TRUE;

        break 'quit;
    }

    if !WinlogonSid.is_null() {
        FreeSid(WinlogonSid);
    }

    if !AdminsSid.is_null() {
        FreeSid(AdminsSid);
    }

    /* HACK */
    if !NetworkServiceSid.is_null() {
        FreeSid(NetworkServiceSid);
    }
    /* END HACK */

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if Success == FALSE {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        }
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn CreateWinlogonDesktopSecurity(
    WinlogonDesktopSd: *mut PSECURITY_DESCRIPTOR,
) -> BOOL {
    let mut Success: BOOL = FALSE;
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut WinlogonSid: PSID = core::ptr::null_mut();
    let mut AdminsSid: PSID = core::ptr::null_mut();
    let DaclSize: DWORD;
    let mut Dacl: PACL = core::ptr::null_mut();

    /* Create the Winlogon SID */
    if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                1,
                                SECURITY_LOCAL_SYSTEM_RID,
                                0, 0, 0, 0, 0, 0, 0,
                                &mut WinlogonSid) == 0
    {
        return FALSE;
    }

    'quit: loop {
        /* Create the admins SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    2,
                                    SECURITY_BUILTIN_DOMAIN_RID,
                                    DOMAIN_ALIAS_RID_ADMINS,
                                    0, 0, 0, 0, 0, 0,
                                    &mut AdminsSid) == 0
        {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as DWORD +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid);

        /* Allocate the DACL now */
        Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY,
                               DaclSize as _) as PACL;
        if Dacl.is_null() {
            break 'quit;
        }

        /* Initialize it */
        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            break 'quit;
        }

        /* First ACE -- Give full desktop access to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Second ACE -- Give limited desktop access to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_WINLOGON_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* Initialize the security descriptor */
        if InitializeSecurityDescriptor(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION) == 0 {
            break 'quit;
        }

        /* Set the DACL to the descriptor */
        if SetSecurityDescriptorDacl(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, TRUE, Dacl, FALSE) == 0 {
            break 'quit;
        }

        /* Conver it to self-relative format */
        RelativeSd = ConvertToSelfRelative(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR);
        if RelativeSd.is_null() {
            break 'quit;
        }

        /* Give the descriptor to the caller */
        *WinlogonDesktopSd = RelativeSd;
        Success = TRUE;

        break 'quit;
    }

    if !WinlogonSid.is_null() {
        FreeSid(WinlogonSid);
    }

    if !AdminsSid.is_null() {
        FreeSid(AdminsSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if Success == FALSE {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        }
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn CreateScreenSaverSecurity(
    ScreenSaverDesktopSd: *mut PSECURITY_DESCRIPTOR,
) -> BOOL {
    let mut Success: BOOL = FALSE;
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut WinlogonSid: PSID = core::ptr::null_mut();
    let mut AdminsSid: PSID = core::ptr::null_mut();
    let mut InteractiveSid: PSID = core::ptr::null_mut();
    let DaclSize: DWORD;
    let mut Dacl: PACL = core::ptr::null_mut();

    /* Create the Winlogon SID */
    if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                1,
                                SECURITY_LOCAL_SYSTEM_RID,
                                0, 0, 0, 0, 0, 0, 0,
                                &mut WinlogonSid) == 0
    {
        return FALSE;
    }

    'quit: loop {
        /* Create the admins SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    2,
                                    SECURITY_BUILTIN_DOMAIN_RID,
                                    DOMAIN_ALIAS_RID_ADMINS,
                                    0, 0, 0, 0, 0, 0,
                                    &mut AdminsSid) == 0
        {
            break 'quit;
        }

        /* Create the interactive logon SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    1,
                                    SECURITY_INTERACTIVE_RID,
                                    0, 0, 0, 0, 0, 0, 0,
                                    &mut InteractiveSid) == 0
        {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as DWORD +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(InteractiveSid);

        /* Allocate the DACL now */
        Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY,
                               DaclSize as _) as PACL;
        if Dacl.is_null() {
            break 'quit;
        }

        /* Initialize it */
        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            break 'quit;
        }

        /* First ACE -- Give full desktop access to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Second ACE -- Give limited desktop access to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 DESKTOP_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* Third ACE -- Give full desktop access to interactive logon users */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 DESKTOP_INTERACTIVE_LIMITED,
                                 InteractiveSid) == 0
        {
            break 'quit;
        }

        /* Initialize the security descriptor */
        if InitializeSecurityDescriptor(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION) == 0 {
            break 'quit;
        }

        /* Set the DACL to the descriptor */
        if SetSecurityDescriptorDacl(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, TRUE, Dacl, FALSE) == 0 {
            break 'quit;
        }

        /* Conver it to self-relative format */
        RelativeSd = ConvertToSelfRelative(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR);
        if RelativeSd.is_null() {
            break 'quit;
        }

        /* Give the descriptor to the caller */
        *ScreenSaverDesktopSd = RelativeSd;
        Success = TRUE;

        break 'quit;
    }

    if !WinlogonSid.is_null() {
        FreeSid(WinlogonSid);
    }

    if !AdminsSid.is_null() {
        FreeSid(AdminsSid);
    }

    if !InteractiveSid.is_null() {
        FreeSid(InteractiveSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if Success == FALSE {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        }
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn AllowWinstaAccessToUser(
    WinSta: HWINSTA,
    LogonSid: PSID,
) -> BOOL {
    let mut Success: BOOL = FALSE;
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut WinlogonSid: PSID = core::ptr::null_mut();
    let mut AdminsSid: PSID = core::ptr::null_mut();
    let mut InteractiveSid: PSID = core::ptr::null_mut();
    let mut NetworkServiceSid: PSID = core::ptr::null_mut();
    let SecurityInformation: SECURITY_INFORMATION;
    let DaclSize: DWORD;
    let mut Dacl: PACL = core::ptr::null_mut();

    /* Create the Winlogon SID */
    if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                1,
                                SECURITY_LOCAL_SYSTEM_RID,
                                0, 0, 0, 0, 0, 0, 0,
                                &mut WinlogonSid) == 0
    {
        return FALSE;
    }

    'quit: loop {
        /* Create the admins SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    2,
                                    SECURITY_BUILTIN_DOMAIN_RID,
                                    DOMAIN_ALIAS_RID_ADMINS,
                                    0, 0, 0, 0, 0, 0,
                                    &mut AdminsSid) == 0
        {
            break 'quit;
        }

        /* Create the interactive logon SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    1,
                                    SECURITY_INTERACTIVE_RID,
                                    0, 0, 0, 0, 0, 0, 0,
                                    &mut InteractiveSid) == 0
        {
            break 'quit;
        }

        /* HACK: Create the network service SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    1,
                                    SECURITY_NETWORK_SERVICE_RID,
                                    0, 0, 0, 0, 0, 0, 0,
                                    &mut NetworkServiceSid) == 0
        {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as DWORD +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(InteractiveSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(InteractiveSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(LogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(LogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(NetworkServiceSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(NetworkServiceSid);

        /* Allocate the DACL now */
        Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY,
                               DaclSize as _) as PACL;
        if Dacl.is_null() {
            break 'quit;
        }

        /* Initialize it */
        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            break 'quit;
        }

        /* First ACE -- Give full winsta access to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ALL,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Second ACE -- Give generic access to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 GENERIC_ACCESS,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Third ACE -- Give limited winsta access to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* Fourth ACE -- Give limited desktop access to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 DESKTOP_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* Fifth ACE -- Give full winsta access to interactive logon users */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ALL,
                                 InteractiveSid) == 0
        {
            break 'quit;
        }

        /* Sixth ACE -- Give generic access to interactive logon users */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 GENERIC_ACCESS,
                                 InteractiveSid) == 0
        {
            break 'quit;
        }

        /* Seventh ACE -- Give full winsta access to logon user */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ALL,
                                 LogonSid) == 0
        {
            break 'quit;
        }

        /* Eighth ACE -- Give generic access to logon user */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 GENERIC_ACCESS,
                                 LogonSid) == 0
        {
            break 'quit;
        }

        /* HACK : Ninenth ACE -- Give full winsta access to network services */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 NO_PROPAGATE_INHERIT_ACE,
                                 WINSTA_ALL,
                                 NetworkServiceSid) == 0
        {
            break 'quit;
        }

        /* HACK: Tenth ACE -- Give generic access to network services */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                                 GENERIC_ACCESS,
                                 NetworkServiceSid) == 0
        {
            break 'quit;
        }

        /* Initialize the security descriptor */
        if InitializeSecurityDescriptor(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION) == 0 {
            break 'quit;
        }

        /* Set the DACL to descriptor */
        if SetSecurityDescriptorDacl(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, TRUE, Dacl, FALSE) == 0 {
            break 'quit;
        }

        /* Convert it to self-relative format */
        RelativeSd = ConvertToSelfRelative(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR);
        if RelativeSd.is_null() {
            break 'quit;
        }

        /* Set new winsta security based on this descriptor */
        SecurityInformation = DACL_SECURITY_INFORMATION;
        if SetUserObjectSecurity(WinSta as HANDLE,
                                 &SecurityInformation as *const _ as *mut SECURITY_INFORMATION,
                                 RelativeSd) == 0
        {
            break 'quit;
        }

        Success = TRUE;

        break 'quit;
    }

    if !WinlogonSid.is_null() {
        FreeSid(WinlogonSid);
    }

    if !AdminsSid.is_null() {
        FreeSid(AdminsSid);
    }

    if !InteractiveSid.is_null() {
        FreeSid(InteractiveSid);
    }

    /* HACK */
    if !NetworkServiceSid.is_null() {
        FreeSid(NetworkServiceSid);
    }
    /* END HACK */

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !RelativeSd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn AllowDesktopAccessToUser(
    Desktop: HDESK,
    LogonSid: PSID,
) -> BOOL {
    let mut Success: BOOL = FALSE;
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut WinlogonSid: PSID = core::ptr::null_mut();
    let mut AdminsSid: PSID = core::ptr::null_mut();
    let mut InteractiveSid: PSID = core::ptr::null_mut();
    let mut NetworkServiceSid: PSID = core::ptr::null_mut();
    let SecurityInformation: SECURITY_INFORMATION;
    let DaclSize: DWORD;
    let mut Dacl: PACL = core::ptr::null_mut();

    /* Create the Winlogon SID */
    if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                1,
                                SECURITY_LOCAL_SYSTEM_RID,
                                0, 0, 0, 0, 0, 0, 0,
                                &mut WinlogonSid) == 0
    {
        return FALSE;
    }

    'quit: loop {
        /* Create the admins SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    2,
                                    SECURITY_BUILTIN_DOMAIN_RID,
                                    DOMAIN_ALIAS_RID_ADMINS,
                                    0, 0, 0, 0, 0, 0,
                                    &mut AdminsSid) == 0
        {
            break 'quit;
        }

        /* Create the interactive logon SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    1,
                                    SECURITY_INTERACTIVE_RID,
                                    0, 0, 0, 0, 0, 0, 0,
                                    &mut InteractiveSid) == 0
        {
            break 'quit;
        }

        /* HACK: Create the network service SID */
        if AllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                    1,
                                    SECURITY_NETWORK_SERVICE_RID,
                                    0, 0, 0, 0, 0, 0, 0,
                                    &mut NetworkServiceSid) == 0
        {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as DWORD +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(WinlogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(AdminsSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(InteractiveSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(LogonSid) +
            core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD - core::mem::size_of::<DWORD>() as DWORD + GetLengthSid(NetworkServiceSid);

        /* Allocate the DACL now */
        Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY,
                               DaclSize as _) as PACL;
        if Dacl.is_null() {
            break 'quit;
        }

        /* Initialize it */
        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            break 'quit;
        }

        /* First ACE -- Give full desktop access to Winlogon */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 WinlogonSid) == 0
        {
            break 'quit;
        }

        /* Second ACE -- Give limited desktop access to admins */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ADMINS_LIMITED,
                                 AdminsSid) == 0
        {
            break 'quit;
        }

        /* Third ACE -- Give full desktop access to interactive logon users */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 InteractiveSid) == 0
        {
            break 'quit;
        }

        /* Fourth ACE -- Give full desktop access to logon user */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 LogonSid) == 0
        {
            break 'quit;
        }

        /* HACK: Fifth ACE -- Give full desktop to network services */
        if AddAccessAllowedAceEx(Dacl,
                                 ACL_REVISION,
                                 0,
                                 DESKTOP_ALL,
                                 NetworkServiceSid) == 0
        {
            break 'quit;
        }

        /* Initialize the security descriptor */
        if InitializeSecurityDescriptor(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION) == 0 {
            break 'quit;
        }

        /* Set the DACL to the descriptor */
        if SetSecurityDescriptorDacl(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR, TRUE, Dacl, FALSE) == 0 {
            break 'quit;
        }

        /* Conver it to self-relative format */
        RelativeSd = ConvertToSelfRelative(&mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR);
        if RelativeSd.is_null() {
            break 'quit;
        }

        /* Assign new security to desktop based on this descriptor */
        SecurityInformation = DACL_SECURITY_INFORMATION;
        if SetUserObjectSecurity(Desktop as HANDLE,
                                 &SecurityInformation as *const _ as *mut SECURITY_INFORMATION,
                                 RelativeSd) == 0
        {
            break 'quit;
        }

        Success = TRUE;

        break 'quit;
    }

    if !WinlogonSid.is_null() {
        FreeSid(WinlogonSid);
    }

    if !AdminsSid.is_null() {
        FreeSid(AdminsSid);
    }

    if !InteractiveSid.is_null() {
        FreeSid(InteractiveSid);
    }

    /* HACK */
    if !NetworkServiceSid.is_null() {
        FreeSid(NetworkServiceSid);
    }
    /* END HACK */

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !RelativeSd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn AllowAccessOnSession(
    Session: PWLSESSION,
) -> BOOL {
    let mut Success: BOOL = FALSE;
    let mut Index: DWORD;
    let SidLength: DWORD;
    let mut GroupsLength: DWORD = 0;
    let mut TokenGroup: PTOKEN_GROUPS = core::ptr::null_mut();
    let mut LogonSid: PSID = core::ptr::null_mut();

    /* Get required buffer size and allocate the TOKEN_GROUPS buffer */
    if GetTokenInformation((*Session).UserToken,
                           TokenGroups,
                           TokenGroup as PVOID,
                           0,
                           &mut GroupsLength) == 0
    {
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            return FALSE;
        }

        TokenGroup = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, GroupsLength as _) as PTOKEN_GROUPS;
        if TokenGroup.is_null() {
            return FALSE;
        }
    }

    'quit: loop {
        /* Get the token group information from the access token */
        if GetTokenInformation((*Session).UserToken,
                               TokenGroups,
                               TokenGroup as PVOID,
                               GroupsLength,
                               &mut GroupsLength) == 0
        {
            break 'quit;
        }

        /* Loop through the groups to find the logon SID */
        Index = 0;
        while Index < (*TokenGroup).GroupCount {
            let group = (*TokenGroup).Groups.as_ptr().add(Index as usize);
            if ((*group).Attributes & SE_GROUP_LOGON_ID) == SE_GROUP_LOGON_ID {
                LogonSid = (*group).Sid;
                break;
            }
            Index += 1;
        }

        /* Allow window station access to this user within this session */
        if AllowWinstaAccessToUser((*Session).InteractiveWindowStation, LogonSid) == 0 {
            break 'quit;
        }

        /* Allow application desktop access to this user within this session */
        if AllowDesktopAccessToUser((*Session).ApplicationDesktop, LogonSid) == 0 {
            break 'quit;
        }

        /* Get the length of this logon SID */
        SidLength = GetLengthSid(LogonSid);

        /* Assign the window station to this logged in user */
        if SetWindowStationUser((*Session).InteractiveWindowStation,
                                &mut (*Session).LogonId,
                                LogonSid,
                                SidLength) == 0
        {
            break 'quit;
        }

        Success = TRUE;

        break 'quit;
    }

    if !TokenGroup.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, TokenGroup as PVOID);
    }

    Success
}