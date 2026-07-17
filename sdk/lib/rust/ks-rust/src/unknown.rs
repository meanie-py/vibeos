use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::{GUID};
use core::ffi::c_void;

#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut vt_INonDelegatedUnknownVtbl: INonDelegatedUnknownVtbl = unsafe {
    INonDelegatedUnknownVtbl {
        NonDelegatedQueryInterface: core::mem::transmute(
            INonDelegatedUnknown_fnQueryInterface
                as unsafe extern "system" fn(*mut INonDelegatedUnknown, *const IID, *mut PVOID) -> NTSTATUS,
        ),
        NonDelegatedAddRef: core::mem::transmute(
            INonDelegatedUnknown_fnAddRef as unsafe extern "system" fn(*mut INonDelegatedUnknown) -> ULONG,
        ),
        NonDelegatedRelease: core::mem::transmute(
            INonDelegatedUnknown_fnRelease as unsafe extern "system" fn(*mut INonDelegatedUnknown) -> ULONG,
        ),
    }
};

static mut vt_IIndirectedUnknownVtbl: IIndirectedUnknownVtbl = unsafe {
    IIndirectedUnknownVtbl {
        IndirectedQueryInterface: core::mem::transmute(
            IIndirectedUnknown_fnQueryInterface
                as unsafe extern "system" fn(*mut IIndirectedUnknown, *const IID, *mut *mut core::ffi::c_void) -> NTSTATUS,
        ),
        IndirectedAddRef: core::mem::transmute(
            IIndirectedUnknown_fnAddRef as unsafe extern "system" fn(*mut IIndirectedUnknown) -> ULONG,
        ),
        IndirectedRelease: core::mem::transmute(
            IIndirectedUnknown_fnRelease as unsafe extern "system" fn(*mut IIndirectedUnknown) -> ULONG,
        ),
    }
};

#[repr(C)]
pub struct CBaseUnknownImpl {
    pub lpVtbl: *mut INonDelegatedUnknownVtbl,
    pub lpVtblIndirectedUnknown: *mut IIndirectedUnknownVtbl,
    pub m_RefCount: LONG,
    pub m_UsingClassId: BOOLEAN,
    pub m_ClassId: CLSID,
    pub m_UnknownOuter: *mut IUnknown,
}

#[no_mangle]
pub unsafe extern "system" fn INonDelegatedUnknown_fnQueryInterface(
    iface: *mut INonDelegatedUnknown,
    refiid: *const IID,
    Output: *mut PVOID,
) -> NTSTATUS {
    let This: *mut CBaseUnknownImpl = (iface as usize)
        .wrapping_sub(core::mem::offset_of!(CBaseUnknownImpl, lpVtbl))
        as *mut CBaseUnknownImpl;

    if IsEqualGUIDAligned(refiid, &IID_IUnknown) {
        *Output = core::ptr::addr_of_mut!((*This).lpVtbl) as PVOID;
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).m_RefCount));
        return STATUS_SUCCESS;
    }
    return STATUS_NOINTERFACE;
}

#[no_mangle]
pub unsafe extern "system" fn INonDelegatedUnknown_fnAddRef(
    iface: *mut INonDelegatedUnknown,
) -> ULONG {
    let This = (iface as usize)
        .wrapping_sub(core::mem::offset_of!(CBaseUnknownImpl, lpVtbl))
        as *mut CBaseUnknownImpl;
    InterlockedIncrement(&mut (*This).m_RefCount) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn INonDelegatedUnknown_fnRelease(
    iface: *mut INonDelegatedUnknown,
) -> ULONG {
    let This = CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtbl);
    InterlockedDecrement(&mut (*This).m_RefCount);
    return (*This).m_RefCount as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn IIndirectedUnknown_fnQueryInterface(
    iface: *mut IIndirectedUnknown,
    refiid: *const IID,
    Output: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let This = (iface as usize)
        .wrapping_sub(core::mem::offset_of!(CBaseUnknownImpl, lpVtblIndirectedUnknown))
        as *mut CBaseUnknownImpl;
    (*((*(*This).m_UnknownOuter).lpVtbl))
        .QueryInterface
        .expect("non-null function pointer")((*This).m_UnknownOuter, refiid, Output)
}

#[no_mangle]
pub unsafe extern "system" fn IIndirectedUnknown_fnAddRef(
    iface: *mut IIndirectedUnknown,
) -> ULONG {
    let This = CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
    return ((*((*(*This).m_UnknownOuter).lpVtbl)).AddRef.expect("non-null function pointer"))((*This).m_UnknownOuter);
}

#[no_mangle]
pub unsafe extern "system" fn IIndirectedUnknown_fnRelease(
    iface: *mut IIndirectedUnknown,
) -> ULONG {
    let This = ((iface as usize)
        - core::mem::offset_of!(CBaseUnknownImpl, lpVtblIndirectedUnknown))
        as *mut CBaseUnknownImpl;
    let outer = (*This).m_UnknownOuter;
    ((*(*outer).lpVtbl).Release.expect("non-null function pointer"))(outer)
}

// CBaseUnknown_* are ks.spec exports. On i386, KS_DECL_CXX = `__fastcall` and KS_THIS expands
// to `CLASS* This, void* dummy_` -> the i386 exports are fastcall with an extra dummy arg
// (decorations @16/@12/@12/@8). On amd64/arm64, KS_DECL_CXX = `__cdecl` and KS_THIS = just
// `This`. The bodies ignore dummy_, so each is a shared Rust-ABI impl with thin per-arch
// #[no_mangle] export wrappers (cfg-gated) that produce the exact decorated symbol ks.def needs.

unsafe fn cbu_ctor_guid_impl(
    This: *mut CBaseUnknownImpl,
    lpGUID: *const GUID,
    OuterUnknown: *mut IUnknown,
) -> *mut CBaseUnknownImpl {
    (*This).lpVtbl = core::ptr::addr_of_mut!(vt_INonDelegatedUnknownVtbl);
    (*This).lpVtblIndirectedUnknown = core::ptr::addr_of_mut!(vt_IIndirectedUnknownVtbl);
    (*This).m_UsingClassId = 1u8;
    core::ptr::copy_nonoverlapping(
        lpGUID as *const u8,
        core::ptr::addr_of_mut!((*This).m_ClassId) as *mut u8,
        core::mem::size_of::<GUID>(),
    );
    (*This).m_RefCount = 0i32;
    if !OuterUnknown.is_null() {
        (*This).m_UnknownOuter = OuterUnknown;
    } else {
        (*This).m_UnknownOuter = core::ptr::addr_of_mut!((*This).lpVtbl) as *mut IUnknown;
    }
    This
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CBaseUnknown_ConstructorWithGUID(
    This: *mut CBaseUnknownImpl,
    _dummy: *mut core::ffi::c_void,
    lpGUID: *const GUID,
    OuterUnknown: *mut IUnknown,
) -> *mut CBaseUnknownImpl {
    cbu_ctor_guid_impl(This, lpGUID, OuterUnknown)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_ConstructorWithGUID(
    This: *mut CBaseUnknownImpl,
    lpGUID: *const GUID,
    OuterUnknown: *mut IUnknown,
) -> *mut CBaseUnknownImpl {
    cbu_ctor_guid_impl(This, lpGUID, OuterUnknown)
}

unsafe fn cbu_ctor_impl(
    This: *mut CBaseUnknownImpl,
    OuterUnknown: *mut IUnknown,
) -> *mut CBaseUnknownImpl {
    (*This).lpVtbl = &mut vt_INonDelegatedUnknownVtbl;
    (*This).lpVtblIndirectedUnknown = &mut vt_IIndirectedUnknownVtbl;
    (*This).m_UsingClassId = 0u8;
    (*This).m_RefCount = 0i32;
    if !OuterUnknown.is_null() {
        (*This).m_UnknownOuter = OuterUnknown;
    } else {
        (*This).m_UnknownOuter = &mut (*This).lpVtbl as *mut _ as *mut IUnknown;
    }
    This
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CBaseUnknown_Constructor(
    This: *mut CBaseUnknownImpl,
    _dummy: *mut core::ffi::c_void,
    OuterUnknown: *mut IUnknown,
) -> *mut CBaseUnknownImpl {
    cbu_ctor_impl(This, OuterUnknown)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_Constructor(
    This: *mut CBaseUnknownImpl,
    OuterUnknown: *mut IUnknown,
) -> *mut CBaseUnknownImpl {
    cbu_ctor_impl(This, OuterUnknown)
}

unsafe fn cbu_dtor_impl(This: *mut CBaseUnknownImpl, OuterUnknown: *mut IUnknown) {
    (*This).lpVtbl = &raw mut vt_INonDelegatedUnknownVtbl;
    (*This).lpVtblIndirectedUnknown = &raw mut vt_IIndirectedUnknownVtbl;
    if !OuterUnknown.is_null() {
        (*This).m_UnknownOuter = OuterUnknown;
    } else {
        (*This).m_UnknownOuter = &raw mut (*This).lpVtbl as *mut IUnknown;
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CBaseUnknown_Destructor(
    This: *mut CBaseUnknownImpl,
    _dummy: *mut core::ffi::c_void,
    OuterUnknown: *mut IUnknown,
) {
    cbu_dtor_impl(This, OuterUnknown)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_Destructor(
    This: *mut CBaseUnknownImpl,
    OuterUnknown: *mut IUnknown,
) {
    cbu_dtor_impl(This, OuterUnknown)
}

unsafe fn cbu_default_dtor_impl(This: *mut CBaseUnknownImpl) {
    (*This).lpVtbl = core::ptr::addr_of_mut!(vt_INonDelegatedUnknownVtbl);
    (*This).lpVtblIndirectedUnknown = core::ptr::addr_of_mut!(vt_IIndirectedUnknownVtbl);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CBaseUnknown_DefaultDestructor(
    This: *mut CBaseUnknownImpl,
    _dummy: *mut core::ffi::c_void,
) {
    cbu_default_dtor_impl(This)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_DefaultDestructor(This: *mut CBaseUnknownImpl) {
    cbu_default_dtor_impl(This)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/topoology.c
//  * PURPOSE:         KS CBaseUnknown functions
//  * PROGRAMMER:      Johannes Anderwald
//  *                  KJK::Hyperion
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #ifdef _X86_
// #define KS_DECL_CXX(RET_) extern RET_ __fastcall
// #define KS_THIS(CLASS_)   CLASS_ * This, void * dummy_
// #else
// #define KS_DECL_CXX(RET_) extern RET_ __cdecl
// #define KS_THIS(CLASS_)   CLASS_ * This
// #endif
//
// typedef struct
// {
//     INonDelegatedUnknownVtbl *lpVtbl;
//     IIndirectedUnknownVtbl *lpVtblIndirectedUnknown;
//
//     LONG m_RefCount;
//
//     BOOLEAN m_UsingClassId;
//     CLSID m_ClassId;
//     IUnknown* m_UnknownOuter;
//
// }CBaseUnknownImpl;
//
//
//
// NTSTATUS
// NTAPI
// INonDelegatedUnknown_fnQueryInterface(
//     INonDelegatedUnknown * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtbl);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown))
//     {
//         *Output = &This->lpVtbl;
//         _InterlockedIncrement(&This->m_RefCount);
//         return STATUS_SUCCESS;
//     }
//     return STATUS_NOINTERFACE;
// }
//
// ULONG
// NTAPI
// INonDelegatedUnknown_fnAddRef(
//     INonDelegatedUnknown * iface)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtbl);
//
//     return InterlockedIncrement(&This->m_RefCount);
// }
//
// ULONG
// NTAPI
// INonDelegatedUnknown_fnRelease(
//     INonDelegatedUnknown * iface)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtbl);
//
//     InterlockedDecrement(&This->m_RefCount);
//
//     /* Return new reference count */
//     return This->m_RefCount;
// }
//
// static INonDelegatedUnknownVtbl vt_INonDelegatedUnknownVtbl =
// {
//     INonDelegatedUnknown_fnQueryInterface,
//     INonDelegatedUnknown_fnAddRef,
//     INonDelegatedUnknown_fnRelease
// };
//
// NTSTATUS
// NTAPI
// IIndirectedUnknown_fnQueryInterface(
//     IIndirectedUnknown * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
//
//     return This->m_UnknownOuter->lpVtbl->QueryInterface(This->m_UnknownOuter, refiid, Output);
// }
//
// ULONG
// NTAPI
// IIndirectedUnknown_fnAddRef(
//     IIndirectedUnknown * iface)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
//
//     return This->m_UnknownOuter->lpVtbl->AddRef(This->m_UnknownOuter);
// }
//
// ULONG
// NTAPI
// IIndirectedUnknown_fnRelease(
//     IIndirectedUnknown * iface)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD!(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
//
//     return This->m_UnknownOuter->lpVtbl->Release(This->m_UnknownOuter);
// }
//
// static IIndirectedUnknownVtbl vt_IIndirectedUnknownVtbl =
// {
//     IIndirectedUnknown_fnQueryInterface,
//     IIndirectedUnknown_fnAddRef,
//     IIndirectedUnknown_fnRelease
// };
//
//
// KS_DECL_CXX(CBaseUnknownImpl *) CBaseUnknown_ConstructorWithGUID(KS_THIS(CBaseUnknownImpl), const GUID *lpGUID, IUnknown * OuterUnknown)
// {
//
//     This->lpVtbl = &vt_INonDelegatedUnknownVtbl;
//     This->lpVtblIndirectedUnknown = &vt_IIndirectedUnknownVtbl;
//
//     /* class uses class id */
//     This->m_UsingClassId = TRUE;
//
//     /* copy guid */
//     RtlMoveMemory(&This->m_ClassId, lpGUID, sizeof(GUID));
//
//     /* set refcount to zero */
//     This->m_RefCount = 0;
//
//     if (OuterUnknown)
//     {
//         /* use outer unknown */
//         This->m_UnknownOuter = OuterUnknown;
//     }
//     else
//     {
//         /* use unknown from INonDelegatedUnknown */
//         This->m_UnknownOuter = (PUNKNOWN)&This->lpVtbl;
//     }
//
//     /* return result */
//     return This;
// }
//
// KS_DECL_CXX(CBaseUnknownImpl *) CBaseUnknown_Constructor(KS_THIS(CBaseUnknownImpl), IUnknown * OuterUnknown)
// {
//
//     This->lpVtbl = &vt_INonDelegatedUnknownVtbl;
//     This->lpVtblIndirectedUnknown = &vt_IIndirectedUnknownVtbl;
//
//     /* class uses class id */
//     This->m_UsingClassId = FALSE;
//
//     /* set refcount to zero */
//     This->m_RefCount = 0;
//
//     if (OuterUnknown)
//     {
//         /* use outer unknown */
//         This->m_UnknownOuter = OuterUnknown;
//     }
//     else
//     {
//         /* use unknown from INonDelegatedUnknown */
//         This->m_UnknownOuter = (PUNKNOWN)&This->lpVtbl;
//     }
//
//     /* return result */
//     return This;
// }
//
// KS_DECL_CXX(VOID) CBaseUnknown_Destructor(KS_THIS(CBaseUnknownImpl), IUnknown * OuterUnknown)
// {
//     /* restore vtbl's */
//     This->lpVtbl = &vt_INonDelegatedUnknownVtbl;
//     This->lpVtblIndirectedUnknown = &vt_IIndirectedUnknownVtbl;
//
//
//     if (OuterUnknown)
//     {
//         /* use outer unknown */
//         This->m_UnknownOuter = OuterUnknown;
//     }
//     else
//     {
//         /* use unknown from INonDelegatedUnknown */
//         This->m_UnknownOuter = (PUNKNOWN)&This->lpVtbl;
//     }
// }
//
// KS_DECL_CXX(VOID) CBaseUnknown_DefaultDestructor(KS_THIS(CBaseUnknownImpl))
// {
//     /* restore vtbl's */
//     This->lpVtbl = &vt_INonDelegatedUnknownVtbl;
//     This->lpVtblIndirectedUnknown = &vt_IIndirectedUnknownVtbl;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
