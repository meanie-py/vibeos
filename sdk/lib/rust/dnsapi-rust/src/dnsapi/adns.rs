use crate::prelude::*;
use core::ffi::c_void;


// ---- DnsIntTranslateAdnsToDNS_STATUS ----
pub unsafe fn DnsIntTranslateAdnsToDNS_STATUS(Status: i32) -> DNS_STATUS {
    match Status {
        adns_s_ok => ERROR_SUCCESS,
        adns_s_nomemory | adns_s_systemfail => ERROR_OUTOFMEMORY,
        _ => ERROR_OUTOFMEMORY,
    }
}
