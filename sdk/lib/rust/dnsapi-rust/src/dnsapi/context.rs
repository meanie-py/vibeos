use crate::prelude::*;
use core::ffi::c_void;


// ---- DnsReleaseContextHandle ----
#[no_mangle]
pub unsafe extern "system" fn DnsReleaseContextHandle(ContextHandle: HANDLE) {
    let Context = ContextHandle as *mut WINDNS_CONTEXT;
    adns_finish((*Context).State);
    RtlFreeHeap(RtlGetProcessHeap(), 0, Context as *mut core::ffi::c_void);
}

// ---- DnsAcquireContextHandle_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsAcquireContextHandle_W(
    CredentialsFlags: DWORD,
    Credentials: PVOID,
    ContextHandle: *mut HANDLE,
) -> DNS_STATUS {
    if CredentialsFlags != 0 {
        let Context = RtlAllocateHeap(RtlGetProcessHeap(), 0, core::mem::size_of::<WINDNS_CONTEXT>()) as *mut WINDNS_CONTEXT;
        if Context.is_null() {
            *ContextHandle = core::ptr::null_mut();
            return ERROR_OUTOFMEMORY;
        }
        let adns_status = adns_init(&mut (*Context).State, adns_if_noenv | adns_if_noerrprint | adns_if_noserverwarn, core::ptr::null_mut());
        if adns_status != adns_s_ok {
            *ContextHandle = core::ptr::null_mut();
            RtlFreeHeap(RtlGetProcessHeap(), 0, Context as PVOID);
            return DnsIntTranslateAdnsToDNS_STATUS(adns_status);
        } else {
            *ContextHandle = Context as HANDLE;
            return ERROR_SUCCESS;
        }
    } else {
        return DnsAcquireContextHandle_UTF8(CredentialsFlags, Credentials, ContextHandle);
    }
}

// ---- DnsAcquireContextHandle_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsAcquireContextHandle_UTF8(
    CredentialsFlags: DWORD,
    Credentials: PVOID,
    ContextHandle: *mut HANDLE,
) -> DNS_STATUS {
    if CredentialsFlags != 0 {
        return DnsAcquireContextHandle_W(CredentialsFlags, Credentials, ContextHandle);
    } else {
        let Status = DnsAcquireContextHandle_W(1u32, Credentials, ContextHandle);
        return Status;
    }
}

// ---- DnsAcquireContextHandle_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsAcquireContextHandle_A(
    CredentialFlags: DWORD,
    Credentials: PVOID,
    ContextHandle: *mut HANDLE,
) -> DNS_STATUS {
    if CredentialFlags != 0 {
        DnsAcquireContextHandle_W(CredentialFlags, Credentials, ContextHandle)
    } else {
        DnsAcquireContextHandle_UTF8(CredentialFlags, Credentials, ContextHandle)
    }
}
