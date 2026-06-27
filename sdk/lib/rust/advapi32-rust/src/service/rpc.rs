use crate::prelude::*;

// service/rpc.c — the MIDL RPC allocator pair. The C names are lowercase
// midl_user_allocate/free but rpcndr.h `#define`s them to the uppercase
// MIDL_user_* (the actual exported __RPC_USER = stdcall symbols the generated
// *_c.c client stubs call). HEAP_ZERO_MEMORY = 0x8 (winnt.h).
const HEAP_ZERO_MEMORY: DWORD = 0x0000_0008;

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
