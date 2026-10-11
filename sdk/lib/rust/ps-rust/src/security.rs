use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut PsProcessType: *mut _OBJECT_TYPE;
    static mut PsThreadType: *mut _OBJECT_TYPE;
    static mut SeTokenObjectType: *mut _OBJECT_TYPE;
    static SeAssignPrimaryTokenPrivilege: LUID;
}

extern "system" {
    static mut PspBootAccessToken: PTOKEN;
    fn SeAssignPrimaryToken(Process: PEPROCESS, Token: PTOKEN);
    fn ExGetPreviousMode() -> KPROCESSOR_MODE;
}
extern "C" {
    // PspLock/Unlock functions: imported via ddk::* (bindgen extern "C")
    fn PspClearCrossThreadFlag(Thread: PETHREAD, Flag: ULONG) -> LONG;
    fn PspSetCrossThreadFlag(Thread: PETHREAD, Flag: ULONG);
}

const TAG_PS_IMPERSONATION: ULONG = 0x6D497350;
const CT_ACTIVE_IMPERSONATION_INFO_BIT: ULONG = 0x00000008;
const PROCESS_VM_OPERATION: ACCESS_MASK = 0x0008;
const PROCESS_VM_READ: ACCESS_MASK = 0x0010;
const PROCESS_VM_WRITE: ACCESS_MASK = 0x0020;
const PROCESS_QUERY_INFORMATION: ACCESS_MASK = 0x0400;
const PROCESS_TERMINATE: ACCESS_MASK = 0x0001;
const PROCESS_CREATE_THREAD: ACCESS_MASK = 0x0002;
const PROCESS_DUP_HANDLE: ACCESS_MASK = 0x0040;
const PROCESS_CREATE_PROCESS: ACCESS_MASK = 0x0080;
const PROCESS_SET_INFORMATION: ACCESS_MASK = 0x0200;
const PROCESS_SET_QUOTA: ACCESS_MASK = 0x0100;
const STANDARD_RIGHTS_ALL: ACCESS_MASK = 0x001F0000;
const MAXIMUM_ALLOWED: ACCESS_MASK = 0x02000000;
const TOKEN_ASSIGN_PRIMARY: ACCESS_MASK = 0x0001;
const TOKEN_IMPERSONATE: ACCESS_MASK = 0x0004;
const THREAD_DIRECT_IMPERSONATION: ACCESS_MASK = 0x0200;
const THREAD_IMPERSONATE: ACCESS_MASK = 0x0100;
const JOB_OBJECT_SECURITY_NO_ADMIN: ULONG = 0x00000001;
const JOB_OBJECT_SECURITY_RESTRICTED_TOKEN: ULONG = 0x00000002;

type PEJOB = *mut _EJOB;
type SECURITY_CLIENT_CONTEXT = _SECURITY_CLIENT_CONTEXT;
type SECURITY_QUALITY_OF_SERVICE = _SECURITY_QUALITY_OF_SERVICE;
type SECURITY_SUBJECT_CONTEXT = _SECURITY_SUBJECT_CONTEXT;
type SE_IMPERSONATION_STATE = _SE_IMPERSONATION_STATE;

const TokenPrimary: ULONG = 1;
const TokenImpersonation: ULONG = 2;
const SecurityAnonymous: ULONG = 0;
const SecurityIdentification: ULONG = 1;
const KernelMode: KPROCESSOR_MODE = 0;

#[no_mangle]
pub unsafe extern "system" fn PspDeleteProcessSecurity(Process: PEPROCESS) {
    let Process = opaque_ptr(Process);
    if (*(Process as *mut _EPROCESS)).Token.__bindgen_anon_1.Object != core::ptr::null_mut() {
        SeDeassignPrimaryToken(Process);
        (*(Process as *mut _EPROCESS)).Token.__bindgen_anon_1.Object = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn PspDeleteThreadSecurity(Thread: PETHREAD) {
    let Thread = opaque_ptr(Thread);
    let ImpersonationInfo = (*(Thread as *mut _ETHREAD)).ImpersonationInfo;

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
        ObDereferenceObject((*ImpersonationInfo).Token as PVOID);
    }

    if !ImpersonationInfo.is_null() {
        ExFreePool(ImpersonationInfo as PVOID);
        PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
        (*(Thread as *mut _ETHREAD)).ImpersonationInfo = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn PspInitializeProcessSecurity(
    Process: PEPROCESS,
    Parent: PEPROCESS,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Parent = opaque_ptr(Parent);
    let mut Status = STATUS_SUCCESS;
    let mut NewToken: PTOKEN = core::ptr::null_mut();
    let ParentToken: PACCESS_TOKEN;

    if !Parent.is_null() {
        ParentToken = PsReferencePrimaryToken(Parent);

        Status = SeSubProcessToken(
            ParentToken as PTOKEN,
            &NewToken as *const _ as *mut PTOKEN,
            1u8,
            MmGetSessionId(Process),
        );

        ObFastDereferenceObject(
            core::ptr::addr_of_mut!((*(Parent as *mut _EPROCESS)).Token),
            ParentToken,
        );

        if Status >= 0 {
            ObInitializeFastReference(
                core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Token),
                NewToken as PVOID,
            );
        }
    } else {
        ObInitializeFastReference(
            core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Token),
            core::ptr::null_mut(),
        );
        SeAssignPrimaryToken(Process, PspBootAccessToken);
    }

    Status
}

unsafe fn PspWriteTebImpersonationInfo(
    Thread: PETHREAD,
    CurrentThread: PETHREAD,
) -> NTSTATUS {
    let Process: PEPROCESS;
    let Teb: PTEB;
    let mut Attached = 0u8;
    let IsImpersonating: BOOLEAN;
    let mut ApcState: KAPC_STATE = core::mem::zeroed();

    Process = (*(Thread as *mut _ETHREAD)).ThreadsProcess as PEPROCESS;
    Teb = (*(Thread as *mut _ETHREAD)).Tcb.Teb;
    if !Teb.is_null() {
        if (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_1.ApcState.Process
            != core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb) as PKPROCESS
        {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb) as PKPROCESS,
                &mut ApcState,
            );
            Attached = 1u8;
        }

        if Thread == CurrentThread
            || ExAcquireRundownProtection(core::ptr::addr_of_mut!(
                (*(Thread as *mut _ETHREAD)).RundownProtect
            )) != 0
        {
            IsImpersonating = (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() as BOOLEAN;
            if IsImpersonating != 0 {
                (*Teb).ImpersonationLocale = -1i32 as u32;
                (*Teb).IsImpersonating = 1;
            } else {
                (*Teb).ImpersonationLocale = 0;
                (*Teb).IsImpersonating = 0;
            }
        }

        if Thread != CurrentThread {
            ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                (*(Thread as *mut _ETHREAD)).RundownProtect
            ));
        }

        if Attached != 0 {
            KeUnstackDetachProcess(&mut ApcState);
        }
    }

    STATUS_SUCCESS
}

unsafe fn PspAssignPrimaryToken(
    Process: PEPROCESS,
    Token: HANDLE,
    AccessToken: PACCESS_TOKEN,
) -> NTSTATUS {
    let mut NewToken = AccessToken;
    let mut OldToken: PACCESS_TOKEN = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if AccessToken.is_null() {
        Status = ObReferenceObjectByHandle(
            Token,
            TOKEN_ASSIGN_PRIMARY,
            SeTokenObjectType as POBJECT_TYPE,
            ExGetPreviousMode(),
            &mut NewToken as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
    }

    Status = SeExchangePrimaryToken(Process, NewToken, &OldToken as *const _ as *mut PACCESS_TOKEN);

    PspLockProcessSecurityExclusive(Process);
    PspUnlockProcessSecurityExclusive(Process);

    if Status >= 0 {
        ObDereferenceObject(OldToken as PVOID);
    }
    if AccessToken.is_null() {
        ObDereferenceObject(NewToken as PVOID);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PspSetPrimaryToken(
    Process: PEPROCESS,
    TokenHandle: HANDLE,
    Token: PACCESS_TOKEN,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let TokenHandle = opaque_ptr(TokenHandle);
    let PreviousMode = ExGetPreviousMode();
    let mut IsChildOrSibling: BOOLEAN = 0;
    let mut NewToken = Token;
    let mut Status: NTSTATUS;
    let mut AccessStatus: NTSTATUS = 0;
    let mut Result: BOOLEAN;
    let mut SdAllocated: BOOLEAN = 0;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SubjectContext: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();

    if Token.is_null() {
        Status = ObReferenceObjectByHandle(
            TokenHandle,
            TOKEN_ASSIGN_PRIMARY,
            SeTokenObjectType as POBJECT_TYPE,
            PreviousMode,
            &mut NewToken as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
    }

    Status = SeIsTokenChild(NewToken as PTOKEN, &mut IsChildOrSibling);
    if Status < 0 {
        if Token.is_null() {
            ObDereferenceObject(NewToken as PVOID);
        }
        return Status;
    }
    if IsChildOrSibling == 0 {
        Status = SeIsTokenSibling(NewToken as PTOKEN, &mut IsChildOrSibling);
        if Status < 0 {
            if Token.is_null() {
                ObDereferenceObject(NewToken as PVOID);
            }
            return Status;
        }
    }

    if IsChildOrSibling == 0 {
        if SeSinglePrivilegeCheck(SeAssignPrimaryTokenPrivilege, PreviousMode) == 0 {
            if Token.is_null() {
                ObDereferenceObject(NewToken as PVOID);
            }
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }

    Status = PspAssignPrimaryToken(Process, core::ptr::null_mut(), NewToken);
    if Status >= 0 {
        Status = ObGetObjectSecurity(
            Process as PVOID,
            &mut SecurityDescriptor,
            &mut SdAllocated,
        );
        if Status >= 0 {
            SubjectContext.ProcessAuditId = Process as PVOID;
            SubjectContext.PrimaryToken = PsReferencePrimaryToken(Process);
            SubjectContext.ClientToken = core::ptr::null_mut();

            Result = SeAccessCheck(
                SecurityDescriptor,
                &mut SubjectContext,
                0u8,
                MAXIMUM_ALLOWED,
                0,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*(PsProcessType as *mut _OBJECT_TYPE)).TypeInfo.GenericMapping),
                PreviousMode,
                &mut (*(Process as *mut _EPROCESS)).GrantedAccess,
                &AccessStatus as *const _ as *mut NTSTATUS,
            );

            ObFastDereferenceObject(
                core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Token),
                SubjectContext.PrimaryToken,
            );
            ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);

            if Result == 0 {
                (*(Process as *mut _EPROCESS)).GrantedAccess = 0;
            }

            (*(Process as *mut _EPROCESS)).GrantedAccess |= PROCESS_VM_OPERATION
                | PROCESS_VM_READ
                | PROCESS_VM_WRITE
                | PROCESS_QUERY_INFORMATION
                | PROCESS_TERMINATE
                | PROCESS_CREATE_THREAD
                | PROCESS_DUP_HANDLE
                | PROCESS_CREATE_PROCESS
                | PROCESS_SET_INFORMATION
                | STANDARD_RIGHTS_ALL
                | PROCESS_SET_QUOTA;
        }

        if ObIsLUIDDeviceMapsEnabled() != 0 {
            ObDereferenceDeviceMap(Process);
        }
    }

    if Token.is_null() {
        ObDereferenceObject(NewToken as PVOID);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenProcessToken(
    ProcessHandle: HANDLE,
    DesiredAccess: ACCESS_MASK,
    TokenHandle: PHANDLE,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let TokenHandle = opaque_ptr(TokenHandle);
    NtOpenProcessTokenEx(ProcessHandle, DesiredAccess, 0, TokenHandle)
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenProcessTokenEx(
    ProcessHandle: HANDLE,
    DesiredAccess: ACCESS_MASK,
    HandleAttributes: ULONG,
    TokenHandle: PHANDLE,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let TokenHandle = opaque_ptr(TokenHandle);
    let mut Token: PACCESS_TOKEN = core::ptr::null_mut();
    let mut hToken: HANDLE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if PreviousMode != KernelMode {
        if let Err(s) = probe_user_buffer_write(TokenHandle as *const core::ffi::c_void, core::mem::size_of::<HANDLE>(), core::mem::align_of::<HANDLE>(), PreviousMode) {
            return s;
        }
    }

    let HandleAttributes = ObpValidateAttributes(HandleAttributes, PreviousMode);

    Status = PsOpenTokenOfProcess(ProcessHandle, &Token as *const _ as *mut PACCESS_TOKEN);
    if Status >= 0 {
        Status = ObOpenObjectByPointer(
            Token as PVOID,
            HandleAttributes,
            core::ptr::null_mut(),
            DesiredAccess,
            SeTokenObjectType as POBJECT_TYPE,
            PreviousMode,
            &mut hToken,
        );
        ObDereferenceObject(Token as PVOID);

        if Status >= 0 {
            if let Err(s) = probe_write_user(TokenHandle, hToken, PreviousMode) {
                return s;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PsReferencePrimaryToken(Process: PEPROCESS) -> PACCESS_TOKEN {
    let Process = opaque_ptr(Process);
    let mut Token: PACCESS_TOKEN;

    Token = ObFastReferenceObject(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Token));

    if Token.is_null() {
        PspLockProcessSecurityShared(Process);

        Token = ObFastReferenceObjectLocked(core::ptr::addr_of_mut!(
            (*(Process as *mut _EPROCESS)).Token
        ));

        PspUnlockProcessSecurityShared(Process);
    }

    Token
}

#[no_mangle]
pub unsafe extern "system" fn PsOpenTokenOfProcess(
    ProcessHandle: HANDLE,
    Token: *mut PACCESS_TOKEN,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let Token = opaque_ptr(Token);
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let Status: NTSTATUS;

    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        PROCESS_QUERY_INFORMATION,
        PsProcessType as POBJECT_TYPE,
        ExGetPreviousMode(),
        &Process as *const _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        *Token = PsReferencePrimaryToken(Process);
        ObDereferenceObject(Process as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PsAssignImpersonationToken(
    Thread: PETHREAD,
    TokenHandle: HANDLE,
) -> NTSTATUS {
    let Thread = opaque_ptr(Thread);
    let TokenHandle = opaque_ptr(TokenHandle);
    let mut Token: PACCESS_TOKEN = core::ptr::null_mut();
    let ImpersonationLevel: ULONG;
    let mut Status: NTSTATUS;

    if TokenHandle.is_null() {
        PsRevertThreadToSelf(Thread);
        return STATUS_SUCCESS;
    }

    Status = ObReferenceObjectByHandle(
        TokenHandle,
        TOKEN_IMPERSONATE,
        SeTokenObjectType as POBJECT_TYPE,
        ExGetPreviousMode(),
        &Token as *const _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if SeTokenType(Token) != TokenImpersonation {
        ObDereferenceObject(Token as PVOID);
        return STATUS_BAD_TOKEN_TYPE;
    }

    ImpersonationLevel = SeTokenImpersonationLevel(Token);

    Status = PsImpersonateClient(Thread, Token, 0u8, 0u8, ImpersonationLevel);

    ObDereferenceObject(Token as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PsRevertToSelf() {
    PsRevertThreadToSelf(PsGetCurrentThread());
}

#[no_mangle]
pub unsafe extern "system" fn PsRevertThreadToSelf(Thread: PETHREAD) {
    let Thread = opaque_ptr(Thread);
    let mut Token: PACCESS_TOKEN = core::ptr::null_mut();

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
        PspLockThreadSecurityExclusive(Thread);

        if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
            PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);

            Token = (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).Token as PACCESS_TOKEN;
        }

        PspUnlockThreadSecurityExclusive(Thread);

        if !Token.is_null() {
            ObDereferenceObject(Token as PVOID);

            PspWriteTebImpersonationInfo(Thread, PsGetCurrentThread());
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PsImpersonateClient(
    Thread: PETHREAD,
    Token: PACCESS_TOKEN,
    CopyOnOpen: BOOLEAN,
    EffectiveOnly: BOOLEAN,
    ImpersonationLevel: ULONG,
) -> NTSTATUS {
    let Thread = opaque_ptr(Thread);
    let mut Impersonation: PPS_IMPERSONATION_INFORMATION;
    let OldData: PPS_IMPERSONATION_INFORMATION;
    let mut OldToken: PACCESS_TOKEN = core::ptr::null_mut();
    let mut ProcessToken: PACCESS_TOKEN = core::ptr::null_mut();
    let mut CopiedToken = 0u8;
    let mut NewToken: PACCESS_TOKEN = core::ptr::null_mut();
    let mut ImpersonationToken: PACCESS_TOKEN;
    let Job: PEJOB;
    let mut Status: NTSTATUS;
    let mut ImpersonationLevel = ImpersonationLevel;

    if Token.is_null() {
        if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
            PspLockThreadSecurityExclusive(Thread);

            if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
                PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);

                OldToken = (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).Token as PACCESS_TOKEN;
            }

            PspUnlockThreadSecurityExclusive(Thread);
            PspWriteTebImpersonationInfo(Thread, PsGetCurrentThread());
        }
    } else {
        Impersonation = (*(Thread as *mut _ETHREAD)).ImpersonationInfo;
        if Impersonation.is_null() {
            Impersonation = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<_PS_IMPERSONATION_INFORMATION>() as SIZE_T,
                TAG_PS_IMPERSONATION,
            ) as PPS_IMPERSONATION_INFORMATION;
            if Impersonation.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            OldData = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).ImpersonationInfo)
                    as *mut PVOID,
                Impersonation as PVOID,
                core::ptr::null_mut(),
            ) as PPS_IMPERSONATION_INFORMATION;
            if !OldData.is_null() {
                ExFreePoolWithTag(Impersonation as PVOID, TAG_PS_IMPERSONATION);
                Impersonation = OldData;
            }
        }

        ImpersonationToken = Token;

        ProcessToken = PsReferencePrimaryToken((*(Thread as *mut _ETHREAD)).ThreadsProcess as PEPROCESS);
        if ProcessToken.is_null() {
            return STATUS_UNSUCCESSFUL;
        }

        if SeTokenCanImpersonate(ProcessToken as PTOKEN, Token as PTOKEN, ImpersonationLevel) == 0 {
            Status = SeCopyClientToken(
                Token,
                SecurityIdentification,
                KernelMode,
                &NewToken as *const _ as *mut PACCESS_TOKEN,
            );
            if Status < 0 {
                let process_ptr = (*(Thread as *mut _ETHREAD)).ThreadsProcess;
                ObFastDereferenceObject(
                    core::ptr::addr_of_mut!((*process_ptr).Token),
                    ProcessToken,
                );
                return Status;
            }

            CopiedToken = 1u8;
            ImpersonationLevel = SecurityIdentification;
            ImpersonationToken = NewToken;
        }

        {
            let process_ptr = (*(Thread as *mut _ETHREAD)).ThreadsProcess;
            ObFastDereferenceObject(
                core::ptr::addr_of_mut!((*process_ptr).Token),
                ProcessToken,
            );
        }

        {
            let process_ptr = (*(Thread as *mut _ETHREAD)).ThreadsProcess;
            Job = (*process_ptr).Job as PEJOB;
        }
        if !Job.is_null() {
            if ((*Job).SecurityLimitFlags & JOB_OBJECT_SECURITY_NO_ADMIN != 0)
                && SeTokenIsAdmin(ImpersonationToken) != 0
            {
                if CopiedToken != 0 {
                    ObDereferenceObject(ImpersonationToken as PVOID);
                }
                return STATUS_ACCESS_DENIED;
            }

            if ((*Job).SecurityLimitFlags & JOB_OBJECT_SECURITY_RESTRICTED_TOKEN != 0)
                && SeTokenIsRestricted(ImpersonationToken) != 0
            {
                if CopiedToken != 0 {
                    ObDereferenceObject(ImpersonationToken as PVOID);
                }
                return STATUS_ACCESS_DENIED;
            }
        }

        PspLockThreadSecurityExclusive(Thread);

        if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
            OldToken = (*Impersonation).Token as PACCESS_TOKEN;
        } else {
            PspSetCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
        }

        (*Impersonation).ImpersonationLevel = ImpersonationLevel;
        (*Impersonation).CopyOnOpen = CopyOnOpen;
        (*Impersonation).EffectiveOnly = EffectiveOnly;
        (*Impersonation).Token = ImpersonationToken;

        if CopiedToken == 0 {
            ObReferenceObject(ImpersonationToken as PVOID);
        }

        PspUnlockThreadSecurityExclusive(Thread);

        PspWriteTebImpersonationInfo(Thread, PsGetCurrentThread());
    }

    if !OldToken.is_null() {
        PsDereferenceImpersonationToken(OldToken);
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PsReferenceEffectiveToken(
    Thread: PETHREAD,
    TokenType: PTOKEN_TYPE,
    EffectiveOnly: PBOOLEAN,
    ImpersonationLevel: PSECURITY_IMPERSONATION_LEVEL,
) -> PACCESS_TOKEN {
    let Thread = opaque_ptr(Thread);
    let EffectiveOnly = opaque_ptr(EffectiveOnly);
    let Process: PEPROCESS;
    let mut Token: PACCESS_TOKEN = core::ptr::null_mut();

    Process = (*(Thread as *mut _ETHREAD)).ThreadsProcess as PEPROCESS;
    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
        PspLockProcessSecurityShared(Process);

        if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
            Token = (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).Token as PACCESS_TOKEN;
            ObReferenceObject(Token as PVOID);

            *TokenType = TokenImpersonation;
            *EffectiveOnly =
                (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).EffectiveOnly;
            *ImpersonationLevel =
                (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).ImpersonationLevel;

            PspUnlockProcessSecurityShared(Process);
            return Token;
        }

        PspUnlockProcessSecurityShared(Process);
    }

    Token = ObFastReferenceObject(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Token));

    if Token.is_null() {
        PspLockProcessSecurityShared(Process);

        Token = ObFastReferenceObjectLocked(core::ptr::addr_of_mut!(
            (*(Process as *mut _EPROCESS)).Token
        ));

        PspUnlockProcessSecurityShared(Process);
    }

    *TokenType = TokenPrimary;
    *EffectiveOnly = 0u8;
    Token
}

#[no_mangle]
pub unsafe extern "system" fn PsReferenceImpersonationToken(
    Thread: PETHREAD,
    CopyOnOpen: PBOOLEAN,
    EffectiveOnly: PBOOLEAN,
    ImpersonationLevel: PSECURITY_IMPERSONATION_LEVEL,
) -> PACCESS_TOKEN {
    let Thread = opaque_ptr(Thread);
    let CopyOnOpen = opaque_ptr(CopyOnOpen);
    let EffectiveOnly = opaque_ptr(EffectiveOnly);
    let mut Token: PACCESS_TOKEN = core::ptr::null_mut();

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() == 0 {
        return core::ptr::null_mut();
    }

    PspLockThreadSecurityShared(Thread);

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
        ObReferenceObject((*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).Token as PVOID);
        *ImpersonationLevel =
            (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).ImpersonationLevel;
        *CopyOnOpen = (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).CopyOnOpen;
        *EffectiveOnly = (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).EffectiveOnly;

        Token = (*(*(Thread as *mut _ETHREAD)).ImpersonationInfo).Token as PACCESS_TOKEN;
    }

    PspUnlockThreadSecurityShared(Thread);
    Token
}

#[no_mangle]
pub unsafe extern "system" fn PsDereferenceImpersonationToken(ImpersonationToken: PACCESS_TOKEN) {
    if !ImpersonationToken.is_null() {
        ObDereferenceObject(ImpersonationToken as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PsDereferencePrimaryToken(PrimaryToken: PACCESS_TOKEN) {
    ObDereferenceObject(PrimaryToken as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn PsDisableImpersonation(
    Thread: PETHREAD,
    ImpersonationState: PSE_IMPERSONATION_STATE,
) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    let mut Impersonation: PPS_IMPERSONATION_INFORMATION = core::ptr::null_mut();
    let OldFlags: LONG;

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
        PspLockThreadSecurityExclusive(Thread);

        OldFlags = PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);

        if OldFlags & CT_ACTIVE_IMPERSONATION_INFO_BIT as LONG != 0 {
            Impersonation = (*(Thread as *mut _ETHREAD)).ImpersonationInfo;
            (*ImpersonationState).Token = (*Impersonation).Token;
            (*ImpersonationState).CopyOnOpen = (*Impersonation).CopyOnOpen;
            (*ImpersonationState).EffectiveOnly = (*Impersonation).EffectiveOnly;
            (*ImpersonationState).Level = (*Impersonation).ImpersonationLevel;
        }

        PspUnlockThreadSecurityExclusive(Thread);

        if !Impersonation.is_null() {
            return 1u8;
        }
    }

    (*ImpersonationState).Token = core::ptr::null_mut();
    (*ImpersonationState).CopyOnOpen = 0u8;
    (*ImpersonationState).EffectiveOnly = 0u8;
    (*ImpersonationState).Level = SecurityAnonymous;
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn PsRestoreImpersonation(
    Thread: PETHREAD,
    ImpersonationState: PSE_IMPERSONATION_STATE,
) {
    let Thread = opaque_ptr(Thread);
    let mut Token: PACCESS_TOKEN = core::ptr::null_mut();
    let Impersonation: PPS_IMPERSONATION_INFORMATION;

    PspLockThreadSecurityExclusive(Thread);

    Impersonation = (*(Thread as *mut _ETHREAD)).ImpersonationInfo;

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
        Token = (*Impersonation).Token as PACCESS_TOKEN;
    }

    if !ImpersonationState.is_null() {
        (*Impersonation).ImpersonationLevel = (*ImpersonationState).Level;
        (*Impersonation).CopyOnOpen = (*ImpersonationState).CopyOnOpen;
        (*Impersonation).EffectiveOnly = (*ImpersonationState).EffectiveOnly;
        (*Impersonation).Token = (*ImpersonationState).Token;

        PspSetCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
    } else {
        PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
    }

    PspUnlockThreadSecurityExclusive(Thread);

    if !Token.is_null() {
        ObDereferenceObject(Token as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtImpersonateThread(
    ThreadHandle: HANDLE,
    ThreadToImpersonateHandle: HANDLE,
    SecurityQualityOfService: PSECURITY_QUALITY_OF_SERVICE,
) -> NTSTATUS {
    let ThreadHandle = opaque_ptr(ThreadHandle);
    let ThreadToImpersonateHandle = opaque_ptr(ThreadToImpersonateHandle);
    let mut SafeServiceQoS: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let mut ClientContext: SECURITY_CLIENT_CONTEXT = core::mem::zeroed();
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let mut ThreadToImpersonate: PETHREAD = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut SecurityQualityOfService = SecurityQualityOfService;

    if PreviousMode != KernelMode {
        match probe_read_user::<SECURITY_QUALITY_OF_SERVICE>(
            SecurityQualityOfService as _,
            PreviousMode,
        ) {
            Ok(v) => SafeServiceQoS = v,
            Err(s) => return s,
        }
        SecurityQualityOfService = &mut SafeServiceQoS;
    }

    Status = ObReferenceObjectByHandle(
        ThreadHandle,
        THREAD_DIRECT_IMPERSONATION,
        PsThreadType as POBJECT_TYPE,
        PreviousMode,
        &Thread as *const _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        Status = ObReferenceObjectByHandle(
            ThreadToImpersonateHandle,
            THREAD_IMPERSONATE,
            PsThreadType as POBJECT_TYPE,
            PreviousMode,
            &ThreadToImpersonate as *const _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status >= 0 {
            Status = SeCreateClientSecurity(
                ThreadToImpersonate,
                SecurityQualityOfService,
                0u8,
                &mut ClientContext,
            );
            if Status >= 0 {
                SeImpersonateClient(&mut ClientContext, Thread);
                if !ClientContext.ClientToken.is_null() {
                    ObDereferenceObject(ClientContext.ClientToken as PVOID);
                }
            }

            ObDereferenceObject(ThreadToImpersonate as PVOID);
        }

        ObDereferenceObject(Thread as PVOID);
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ps/security.c
//  * PURPOSE:         Process Manager: Process/Thread Security
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// PTOKEN PspBootAccessToken;
//
// VOID
// NTAPI
// SeAssignPrimaryToken(
//     IN PEPROCESS Process,
//     IN PTOKEN Token
// );
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// PspDeleteProcessSecurity(IN PEPROCESS Process)
// {
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Process: %p\n", Process);
//
//     /* Check if we have a token */
//     if (Process->Token.Object)
//     {
//         /* Deassign it */
//         SeDeassignPrimaryToken(Process);
//         Process->Token.Object = NULL;
//     }
// }
//
// VOID
// NTAPI
// PspDeleteThreadSecurity(IN PETHREAD Thread)
// {
//     PPS_IMPERSONATION_INFORMATION ImpersonationInfo = Thread->ImpersonationInfo;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Thread: %p\n", Thread);
//
//     /* Check if we have active impersonation info */
//     if (Thread->ActiveImpersonationInfo)
//     {
//         /* Dereference its token */
//         ObDereferenceObject(ImpersonationInfo->Token);
//     }
//
//     /* Check if we have impersonation info */
//     if (ImpersonationInfo)
//     {
//         /* Free it */
//         ExFreePool(ImpersonationInfo);
//         PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
//         Thread->ImpersonationInfo = NULL;
//     }
// }
//
// NTSTATUS
// NTAPI
// PspInitializeProcessSecurity(IN PEPROCESS Process,
//                              IN PEPROCESS Parent OPTIONAL)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PTOKEN NewToken, ParentToken;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Process: %p\n", Process);
//
//     /* If we have a parent, then duplicate the Token */
//     if (Parent)
//     {
//         /* Get the Parent Token */
//         ParentToken = PsReferencePrimaryToken(Parent);
//
//         /* Duplicate it */
//         Status = SeSubProcessToken(ParentToken,
//                                    &NewToken,
//                                    TRUE,
//                                    MmGetSessionId(Process));
//
//         /* Dereference the Parent */
//         ObFastDereferenceObject(&Parent->Token, ParentToken);
//
//         /* Set the new Token */
//         if (NT_SUCCESS(Status))
//         {
//             /* Initailize the fast reference */
//             ObInitializeFastReference(&Process->Token, NewToken);
//         }
//     }
//     else
//     {
//         /* No parent, assign the Boot Token */
//         ObInitializeFastReference(&Process->Token, NULL);
//         SeAssignPrimaryToken(Process, PspBootAccessToken);
//     }
//
//     /* Return to caller */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// PspWriteTebImpersonationInfo(IN PETHREAD Thread,
//                              IN PETHREAD CurrentThread)
// {
//     PEPROCESS Process;
//     PTEB Teb;
//     BOOLEAN Attached = FALSE;
//     BOOLEAN IsImpersonating;
//     KAPC_STATE ApcState;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Thread: %p\n", Thread);
//
//     /* Sanity check */
//     ASSERT(CurrentThread == PsGetCurrentThread());
//
//     /* Get process and TEB */
//     Process = Thread->ThreadsProcess;
//     Teb = Thread->Tcb.Teb;
//     if (Teb)
//     {
//         /* Check if we're not in the right process */
//         if (Thread->Tcb.ApcState.Process != &Process->Pcb)
//         {
//             /* Attach to the process */
//             KeStackAttachProcess(&Process->Pcb, &ApcState);
//             Attached = TRUE;
//         }
//
//         /* Check if we're in a different thread or acquire rundown */
//         if ((Thread == CurrentThread) ||
//             (ExAcquireRundownProtection(&Thread->RundownProtect)))
//         {
//             /* Check if the thread is impersonating */
//             IsImpersonating = (BOOLEAN)Thread->ActiveImpersonationInfo;
//             if (IsImpersonating)
//             {
//                 /* Set TEB data */
//                 Teb->ImpersonationLocale = -1;
//                 Teb->IsImpersonating = 1;
//             }
//             else
//             {
//                 /* Set TEB data */
//                 Teb->ImpersonationLocale = 0;
//                 Teb->IsImpersonating = 0;
//             }
//         }
//
//         /* Check if we're in a different thread */
//         if (Thread != CurrentThread)
//         {
//             /* Release protection */
//             ExReleaseRundownProtection(&Thread->RundownProtect);
//         }
//
//         /* Detach */
//         if (Attached) KeUnstackDetachProcess(&ApcState);
//     }
//
//     /* Return to caller */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// PspAssignPrimaryToken(IN PEPROCESS Process,
//                       IN HANDLE Token,
//                       IN PACCESS_TOKEN AccessToken OPTIONAL)
// {
//     PACCESS_TOKEN NewToken = AccessToken, OldToken;
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Process: %p Token: %p\n", Process, Token);
//
//     /* Check if we don't have a pointer */
//     if (!AccessToken)
//     {
//         /* Reference it from the handle */
//         Status = ObReferenceObjectByHandle(Token,
//                                            TOKEN_ASSIGN_PRIMARY,
//                                            SeTokenObjectType,
//                                            ExGetPreviousMode(),
//                                            &NewToken,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Exchange tokens */
//     Status = SeExchangePrimaryToken(Process, NewToken, &OldToken);
//
//     /* Acquire and release the lock */
//     PspLockProcessSecurityExclusive(Process);
//     PspUnlockProcessSecurityExclusive(Process);
//
//     /* Dereference Tokens and Return */
//     if (NT_SUCCESS(Status)) ObDereferenceObject(OldToken);
//     if (!AccessToken) ObDereferenceObject(NewToken);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// PspSetPrimaryToken(IN PEPROCESS Process,
//                    IN HANDLE TokenHandle OPTIONAL,
//                    IN PACCESS_TOKEN Token OPTIONAL)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     BOOLEAN IsChildOrSibling;
//     PACCESS_TOKEN NewToken = Token;
//     NTSTATUS Status, AccessStatus;
//     BOOLEAN Result, SdAllocated;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//
//     PSTRACE(PS_SECURITY_DEBUG, "Process: %p Token: %p\n", Process, Token);
//
//     /* Reference the token by handle if we don't already have a token object */
//     if (!Token)
//     {
//         Status = ObReferenceObjectByHandle(TokenHandle,
//                                            TOKEN_ASSIGN_PRIMARY,
//                                            SeTokenObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&NewToken,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /*
//      * Check whether this token is a child or sibling of the current process token.
//      * NOTE: On Windows Vista+ both of these checks (together with extra steps)
//      * are now performed by a new SeIsTokenAssignableToProcess() helper.
//      */
//     Status = SeIsTokenChild(NewToken, &IsChildOrSibling);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, dereference */
//         if (!Token) ObDereferenceObject(NewToken);
//         return Status;
//     }
//     if (!IsChildOrSibling)
//     {
//         Status = SeIsTokenSibling(NewToken, &IsChildOrSibling);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Failed, dereference */
//             if (!Token) ObDereferenceObject(NewToken);
//             return Status;
//         }
//     }
//
//     /* Check if this was an independent token */
//     if (!IsChildOrSibling)
//     {
//         /* Make sure we have the privilege to assign a new one */
//         if (!SeSinglePrivilegeCheck(SeAssignPrimaryTokenPrivilege,
//                                     PreviousMode))
//         {
//             /* Failed, dereference */
//             if (!Token) ObDereferenceObject(NewToken);
//             return STATUS_PRIVILEGE_NOT_HELD;
//         }
//     }
//
//     /* Assign the token */
//     Status = PspAssignPrimaryToken(Process, NULL, NewToken);
//     if (NT_SUCCESS(Status))
//     {
//         /*
//          * We need to completely reverify if the process still has access to
//          * itself under this new token.
//          */
//         Status = ObGetObjectSecurity(Process,
//                                      &SecurityDescriptor,
//                                      &SdAllocated);
//         if (NT_SUCCESS(Status))
//         {
//             /* Setup the security context */
//             SubjectContext.ProcessAuditId = Process;
//             SubjectContext.PrimaryToken = PsReferencePrimaryToken(Process);
//             SubjectContext.ClientToken = NULL;
//
//             /* Do the access check */
//             Result = SeAccessCheck(SecurityDescriptor,
//                                    &SubjectContext,
//                                    FALSE,
//                                    MAXIMUM_ALLOWED,
//                                    0,
//                                    NULL,
//                                    &PsProcessType->TypeInfo.GenericMapping,
//                                    PreviousMode,
//                                    &Process->GrantedAccess,
//                                    &AccessStatus);
//
//             /* Dereference the token and let go the SD */
//             ObFastDereferenceObject(&Process->Token,
//                                     SubjectContext.PrimaryToken);
//             ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
//
//             /* Remove access if it failed */
//             if (!Result) Process->GrantedAccess = 0;
//
//             /* Setup granted access */
//             Process->GrantedAccess |= (PROCESS_VM_OPERATION |
//                                        PROCESS_VM_READ |
//                                        PROCESS_VM_WRITE |
//                                        PROCESS_QUERY_INFORMATION |
//                                        PROCESS_TERMINATE |
//                                        PROCESS_CREATE_THREAD |
//                                        PROCESS_DUP_HANDLE |
//                                        PROCESS_CREATE_PROCESS |
//                                        PROCESS_SET_INFORMATION |
//                                        STANDARD_RIGHTS_ALL |
//                                        PROCESS_SET_QUOTA);
//         }
//
//         /*
//          * In case LUID device maps are enable, we may not be using
//          * system device map for this process, but a logon LUID based
//          * device map. Because we change primary token, this usage is
//          * no longer valid, so dereference the process device map
//          */
//         if (ObIsLUIDDeviceMapsEnabled()) ObDereferenceDeviceMap(Process);
//     }
//
//     /* Dereference the token */
//     if (!Token) ObDereferenceObject(NewToken);
//     return Status;
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtOpenProcessToken(IN HANDLE ProcessHandle,
//                    IN ACCESS_MASK DesiredAccess,
//                    OUT PHANDLE TokenHandle)
// {
//     /* Call the newer API */
//     return NtOpenProcessTokenEx(ProcessHandle,
//                                 DesiredAccess,
//                                 0,
//                                 TokenHandle);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtOpenProcessTokenEx(IN HANDLE ProcessHandle,
//                      IN ACCESS_MASK DesiredAccess,
//                      IN ULONG HandleAttributes,
//                      OUT PHANDLE TokenHandle)
// {
//     PACCESS_TOKEN Token;
//     HANDLE hToken;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG,
//             "Process: %p DesiredAccess: %lx\n", ProcessHandle, DesiredAccess);
//
//     /* Check if caller was user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the token handle */
//             ProbeForWriteHandle(TokenHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Validate object attributes */
//     HandleAttributes = ObpValidateAttributes(HandleAttributes, PreviousMode);
//
//     /* Open the process token */
//     Status = PsOpenTokenOfProcess(ProcessHandle, &Token);
//     if (NT_SUCCESS(Status))
//     {
//         /* Reference it by handle and dereference the pointer */
//         Status = ObOpenObjectByPointer(Token,
//                                        HandleAttributes,
//                                        NULL,
//                                        DesiredAccess,
//                                        SeTokenObjectType,
//                                        PreviousMode,
//                                        &hToken);
//         ObDereferenceObject(Token);
//
//         /* Make sure we got a handle */
//         if (NT_SUCCESS(Status))
//         {
//             /* Enter SEH for write */
//             _SEH2_TRY
//             {
//                 /* Return the handle */
//                 *TokenHandle = hToken;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// PACCESS_TOKEN
// NTAPI
// PsReferencePrimaryToken(PEPROCESS Process)
// {
//     PACCESS_TOKEN Token;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Process: %p\n", Process);
//
//     /* Fast Reference the Token */
//     Token = ObFastReferenceObject(&Process->Token);
//
//     /* Check if we got the Token or if we got locked */
//     if (!Token)
//     {
//         /* Lock the Process */
//         PspLockProcessSecurityShared(Process);
//
//         /* Do a Locked Fast Reference */
//         Token = ObFastReferenceObjectLocked(&Process->Token);
//
//         /* Unlock the Process */
//         PspUnlockProcessSecurityShared(Process);
//     }
//
//     /* Return the Token */
//     return Token;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsOpenTokenOfProcess(IN HANDLE ProcessHandle,
//                      OUT PACCESS_TOKEN* Token)
// {
//     PEPROCESS Process;
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Process: %p\n", ProcessHandle);
//
//     /* Get the Token */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_QUERY_INFORMATION,
//                                        PsProcessType,
//                                        ExGetPreviousMode(),
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Reference the token and dereference the process */
//         *Token = PsReferencePrimaryToken(Process);
//         ObDereferenceObject(Process);
//     }
//
//     /* Return */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsAssignImpersonationToken(IN PETHREAD Thread,
//                            IN HANDLE TokenHandle)
// {
//     PACCESS_TOKEN Token;
//     SECURITY_IMPERSONATION_LEVEL ImpersonationLevel;
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Thread: %p Token: %p\n", Thread, TokenHandle);
//
//     /* Check if we were given a handle */
//     if (!TokenHandle)
//     {
//         /* Undo impersonation */
//         PsRevertThreadToSelf(Thread);
//         return STATUS_SUCCESS;
//     }
//
//     /* Get the token object */
//     Status = ObReferenceObjectByHandle(TokenHandle,
//                                        TOKEN_IMPERSONATE,
//                                        SeTokenObjectType,
//                                        KeGetPreviousMode(),
//                                        (PVOID*)&Token,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return(Status);
//
//     /* Make sure it's an impersonation token */
//     if (SeTokenType(Token) != TokenImpersonation)
//     {
//         /* Fail */
//         ObDereferenceObject(Token);
//         return STATUS_BAD_TOKEN_TYPE;
//     }
//
//     /* Get the impersonation level */
//     ImpersonationLevel = SeTokenImpersonationLevel(Token);
//
//     /* Call the impersonation API */
//     Status = PsImpersonateClient(Thread,
//                                  Token,
//                                  FALSE,
//                                  FALSE,
//                                  ImpersonationLevel);
//
//     /* Dereference the token and return status */
//     ObDereferenceObject(Token);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsRevertToSelf(VOID)
// {
//     /* Call the per-thread API */
//     PAGED_CODE();
//     PsRevertThreadToSelf(PsGetCurrentThread());
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsRevertThreadToSelf(IN PETHREAD Thread)
// {
//     PTOKEN Token = NULL;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Thread: %p\n", Thread);
//
//     /* Make sure we had impersonation information */
//     if (Thread->ActiveImpersonationInfo)
//     {
//         /* Lock the thread security */
//         PspLockThreadSecurityExclusive(Thread);
//
//         /* Make sure it's still active */
//         if (Thread->ActiveImpersonationInfo)
//         {
//             /* Disable impersonation */
//             PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
//
//             /* Get the token */
//             Token = Thread->ImpersonationInfo->Token;
//         }
//
//         /* Release thread security */
//         PspUnlockThreadSecurityExclusive(Thread);
//
//         /* Check if we had a token */
//         if (Token)
//         {
//             /* Dereference the impersonation token */
//             ObDereferenceObject(Token);
//
//             /* Write impersonation info to the TEB */
//             PspWriteTebImpersonationInfo(Thread, PsGetCurrentThread());
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsImpersonateClient(IN PETHREAD Thread,
//                     IN PACCESS_TOKEN Token,
//                     IN BOOLEAN CopyOnOpen,
//                     IN BOOLEAN EffectiveOnly,
//                     IN SECURITY_IMPERSONATION_LEVEL ImpersonationLevel)
// {
//     PPS_IMPERSONATION_INFORMATION Impersonation, OldData;
//     PTOKEN OldToken = NULL, ProcessToken = NULL;
//     BOOLEAN CopiedToken = FALSE;
//     PACCESS_TOKEN NewToken, ImpersonationToken;
//     PEJOB Job;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Thread: %p, Token: %p\n", Thread, Token);
//
//     /* Check if we don't have a token */
//     if (!Token)
//     {
//         /* Make sure we're impersonating */
//         if (Thread->ActiveImpersonationInfo)
//         {
//             /* We seem to be, lock the thread */
//             PspLockThreadSecurityExclusive(Thread);
//
//             /* Make sure we're still impersonating */
//             if (Thread->ActiveImpersonationInfo)
//             {
//                 /* Disable impersonation */
//                 PspClearCrossThreadFlag(Thread,
//                                         CT_ACTIVE_IMPERSONATION_INFO_BIT);
//
//                 /* Get the token */
//                 OldToken = Thread->ImpersonationInfo->Token;
//             }
//
//             /* Unlock the process and write TEB information */
//             PspUnlockThreadSecurityExclusive(Thread);
//             PspWriteTebImpersonationInfo(Thread, PsGetCurrentThread());
//         }
//     }
//     else
//     {
//         /* Check if we have impersonation info */
//         Impersonation = Thread->ImpersonationInfo;
//         if (!Impersonation)
//         {
//             /* We need to allocate a new one */
//             Impersonation = ExAllocatePoolWithTag(PagedPool,
//                                                   sizeof(*Impersonation),
//                                                   TAG_PS_IMPERSONATION);
//             if (!Impersonation) return STATUS_INSUFFICIENT_RESOURCES;
//
//             /* Update the pointer */
//             OldData = InterlockedCompareExchangePointer((PVOID*)&Thread->
//                                                         ImpersonationInfo,
//                                                         Impersonation,
//                                                         NULL);
//             if (OldData)
//             {
//                 /* Someone beat us to it, free our copy */
//                 ExFreePoolWithTag(Impersonation, TAG_PS_IMPERSONATION);
//                 Impersonation = OldData;
//             }
//         }
//
//         /*
//          * Assign the token we get from the caller first. The reason
//          * we have to do that is because we're unsure if we can impersonate
//          * in the first place. In the scenario where we cannot then the
//          * last resort is to make a copy of the token and assign that newly
//          * token to the impersonation information.
//          */
//         ImpersonationToken = Token;
//
//         /* Obtain a token from the process */
//         ProcessToken = PsReferencePrimaryToken(Thread->ThreadsProcess);
//         if (!ProcessToken)
//         {
//             /* We can't continue this way without having the process' token... */
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* Make sure we can impersonate */
//         if (!SeTokenCanImpersonate(ProcessToken,
//                                    Token,
//                                    ImpersonationLevel))
//         {
//             /* We can't, make a copy of the token instead */
//             Status = SeCopyClientToken(Token,
//                                        SecurityIdentification,
//                                        KernelMode,
//                                        &NewToken);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* We can't even make a copy of the token? Then bail out... */
//                 ObFastDereferenceObject(&Thread->ThreadsProcess->Token, ProcessToken);
//                 return Status;
//             }
//
//             /*
//              * Since we cannot impersonate, assign the newly copied token.
//              * SeCopyClientToken already holds a reference to the copied token,
//              * let the code path below know that it must not reference it twice.
//              */
//             CopiedToken = TRUE;
//             ImpersonationLevel = SecurityIdentification;
//             ImpersonationToken = NewToken;
//         }
//
//         /* We no longer need the process' token */
//         ObFastDereferenceObject(&Thread->ThreadsProcess->Token, ProcessToken);
//
//         /* Check if this is a job */
//         Job = Thread->ThreadsProcess->Job;
//         if (Job != NULL)
//         {
//             /* No admin allowed in this job */
//             if ((Job->SecurityLimitFlags & JOB_OBJECT_SECURITY_NO_ADMIN) &&
//                 SeTokenIsAdmin(ImpersonationToken))
//             {
//                 if (CopiedToken)
//                 {
//                     ObDereferenceObject(ImpersonationToken);
//                 }
//
//                 return STATUS_ACCESS_DENIED;
//             }
//
//             /* No restricted tokens allowed in this job */
//             if ((Job->SecurityLimitFlags & JOB_OBJECT_SECURITY_RESTRICTED_TOKEN) &&
//                 SeTokenIsRestricted(ImpersonationToken))
//             {
//                 if (CopiedToken)
//                 {
//                     ObDereferenceObject(ImpersonationToken);
//                 }
//
//                 return STATUS_ACCESS_DENIED;
//             }
//
//             /* We don't support job filters yet */
//             if (Job->Filter != NULL)
//             {
//                 ASSERT(Job->Filter == NULL);
//             }
//         }
//
//         /* Lock thread security */
//         PspLockThreadSecurityExclusive(Thread);
//
//         /* Check if we're impersonating */
//         if (Thread->ActiveImpersonationInfo)
//         {
//             /* Get the token */
//             OldToken = Impersonation->Token;
//         }
//         else
//         {
//             /* Otherwise, enable impersonation */
//             PspSetCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
//         }
//
//         /* Now fill it out */
//         Impersonation->ImpersonationLevel = ImpersonationLevel;
//         Impersonation->CopyOnOpen = CopyOnOpen;
//         Impersonation->EffectiveOnly = EffectiveOnly;
//         Impersonation->Token = ImpersonationToken;
//
//         /* Do not reference the token again if we copied it */
//         if (!CopiedToken)
//         {
//             ObReferenceObject(ImpersonationToken);
//         }
//
//         /* Unlock the thread */
//         PspUnlockThreadSecurityExclusive(Thread);
//
//         /* Write impersonation info to the TEB */
//         PspWriteTebImpersonationInfo(Thread, PsGetCurrentThread());
//     }
//
//     /* Dereference the token and return success */
//     if (OldToken) PsDereferenceImpersonationToken(OldToken);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// PACCESS_TOKEN
// NTAPI
// PsReferenceEffectiveToken(IN PETHREAD Thread,
//                           OUT IN PTOKEN_TYPE TokenType,
//                           OUT PBOOLEAN EffectiveOnly,
//                           OUT PSECURITY_IMPERSONATION_LEVEL ImpersonationLevel)
// {
//     PEPROCESS Process;
//     PACCESS_TOKEN Token = NULL;
//
//     PAGED_CODE();
//
//     PSTRACE(PS_SECURITY_DEBUG,
//             "Thread: %p, TokenType: %p\n", Thread, TokenType);
//
//     /* Check if we don't have impersonation info */
//     Process = Thread->ThreadsProcess;
//     if (Thread->ActiveImpersonationInfo)
//     {
//         /* Lock the Process */
//         PspLockProcessSecurityShared(Process);
//
//         /* Make sure impersonation is still active */
//         if (Thread->ActiveImpersonationInfo)
//         {
//             /* Get the token */
//             Token = Thread->ImpersonationInfo->Token;
//             ObReferenceObject(Token);
//
//             /* Return data to caller */
//             *TokenType = TokenImpersonation;
//             *EffectiveOnly = Thread->ImpersonationInfo->EffectiveOnly;
//             *ImpersonationLevel = Thread->ImpersonationInfo->ImpersonationLevel;
//
//             /* Unlock the Process */
//             PspUnlockProcessSecurityShared(Process);
//             return Token;
//         }
//
//         /* Unlock the Process */
//         PspUnlockProcessSecurityShared(Process);
//     }
//
//     /* Fast Reference the Token */
//     Token = ObFastReferenceObject(&Process->Token);
//
//     /* Check if we got the Token or if we got locked */
//     if (!Token)
//     {
//         /* Lock the Process */
//         PspLockProcessSecurityShared(Process);
//
//         /* Do a Locked Fast Reference */
//         Token = ObFastReferenceObjectLocked(&Process->Token);
//
//         /* Unlock the Process */
//         PspUnlockProcessSecurityShared(Process);
//     }
//
//     /* Return the token */
//     *TokenType = TokenPrimary;
//     *EffectiveOnly = FALSE;
//     // NOTE: ImpersonationLevel is left untouched on purpose!
//     return Token;
// }
//
// /*
//  * @implemented
//  */
// PACCESS_TOKEN
// NTAPI
// PsReferenceImpersonationToken(IN PETHREAD Thread,
//                               OUT PBOOLEAN CopyOnOpen,
//                               OUT PBOOLEAN EffectiveOnly,
//                               OUT PSECURITY_IMPERSONATION_LEVEL ImpersonationLevel)
// {
//     PTOKEN Token = NULL;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG, "Thread: %p\n", Thread);
//
//     /* If we don't have impersonation info, just quit */
//     if (!Thread->ActiveImpersonationInfo) return NULL;
//
//     /* Lock the thread */
//     PspLockThreadSecurityShared(Thread);
//
//     /* Make sure we still have active impersonation */
//     if (Thread->ActiveImpersonationInfo)
//     {
//         /* Return data from caller */
//         ObReferenceObject(Thread->ImpersonationInfo->Token);
//         *ImpersonationLevel = Thread->ImpersonationInfo->ImpersonationLevel;
//         *CopyOnOpen = Thread->ImpersonationInfo->CopyOnOpen;
//         *EffectiveOnly = Thread->ImpersonationInfo->EffectiveOnly;
//
//         /* Set the token */
//         Token = Thread->ImpersonationInfo->Token;
//     }
//
//     /* Unlock thread and return impersonation token */
//     PspUnlockThreadSecurityShared(Thread);
//     return Token;
// }
//
// #undef PsDereferenceImpersonationToken
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsDereferenceImpersonationToken(IN PACCESS_TOKEN ImpersonationToken)
// {
//     PAGED_CODE();
//
//     /* If we got a token, dereference it */
//     if (ImpersonationToken) ObDereferenceObject(ImpersonationToken);
// }
//
// #undef PsDereferencePrimaryToken
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsDereferencePrimaryToken(IN PACCESS_TOKEN PrimaryToken)
// {
//     PAGED_CODE();
//
//     /* Dereference the token*/
//     ObDereferenceObject(PrimaryToken);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsDisableImpersonation(IN PETHREAD Thread,
//                        OUT PSE_IMPERSONATION_STATE ImpersonationState)
// {
//     PPS_IMPERSONATION_INFORMATION Impersonation = NULL;
//     LONG OldFlags;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG,
//             "Thread: %p State: %p\n", Thread, ImpersonationState);
//
//     /* Check if we don't have impersonation */
//     if (Thread->ActiveImpersonationInfo)
//     {
//         /* Lock thread security */
//         PspLockThreadSecurityExclusive(Thread);
//
//         /* Disable impersonation */
//         OldFlags = PspClearCrossThreadFlag(Thread,
//                                            CT_ACTIVE_IMPERSONATION_INFO_BIT);
//
//         /* Make sure nobody disabled it behind our back */
//         if (OldFlags & CT_ACTIVE_IMPERSONATION_INFO_BIT)
//         {
//             /* Copy the old state */
//             Impersonation = Thread->ImpersonationInfo;
//             ImpersonationState->Token = Impersonation->Token;
//             ImpersonationState->CopyOnOpen = Impersonation->CopyOnOpen;
//             ImpersonationState->EffectiveOnly = Impersonation->EffectiveOnly;
//             ImpersonationState->Level = Impersonation->ImpersonationLevel;
//         }
//
//         /* Unlock thread security */
//         PspUnlockThreadSecurityExclusive(Thread);
//
//         /* If we had impersonation info, return true */
//         if (Impersonation) return TRUE;
//     }
//
//     /* Clear everything */
//     ImpersonationState->Token = NULL;
//     ImpersonationState->CopyOnOpen = FALSE;
//     ImpersonationState->EffectiveOnly = FALSE;
//     ImpersonationState->Level = SecurityAnonymous;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsRestoreImpersonation(IN PETHREAD Thread,
//                        IN PSE_IMPERSONATION_STATE ImpersonationState)
// {
//     PTOKEN Token = NULL;
//     PPS_IMPERSONATION_INFORMATION Impersonation;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG,
//             "Thread: %p State: %p\n", Thread, ImpersonationState);
//
//     /* Lock thread security */
//     PspLockThreadSecurityExclusive(Thread);
//
//     /* Get the impersonation info */
//     Impersonation = Thread->ImpersonationInfo;
//
//     /* Check if we're impersonating */
//     if (Thread->ActiveImpersonationInfo)
//     {
//         /* Get the token */
//         Token = Impersonation->Token;
//     }
//
//     /* Check if we have an impersonation state */
//     if (ImpersonationState)
//     {
//         /* Fill out the impersonation info */
//         Impersonation->ImpersonationLevel = ImpersonationState->Level;
//         Impersonation->CopyOnOpen = ImpersonationState->CopyOnOpen;
//         Impersonation->EffectiveOnly = ImpersonationState->EffectiveOnly;
//         Impersonation->Token = ImpersonationState->Token;
//
//         /* Enable impersonation */
//         PspSetCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
//     }
//     else
//     {
//         /* Disable impersonation */
//         PspClearCrossThreadFlag(Thread, CT_ACTIVE_IMPERSONATION_INFO_BIT);
//     }
//
//     /* Unlock the thread */
//     PspUnlockThreadSecurityExclusive(Thread);
//
//     /* Dereference the token */
//     if (Token) ObDereferenceObject(Token);
// }
//
// NTSTATUS
// NTAPI
// NtImpersonateThread(IN HANDLE ThreadHandle,
//                     IN HANDLE ThreadToImpersonateHandle,
//                     IN PSECURITY_QUALITY_OF_SERVICE SecurityQualityOfService)
// {
//     SECURITY_QUALITY_OF_SERVICE SafeServiceQoS;
//     SECURITY_CLIENT_CONTEXT ClientContext;
//     PETHREAD Thread;
//     PETHREAD ThreadToImpersonate;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_SECURITY_DEBUG,
//             "Threads: %p %p\n", ThreadHandle, ThreadToImpersonateHandle);
//
//     /* Check if call came from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe QoS */
//             ProbeForRead(SecurityQualityOfService,
//                          sizeof(SECURITY_QUALITY_OF_SERVICE),
//                          sizeof(ULONG));
//
//             /* Capture it */
//             SafeServiceQoS = *SecurityQualityOfService;
//             SecurityQualityOfService = &SafeServiceQoS;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Reference the thread */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_DIRECT_IMPERSONATION,
//                                        PsThreadType,
//                                        PreviousMode,
//                                        (PVOID*)&Thread,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Reference the impersonating thead */
//         Status = ObReferenceObjectByHandle(ThreadToImpersonateHandle,
//                                            THREAD_IMPERSONATE,
//                                            PsThreadType,
//                                            PreviousMode,
//                                            (PVOID*)&ThreadToImpersonate,
//                                            NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Create a client security context */
//             Status = SeCreateClientSecurity(ThreadToImpersonate,
//                                             SecurityQualityOfService,
//                                             0,
//                                             &ClientContext);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Do the impersonation */
//                 SeImpersonateClient(&ClientContext, Thread);
//                 if (ClientContext.ClientToken)
//                 {
//                     /* Dereference the client token if we had one */
//                     ObDereferenceObject(ClientContext.ClientToken);
//                 }
//             }
//
//             /* Dereference the thread to impersonate */
//             ObDereferenceObject(ThreadToImpersonate);
//         }
//
//         /* Dereference the main thread */
//         ObDereferenceObject(Thread);
//     }
//
//     /* Return status */
//     return Status;
// }
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
