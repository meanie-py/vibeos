//! Per-crate write barrier for se-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` +
//! pulled into the link via kernel-rust's `extern crate se_rust`. Relocated from
//! ntoskrnl-rust/src/global_barrier.rs during the crate-split (Phase 1b).
//!
//! Set = the 58 mutable Se*/Sep* globals that had barrier writes in the monolith
//! (transcribed 1:1, `se::` segment dropped). The 34 const LUIDs in priv_.rs are
//! NON-mut `pub static` and were deliberately NOT in the monolith barrier — they
//! stay out here too (recipe §6).
//!
//! Hand-written for the crate-split. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches se-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _se_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    // acl.rs (7)
    write_volatile(addr_of_mut!(crate::acl::SePublicDefaultDacl), read_volatile(addr_of!(crate::acl::SePublicDefaultDacl)));
    write_volatile(addr_of_mut!(crate::acl::SeSystemDefaultDacl), read_volatile(addr_of!(crate::acl::SeSystemDefaultDacl)));
    write_volatile(addr_of_mut!(crate::acl::SePublicDefaultUnrestrictedDacl), read_volatile(addr_of!(crate::acl::SePublicDefaultUnrestrictedDacl)));
    write_volatile(addr_of_mut!(crate::acl::SePublicOpenDacl), read_volatile(addr_of!(crate::acl::SePublicOpenDacl)));
    write_volatile(addr_of_mut!(crate::acl::SePublicOpenUnrestrictedDacl), read_volatile(addr_of!(crate::acl::SePublicOpenUnrestrictedDacl)));
    write_volatile(addr_of_mut!(crate::acl::SeUnrestrictedDacl), read_volatile(addr_of!(crate::acl::SeUnrestrictedDacl)));
    write_volatile(addr_of_mut!(crate::acl::SeSystemAnonymousLogonDacl), read_volatile(addr_of!(crate::acl::SeSystemAnonymousLogonDacl)));
    // sd.rs (7)
    write_volatile(addr_of_mut!(crate::sd::SePublicDefaultSd), read_volatile(addr_of!(crate::sd::SePublicDefaultSd)));
    write_volatile(addr_of_mut!(crate::sd::SePublicDefaultUnrestrictedSd), read_volatile(addr_of!(crate::sd::SePublicDefaultUnrestrictedSd)));
    write_volatile(addr_of_mut!(crate::sd::SePublicOpenSd), read_volatile(addr_of!(crate::sd::SePublicOpenSd)));
    write_volatile(addr_of_mut!(crate::sd::SePublicOpenUnrestrictedSd), read_volatile(addr_of!(crate::sd::SePublicOpenUnrestrictedSd)));
    write_volatile(addr_of_mut!(crate::sd::SeSystemDefaultSd), read_volatile(addr_of!(crate::sd::SeSystemDefaultSd)));
    write_volatile(addr_of_mut!(crate::sd::SeUnrestrictedSd), read_volatile(addr_of!(crate::sd::SeUnrestrictedSd)));
    write_volatile(addr_of_mut!(crate::sd::SeSystemAnonymousLogonSd), read_volatile(addr_of!(crate::sd::SeSystemAnonymousLogonSd)));
    // semgr.rs (4)
    write_volatile(addr_of_mut!(crate::semgr::SeAnonymousLogonToken), read_volatile(addr_of!(crate::semgr::SeAnonymousLogonToken)));
    write_volatile(addr_of_mut!(crate::semgr::SeAnonymousLogonTokenNoEveryone), read_volatile(addr_of!(crate::semgr::SeAnonymousLogonTokenNoEveryone)));
    write_volatile(addr_of_mut!(crate::semgr::SeExports), read_volatile(addr_of!(crate::semgr::SeExports)));
    write_volatile(addr_of_mut!(crate::semgr::SidInTokenCalls), read_volatile(addr_of!(crate::semgr::SidInTokenCalls)));
    // sid.rs (35: 5 SID_IDENTIFIER_AUTHORITY + 30 PSID)
    write_volatile(addr_of_mut!(crate::sid::SeNullSidAuthority), read_volatile(addr_of!(crate::sid::SeNullSidAuthority)));
    write_volatile(addr_of_mut!(crate::sid::SeWorldSidAuthority), read_volatile(addr_of!(crate::sid::SeWorldSidAuthority)));
    write_volatile(addr_of_mut!(crate::sid::SeLocalSidAuthority), read_volatile(addr_of!(crate::sid::SeLocalSidAuthority)));
    write_volatile(addr_of_mut!(crate::sid::SeCreatorSidAuthority), read_volatile(addr_of!(crate::sid::SeCreatorSidAuthority)));
    write_volatile(addr_of_mut!(crate::sid::SeNtSidAuthority), read_volatile(addr_of!(crate::sid::SeNtSidAuthority)));
    write_volatile(addr_of_mut!(crate::sid::SeNullSid), read_volatile(addr_of!(crate::sid::SeNullSid)));
    write_volatile(addr_of_mut!(crate::sid::SeWorldSid), read_volatile(addr_of!(crate::sid::SeWorldSid)));
    write_volatile(addr_of_mut!(crate::sid::SeLocalSid), read_volatile(addr_of!(crate::sid::SeLocalSid)));
    write_volatile(addr_of_mut!(crate::sid::SeCreatorOwnerSid), read_volatile(addr_of!(crate::sid::SeCreatorOwnerSid)));
    write_volatile(addr_of_mut!(crate::sid::SeCreatorGroupSid), read_volatile(addr_of!(crate::sid::SeCreatorGroupSid)));
    write_volatile(addr_of_mut!(crate::sid::SeCreatorOwnerServerSid), read_volatile(addr_of!(crate::sid::SeCreatorOwnerServerSid)));
    write_volatile(addr_of_mut!(crate::sid::SeCreatorGroupServerSid), read_volatile(addr_of!(crate::sid::SeCreatorGroupServerSid)));
    write_volatile(addr_of_mut!(crate::sid::SeNtAuthoritySid), read_volatile(addr_of!(crate::sid::SeNtAuthoritySid)));
    write_volatile(addr_of_mut!(crate::sid::SeDialupSid), read_volatile(addr_of!(crate::sid::SeDialupSid)));
    write_volatile(addr_of_mut!(crate::sid::SeNetworkSid), read_volatile(addr_of!(crate::sid::SeNetworkSid)));
    write_volatile(addr_of_mut!(crate::sid::SeBatchSid), read_volatile(addr_of!(crate::sid::SeBatchSid)));
    write_volatile(addr_of_mut!(crate::sid::SeInteractiveSid), read_volatile(addr_of!(crate::sid::SeInteractiveSid)));
    write_volatile(addr_of_mut!(crate::sid::SeServiceSid), read_volatile(addr_of!(crate::sid::SeServiceSid)));
    write_volatile(addr_of_mut!(crate::sid::SePrincipalSelfSid), read_volatile(addr_of!(crate::sid::SePrincipalSelfSid)));
    write_volatile(addr_of_mut!(crate::sid::SeLocalSystemSid), read_volatile(addr_of!(crate::sid::SeLocalSystemSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAuthenticatedUserSid), read_volatile(addr_of!(crate::sid::SeAuthenticatedUserSid)));
    write_volatile(addr_of_mut!(crate::sid::SeRestrictedCodeSid), read_volatile(addr_of!(crate::sid::SeRestrictedCodeSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasAdminsSid), read_volatile(addr_of!(crate::sid::SeAliasAdminsSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasUsersSid), read_volatile(addr_of!(crate::sid::SeAliasUsersSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasGuestsSid), read_volatile(addr_of!(crate::sid::SeAliasGuestsSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasPowerUsersSid), read_volatile(addr_of!(crate::sid::SeAliasPowerUsersSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasAccountOpsSid), read_volatile(addr_of!(crate::sid::SeAliasAccountOpsSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasSystemOpsSid), read_volatile(addr_of!(crate::sid::SeAliasSystemOpsSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasPrintOpsSid), read_volatile(addr_of!(crate::sid::SeAliasPrintOpsSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAliasBackupOpsSid), read_volatile(addr_of!(crate::sid::SeAliasBackupOpsSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAuthenticatedUsersSid), read_volatile(addr_of!(crate::sid::SeAuthenticatedUsersSid)));
    write_volatile(addr_of_mut!(crate::sid::SeRestrictedSid), read_volatile(addr_of!(crate::sid::SeRestrictedSid)));
    write_volatile(addr_of_mut!(crate::sid::SeAnonymousLogonSid), read_volatile(addr_of!(crate::sid::SeAnonymousLogonSid)));
    write_volatile(addr_of_mut!(crate::sid::SeLocalServiceSid), read_volatile(addr_of!(crate::sid::SeLocalServiceSid)));
    write_volatile(addr_of_mut!(crate::sid::SeNetworkServiceSid), read_volatile(addr_of!(crate::sid::SeNetworkServiceSid)));
    // subject.rs (1)
    write_volatile(addr_of_mut!(crate::subject::SepSubjectContextLock), read_volatile(addr_of!(crate::subject::SepSubjectContextLock)));
    // token.rs (4)
    write_volatile(addr_of_mut!(crate::token::SeTokenObjectType), read_volatile(addr_of!(crate::token::SeTokenObjectType)));
    write_volatile(addr_of_mut!(crate::token::SeSystemTokenSource), read_volatile(addr_of!(crate::token::SeSystemTokenSource)));
    write_volatile(addr_of_mut!(crate::token::SeSystemAuthenticationId), read_volatile(addr_of!(crate::token::SeSystemAuthenticationId)));
    write_volatile(addr_of_mut!(crate::token::SeAnonymousAuthenticationId), read_volatile(addr_of!(crate::token::SeAnonymousAuthenticationId)));
}
