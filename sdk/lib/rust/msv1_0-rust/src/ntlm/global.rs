//! ntlm/global.c — the module's six globals. Hand-carried, not translated: the C file
//! defines no functions, so `rustconv translate` had nothing to buy here.
//!
//! The island declares all six as `extern` (global.h is in the allowlisted set), which is why
//! this crate CHECKED clean while they were still missing — they only go absent at LINK.
//! These are the definitions; every other module reaches them through the prelude glob and
//! binds to the same symbol, so they carry `#[no_mangle]` and the C spelling.
//!
//! The two dispatch tables were emitted MECHANICALLY from the C's designated initializers
//! (field name -> field name, order and full field set taken from the island's struct), because
//! the risk in 41 near-identical `LsaSp*`/`UsrSp*` names is TRANSPOSITION, not translation: a
//! slot pair swapped by eye compiles, links, boots, and then misauthenticates. The generated
//! C->Rust slot table is reproduced below each table so the check is reviewable without
//! re-deriving it. Fields the C leaves implicit are `None` here rather than absent — Rust has no
//! designated-initializer default, so E0063 would have named them anyway.
use crate::prelude::*;
// The 41 dispatch targets, imported from the TU that defines each. Listed per-module rather
// than globbed so that a name moving between TUs is a compile error here, not a silent rebind.
use crate::msv1_0::lsa::{
    LsaSpAcceptLsaModeContext, LsaSpAcquireCredentialsHandle, LsaSpApplyControlToken,
    LsaSpDeleteContext, LsaSpDeleteCredentials, LsaSpFreeCredentialsHandle, LsaSpGetCredentials,
    LsaSpGetExtendedInformation, LsaSpGetInfoW, LsaSpGetUserInfo, LsaSpInitLsaModeContext,
    LsaSpQueryCredentialsAttributes, LsaSpSaveCredentials, LsaSpSetExtendedInformation,
    LsaSpShutDown, SpAcceptCredentials, SpInitialize,
};
use crate::msv1_0::msv1_0::{
    LsaApCallPackage, LsaApCallPackagePassthrough, LsaApCallPackageUntrusted,
    LsaApLogonTerminated, LsaApLogonUserEx2,
};
use crate::msv1_0::user::{
    SpInstanceInit, UsrSpCompleteAuthToken, UsrSpDeleteUserModeContext,
    UsrSpExportSecurityContext, UsrSpFormatCredentials, UsrSpGetContextToken,
    UsrSpImportSecurityContext, UsrSpMakeSignature, UsrSpMarshallSupplementalCreds,
    UsrSpQueryContextAttributes, UsrSpSealMessage, UsrSpUnsealMessage, UsrSpVerifySignature,
};
use crate::msv1_0::usercontext::UsrSpInitUserModeContext;

// NTLM_MODE NtlmMode = NtlmUnknownMode;
#[no_mangle]
pub static mut NtlmMode: NTLM_MODE = NtlmUnknownMode;

// LSA_DISPATCH_TABLE DispatchTable;  — tentative definition, so zero-initialized.
#[no_mangle]
pub static mut DispatchTable: LSA_DISPATCH_TABLE = unsafe { core::mem::zeroed() };

// PLSA_SECPKG_FUNCTION_TABLE LsaFunctions = NULL;
#[no_mangle]
pub static mut LsaFunctions: PLSA_SECPKG_FUNCTION_TABLE = core::ptr::null_mut();

// PSECPKG_DLL_FUNCTIONS UsrFunctions = NULL;
#[no_mangle]
pub static mut UsrFunctions: PSECPKG_DLL_FUNCTIONS = core::ptr::null_mut();

// msv1_0 (XP, win2k) returns NULL for InitializePackage, LsaLogonUser, LsaLogonUserEx,
// SpQueryContextAttributes and SpAddCredentials — the C's comment, and the reason five of the
// 27 named slots are deliberately NULL rather than unimplemented.
//
// `LogonUserExW`, not the C's `.LogonUserEx`: winbase.h's `#define LogonUserEx LogonUserExW`
// under UNICODE rewrites the ntsecpkg.h MEMBER name, in the C TU exactly as in the island.
//
//  .InitializePackage            NULL                            -> None
//  .LsaLogonUser                 NULL                            -> None
//  .CallPackage                  LsaApCallPackage                -> Some(LsaApCallPackage)
//  .LogonTerminated              LsaApLogonTerminated            -> Some(LsaApLogonTerminated)
//  .CallPackageUntrusted         LsaApCallPackageUntrusted       -> Some(LsaApCallPackageUntrusted)
//  .CallPackagePassthrough       LsaApCallPackagePassthrough     -> Some(LsaApCallPackagePassthrough)
//  .LogonUserExW                 NULL                            -> None
//  .LogonUserEx2                 LsaApLogonUserEx2               -> Some(LsaApLogonUserEx2)
//  .Initialize                   SpInitialize                    -> Some(SpInitialize)
//  .Shutdown                     LsaSpShutDown                   -> Some(LsaSpShutDown)
//  .GetInfo                      LsaSpGetInfoW                   -> Some(LsaSpGetInfoW)
//  .AcceptCredentials            SpAcceptCredentials             -> Some(SpAcceptCredentials)
//  .SpAcquireCredentialsHandle   LsaSpAcquireCredentialsHandle   -> Some(LsaSpAcquireCredentialsHandle)
//  .SpQueryCredentialsAttributes LsaSpQueryCredentialsAttributes -> Some(LsaSpQueryCredentialsAttributes)
//  .FreeCredentialsHandle        LsaSpFreeCredentialsHandle      -> Some(LsaSpFreeCredentialsHandle)
//  .SaveCredentials              LsaSpSaveCredentials            -> Some(LsaSpSaveCredentials)
//  .GetCredentials               LsaSpGetCredentials             -> Some(LsaSpGetCredentials)
//  .DeleteCredentials            LsaSpDeleteCredentials          -> Some(LsaSpDeleteCredentials)
//  .InitLsaModeContext           LsaSpInitLsaModeContext         -> Some(LsaSpInitLsaModeContext)
//  .AcceptLsaModeContext         LsaSpAcceptLsaModeContext       -> Some(LsaSpAcceptLsaModeContext)
//  .DeleteContext                LsaSpDeleteContext              -> Some(LsaSpDeleteContext)
//  .ApplyControlToken            LsaSpApplyControlToken          -> Some(LsaSpApplyControlToken)
//  .GetUserInfo                  LsaSpGetUserInfo                -> Some(LsaSpGetUserInfo)
//  .GetExtendedInformation       LsaSpGetExtendedInformation     -> Some(LsaSpGetExtendedInformation)
//  .SpQueryContextAttributes     NULL                            -> None
//  .SpAddCredentials             NULL                            -> None
//  .SetExtendedInformation       LsaSpSetExtendedInformation     -> Some(LsaSpSetExtendedInformation)
//  (the remaining 9 slots are implicit NULL in the C)
#[no_mangle]
pub static mut NtlmLsaFn: [SECPKG_FUNCTION_TABLE; 1] = [SECPKG_FUNCTION_TABLE {
    InitializePackage: None,
    LsaLogonUser: None,
    CallPackage: Some(LsaApCallPackage),
    LogonTerminated: Some(LsaApLogonTerminated),
    CallPackageUntrusted: Some(LsaApCallPackageUntrusted),
    CallPackagePassthrough: Some(LsaApCallPackagePassthrough),
    LogonUserExW: None,
    LogonUserEx2: Some(LsaApLogonUserEx2),
    Initialize: Some(SpInitialize),
    Shutdown: Some(LsaSpShutDown),
    GetInfo: Some(LsaSpGetInfoW),
    AcceptCredentials: Some(SpAcceptCredentials),
    SpAcquireCredentialsHandle: Some(LsaSpAcquireCredentialsHandle),
    SpQueryCredentialsAttributes: Some(LsaSpQueryCredentialsAttributes),
    FreeCredentialsHandle: Some(LsaSpFreeCredentialsHandle),
    SaveCredentials: Some(LsaSpSaveCredentials),
    GetCredentials: Some(LsaSpGetCredentials),
    DeleteCredentials: Some(LsaSpDeleteCredentials),
    InitLsaModeContext: Some(LsaSpInitLsaModeContext),
    AcceptLsaModeContext: Some(LsaSpAcceptLsaModeContext),
    DeleteContext: Some(LsaSpDeleteContext),
    ApplyControlToken: Some(LsaSpApplyControlToken),
    GetUserInfo: Some(LsaSpGetUserInfo),
    GetExtendedInformation: Some(LsaSpGetExtendedInformation),
    SpQueryContextAttributes: None,
    SpAddCredentials: None,
    SetExtendedInformation: Some(LsaSpSetExtendedInformation),
    SetContextAttributesW: None,
    SetCredentialsAttributes: None,
    ChangeAccountPassword: None,
    QueryMetaData: None,
    ExchangeMetaData: None,
    GetCredUIContext: None,
    UpdateCredentials: None,
    ValidateTargetInfo: None,
    PostLogonUser: None,
}];

//  .InstanceInit                 SpInstanceInit                  -> Some(SpInstanceInit)
//  .InitUserModeContext          UsrSpInitUserModeContext        -> Some(UsrSpInitUserModeContext)
//  .MakeSignature                UsrSpMakeSignature              -> Some(UsrSpMakeSignature)
//  .VerifySignature              UsrSpVerifySignature            -> Some(UsrSpVerifySignature)
//  .SealMessage                  UsrSpSealMessage                -> Some(UsrSpSealMessage)
//  .UnsealMessage                UsrSpUnsealMessage              -> Some(UsrSpUnsealMessage)
//  .GetContextToken              UsrSpGetContextToken            -> Some(UsrSpGetContextToken)
//  .SpQueryContextAttributes     UsrSpQueryContextAttributes     -> Some(UsrSpQueryContextAttributes)
//  .CompleteAuthToken            UsrSpCompleteAuthToken          -> Some(UsrSpCompleteAuthToken)
//  .DeleteUserModeContext        UsrSpDeleteUserModeContext      -> Some(UsrSpDeleteUserModeContext)
//  .FormatCredentials            UsrSpFormatCredentials          -> Some(UsrSpFormatCredentials)
//  .MarshallSupplementalCreds    UsrSpMarshallSupplementalCreds  -> Some(UsrSpMarshallSupplementalCreds)
//  .ExportContext                UsrSpExportSecurityContext      -> Some(UsrSpExportSecurityContext)
//  .ImportContext                UsrSpImportSecurityContext      -> Some(UsrSpImportSecurityContext)
#[no_mangle]
pub static mut NtlmUsrFn: [SECPKG_USER_FUNCTION_TABLE; 1] = [SECPKG_USER_FUNCTION_TABLE {
    InstanceInit: Some(SpInstanceInit),
    InitUserModeContext: Some(UsrSpInitUserModeContext),
    MakeSignature: Some(UsrSpMakeSignature),
    VerifySignature: Some(UsrSpVerifySignature),
    SealMessage: Some(UsrSpSealMessage),
    UnsealMessage: Some(UsrSpUnsealMessage),
    GetContextToken: Some(UsrSpGetContextToken),
    SpQueryContextAttributes: Some(UsrSpQueryContextAttributes),
    CompleteAuthToken: Some(UsrSpCompleteAuthToken),
    DeleteUserModeContext: Some(UsrSpDeleteUserModeContext),
    FormatCredentials: Some(UsrSpFormatCredentials),
    MarshallSupplementalCreds: Some(UsrSpMarshallSupplementalCreds),
    ExportContext: Some(UsrSpExportSecurityContext),
    ImportContext: Some(UsrSpImportSecurityContext),
}];
