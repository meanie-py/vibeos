use ntkrnl_sys::globals::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::constants::*;
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: *const ::core::ffi::c_char, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
pub type PSECURITY_PACKAGE_OPTIONS = PVOID;
pub type PSecurityUserData = PVOID;

#[no_mangle]
pub unsafe extern "system" fn AcceptSecurityContext(
    mut phCredential: PCredHandle,
    mut phContext: PCtxtHandle,
    mut pInput: PSecBufferDesc,
    mut fContextReq: ULONG,
    mut TargetDataRep: ULONG,
    mut phNewContext: PCtxtHandle,
    mut pOutput: PSecBufferDesc,
    mut pfContextAttr: PULONG,
    mut ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"AcceptSecurityContext\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        33 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn AcquireCredentialsHandleW(
    pPrincipal: PSSPI_SEC_STRING,
    pPackage: PSSPI_SEC_STRING,
    fCredentialUse: ULONG,
    pvLogonId: PVOID,
    pAuthData: PVOID,
    pGetKeyFn: SEC_GET_KEY_FN,
    pvGetKeyArgument: PVOID,
    phCredential: PCredHandle,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"AcquireCredentialsHandleW\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        50 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn AddCredentialsW(
    mut hCredentials: PCredHandle,
    mut pPrincipal: PSSPI_SEC_STRING,
    mut pPackage: PSSPI_SEC_STRING,
    mut fCredentialUse: ULONG,
    mut pAuthData: PVOID,
    mut pGetKeyFn: SEC_GET_KEY_FN,
    mut pvGetKeyArgument: PVOID,
    mut ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"AddCredentialsW\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        66 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn ApplyControlToken(
    mut phContext: PCtxtHandle,
    mut pInput: PSecBufferDesc,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"ApplyControlToken\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        76 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn CredMarshalTargetInfo() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"CredMarshalTargetInfo\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        84 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn DeleteSecurityContext(mut phContext: PCtxtHandle) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"DeleteSecurityContext\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        92 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn EfsDecryptFek() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"EfsDecryptFek\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        100 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn EfsGenerateKey() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"EfsGenerateKey\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        107 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn EnumerateSecurityPackagesW(
    pcPackages: PULONG,
    ppPackageInfo: *mut PSecPkgInfoW,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"EnumerateSecurityPackagesW\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const core::ffi::c_char,
        116 as core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn ExportSecurityContext(
    mut phContext: PCtxtHandle,
    mut fFlags: ULONG,
    mut pPackedContext: PSecBuffer,
    mut pToken: *mut PVOID,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"ExportSecurityContext\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const core::ffi::c_char,
        128 as core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn FreeContextBuffer(mut pvContextBuffer: PVOID) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"FreeContextBuffer\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        137 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn FreeCredentialsHandle(mut phCredential: PCredHandle) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"FreeCredentialsHandle\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        146 as core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn GenerateDirEfs() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"GenerateDirEfs\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        154 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn GenerateSessionKey() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"GenerateSessionKey\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        161 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn GetSecurityUserInfo(
    mut LogonId: PLUID,
    mut Flags: ULONG,
    mut UserInformation: *mut PSecurityUserData,
) -> SECURITY_STATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"GetSecurityUserInfo\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        171 as i32,
    );
    *UserInformation = core::ptr::null_mut() as PSecurityUserData;
    return STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "system" fn ImpersonateSecurityContext(
    mut phContext: PCtxtHandle,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ImpersonateSecurityContext\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        181 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn ImportSecurityContextW(
    mut pszPackage: PSSPI_SEC_STRING,
    mut pPackedContext: PSecBuffer,
    mut Token: PVOID,
    mut phContext: PCtxtHandle,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"ImportSecurityContextW\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const core::ffi::c_char,
        193 as core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn InitializeSecurityContextW(
    phCredential: PCredHandle,
    phContext: PCtxtHandle,
    pTargetName: PSSPI_SEC_STRING,
    fContextReq: ULONG,
    Reserved1: ULONG,
    TargetDataRep: ULONG,
    pInput: PSecBufferDesc,
    Reserved2: ULONG,
    phNewContext: PCtxtHandle,
    pOutput: PSecBufferDesc,
    pfContextAttr: PULONG,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"InitializeSecurityContextW\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        213 as core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn InitSecurityInterfaceW() -> PSecurityFunctionTableW {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"InitSecurityInterfaceW\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        222 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn KSecRegisterSecurityProvider() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"KSecRegisterSecurityProvider\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        230 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn KSecValidateBuffer() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"KSecValidateBuffer\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        237 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn LsaEnumerateLogonSessions() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"LsaEnumerateLogonSessions\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        244 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn LsaGetLogonSessionData() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"LsaGetLogonSessionData\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        251 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn MakeSignature(
    mut phContext: PCtxtHandle,
    mut fQOP: ULONG,
    mut pMessage: PSecBufferDesc,
    mut MessageSeqNo: ULONG,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MakeSignature\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        262 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn MapSecurityError() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"MapSecurityError\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        270 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn QueryContextAttributesW(
    mut phContext: PCtxtHandle,
    mut ulAttribute: ULONG,
    mut pBuffer: PVOID,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"QueryContextAttributesW\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        280 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn QueryCredentialsAttributesW(
    mut phCredential: PCredHandle,
    mut ulAttribute: ULONG,
    mut pBuffer: PVOID,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"QueryCredentialsAttributesW\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        291 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn QuerySecurityContextToken(
    mut phContext: PCtxtHandle,
    mut Token: *mut PVOID,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"QuerySecurityContextToken\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const core::ffi::c_char,
        301 as core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn QuerySecurityPackageInfoW(
    pPackageName: PSSPI_SEC_STRING,
    ppPackageInfo: *mut PSecPkgInfoW,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"QuerySecurityPackageInfoW\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        311 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn RevertSecurityContext(mut phContext: PCtxtHandle) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"RevertSecurityContext\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        320 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn SealMessage() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"SealMessage\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        328 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn SecLookupAccountName(
    mut Name: PUNICODE_STRING,
    mut SidSize: PULONG,
    mut Sid: PSID,
    mut NameUse: PSID_NAME_USE,
    mut DomainSize: PULONG,
    mut ReferencedDomain: PUNICODE_STRING,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"SecLookupAccountName\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        341 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn SecLookupAccountSid(
    mut Sid: PSID,
    mut NameSize: PULONG,
    mut NameBuffer: PUNICODE_STRING,
    mut DomainSize: PULONG,
    mut DomainBuffer: PUNICODE_STRING,
    mut NameUse: PSID_NAME_USE,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"SecLookupAccountSid\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        355 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn SecLookupWellKnownSid(
    mut SidType: WELL_KNOWN_SID_TYPE,
    mut Sid: PSID,
    mut SidBufferSize: ULONG,
    mut SidSize: PULONG,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"SecLookupWellKnownSid\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        367 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn SecMakeSPN(
    mut ServiceClass: PUNICODE_STRING,
    mut ServiceName: PUNICODE_STRING,
    mut InstanceName: PUNICODE_STRING,
    mut InstancePort: USHORT,
    mut Referrer: PUNICODE_STRING,
    mut Spn: PUNICODE_STRING,
    mut Length: PULONG,
    mut Allocate: BOOLEAN,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"SecMakeSPN\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        383 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as *const ::core::ffi::c_char);
    DbgBreakPoint();
    return 0 as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn SecMakeSPNEx(
    mut ServiceClass: PUNICODE_STRING,
    mut ServiceName: PUNICODE_STRING,
    mut InstanceName: PUNICODE_STRING,
    mut InstancePort: USHORT,
    mut Referrer: PUNICODE_STRING,
    mut TargetInfo: PUNICODE_STRING,
    mut Spn: PUNICODE_STRING,
    mut Length: PULONG,
    mut Allocate: BOOLEAN,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"SecMakeSPNEx\0" as *const u8 as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            as *const u8 as *const i8,
        400 as i32,
    );
    DbgPrint(b"\0" as *const u8 as *const i8);
    DbgBreakPoint();
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn SecSetPagingMode() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"SecSetPagingMode\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        408 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn UnsealMessage() {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"UnsealMessage\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0".as_ptr() as *const i8,
        414 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn VerifySignature(
    mut phContext: PCtxtHandle,
    mut pMessage: PSecBufferDesc,
    mut MessageSeqNo: ULONG,
    mut pfQOP: PULONG,
) -> SECURITY_STATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"VerifySignature\0".as_ptr() as *const i8,
        b"drivers/crypto/ksecdd/stubs.c\0"
            .as_ptr() as *const i8,
        425 as i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return 0 as SECURITY_STATUS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Drivers
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PURPOSE:         Kernel Security Support Provider Interface Driver
//  *
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "ksecdd.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef PVOID PSECURITY_PACKAGE_OPTIONS, PSecurityUserData;
//
// /* FUNCTIONS ******************************************************************/
//
// SECURITY_STATUS
// SEC_ENTRY
// AcceptSecurityContext(
//      PCredHandle phCredential,
//      PCtxtHandle phContext,
//      PSecBufferDesc pInput,
//      ULONG fContextReq,
//      ULONG TargetDataRep,
//      PCtxtHandle phNewContext,
//      PSecBufferDesc pOutput,
//      PULONG pfContextAttr,
//      PTimeStamp ptsExpiry)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// AcquireCredentialsHandleW(
//      PSSPI_SEC_STRING pPrincipal,
//      PSSPI_SEC_STRING pPackage,
//      ULONG fCredentialUse,
//      PVOID pvLogonId,
//      PVOID pAuthData,
//      SEC_GET_KEY_FN pGetKeyFn,
//      PVOID pvGetKeyArgument,
//      PCredHandle phCredential,
//      PTimeStamp ptsExpiry)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// AddCredentialsW(
//      PCredHandle hCredentials,
//      PSSPI_SEC_STRING pPrincipal,
//      PSSPI_SEC_STRING pPackage,
//      ULONG fCredentialUse,
//      PVOID pAuthData,
//      SEC_GET_KEY_FN pGetKeyFn,
//      PVOID pvGetKeyArgument,
//      PTimeStamp ptsExpiry)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// ApplyControlToken(
//      PCtxtHandle phContext,
//      PSecBufferDesc pInput)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// VOID
// SEC_ENTRY
// CredMarshalTargetInfo(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// DeleteSecurityContext(
//      PCtxtHandle phContext)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// VOID
// SEC_ENTRY
// EfsDecryptFek(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// VOID
// SEC_ENTRY
// EfsGenerateKey(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// EnumerateSecurityPackagesW(
//      PULONG pcPackages,
//      PSecPkgInfoW* ppPackageInfo)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// ExportSecurityContext(
//      PCtxtHandle phContext,
//      ULONG fFlags,
//      PSecBuffer pPackedContext,
//      PVOID* pToken)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// FreeContextBuffer(
//      PVOID pvContextBuffer)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// FreeCredentialsHandle(
//      PCredHandle phCredential)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// VOID
// SEC_ENTRY
// GenerateDirEfs(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// VOID
// SEC_ENTRY
// GenerateSessionKey(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// GetSecurityUserInfo(
//         PLUID LogonId,
//             ULONG Flags,
//         PSecurityUserData *UserInformation)
// {
//     UNIMPLEMENTED;
//     *UserInformation = NULL;
//     return STATUS_UNSUCCESSFUL;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// ImpersonateSecurityContext(
//      PCtxtHandle phContext)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// ImportSecurityContextW(
//      PSSPI_SEC_STRING pszPackage,
//      PSecBuffer pPackedContext,
//      PVOID Token,
//      PCtxtHandle phContext)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// InitializeSecurityContextW(
//      PCredHandle phCredential,
//      PCtxtHandle phContext,
//      PSSPI_SEC_STRING pTargetName,
//      ULONG fContextReq,
//      ULONG Reserved1,
//      ULONG TargetDataRep,
//      PSecBufferDesc pInput,
//      ULONG Reserved2,
//      PCtxtHandle phNewContext,
//      PSecBufferDesc pOutput,
//      PULONG pfContextAttr,
//      PTimeStamp ptsExpiry)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// PSecurityFunctionTableW
// SEC_ENTRY
// InitSecurityInterfaceW(void)
// {
//
//     UNIMPLEMENTED_DBGBREAK();
//     return NULL;
// }
//
// VOID
// SEC_ENTRY
// KSecRegisterSecurityProvider(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// VOID
// SEC_ENTRY
// KSecValidateBuffer(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// VOID
// SEC_ENTRY
// LsaEnumerateLogonSessions(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// VOID
// SEC_ENTRY
// LsaGetLogonSessionData(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// MakeSignature(
//      PCtxtHandle phContext,
//      ULONG fQOP,
//      PSecBufferDesc pMessage,
//      ULONG MessageSeqNo)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// VOID
// SEC_ENTRY
// MapSecurityError(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// QueryContextAttributesW(
//      PCtxtHandle phContext,
//      ULONG ulAttribute,
//      PVOID pBuffer)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// QueryCredentialsAttributesW(
//      PCredHandle phCredential,
//      ULONG ulAttribute,
//      PVOID pBuffer)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// QuerySecurityContextToken(
//      PCtxtHandle phContext,
//      PVOID* Token)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// QuerySecurityPackageInfoW(
//      PSSPI_SEC_STRING pPackageName,
//      PSecPkgInfoW *ppPackageInfo)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// RevertSecurityContext(
//      PCtxtHandle phContext)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// VOID
// SEC_ENTRY
// SealMessage(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// NTSTATUS
// SEC_ENTRY
// SecLookupAccountName(
//      PUNICODE_STRING Name,
//      PULONG SidSize,
//      PSID Sid,
//      PSID_NAME_USE NameUse,
//      PULONG DomainSize,
//      PUNICODE_STRING ReferencedDomain)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// NTSTATUS
// SEC_ENTRY
// SecLookupAccountSid(
//      PSID Sid,
//      PULONG NameSize,
//      PUNICODE_STRING NameBuffer,
//      PULONG DomainSize OPTIONAL,
//      PUNICODE_STRING DomainBuffer,
//      PSID_NAME_USE NameUse)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// NTSTATUS
// SEC_ENTRY
// SecLookupWellKnownSid(
//      WELL_KNOWN_SID_TYPE SidType,
//      PSID Sid,
//      ULONG SidBufferSize,
//      PULONG SidSize)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// SecMakeSPN(
//      PUNICODE_STRING ServiceClass,
//      PUNICODE_STRING ServiceName,
//      PUNICODE_STRING InstanceName,
//      USHORT InstancePort,
//      PUNICODE_STRING Referrer,
//      PUNICODE_STRING Spn,
//      PULONG Length,
//      BOOLEAN Allocate)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// SecMakeSPNEx(
//      PUNICODE_STRING ServiceClass,
//      PUNICODE_STRING ServiceName,
//      PUNICODE_STRING InstanceName,
//      USHORT InstancePort,
//      PUNICODE_STRING Referrer,
//      PUNICODE_STRING TargetInfo,
//      PUNICODE_STRING Spn,
//      PULONG Length,
//      BOOLEAN Allocate)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// VOID
// SEC_ENTRY
// SecSetPagingMode(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
// VOID
// SEC_ENTRY
// UnsealMessage(VOID)
// {
//     UNIMPLEMENTED_DBGBREAK();
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// VerifySignature(
//      PCtxtHandle phContext,
//      PSecBufferDesc pMessage,
//      ULONG MessageSeqNo,
//      PULONG pfQOP)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
