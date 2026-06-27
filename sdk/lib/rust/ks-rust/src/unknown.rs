use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::{GUID};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut vt_INonDelegatedUnknownVtbl: INonDelegatedUnknownVtbl = unsafe {
    INonDelegatedUnknownVtbl {
        NonDelegatedQueryInterface: Some(
            INonDelegatedUnknown_fnQueryInterface
                as unsafe extern "C" fn(
                    *mut INonDelegatedUnknown,
                    *const IID,
                    *mut PVOID,
                ) -> NTSTATUS,
        ),
        NonDelegatedAddRef: Some(
            INonDelegatedUnknown_fnAddRef
                as unsafe extern "C" fn(*mut INonDelegatedUnknown) -> ULONG,
        ),
        NonDelegatedRelease: Some(
            INonDelegatedUnknown_fnRelease
                as unsafe extern "C" fn(*mut INonDelegatedUnknown) -> ULONG,
        ),
    }
};

static mut vt_IIndirectedUnknownVtbl: IIndirectedUnknownVtbl = unsafe {
    IIndirectedUnknownVtbl {
        IndirectedQueryInterface: Some(
            IIndirectedUnknown_fnQueryInterface
                as unsafe extern "C" fn(
                    *mut IIndirectedUnknown,
                    *const IID,
                    *mut PVOID,
                ) -> NTSTATUS,
        ),
        IndirectedAddRef: Some(
            IIndirectedUnknown_fnAddRef as unsafe extern "C" fn(*mut IIndirectedUnknown) -> ULONG,
        ),
        IndirectedRelease: Some(
            IIndirectedUnknown_fnRelease as unsafe extern "C" fn(*mut IIndirectedUnknown) -> ULONG,
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

    if IsEqualGUIDAligned(refiid, &IID_IUnknown) != 0 {
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
        .wrapping_sub(core::ptr::addr_of!((*core::ptr::null_mut::<CBaseUnknownImpl>()).lpVtbl) as usize)
        as *mut CBaseUnknownImpl;
    InterlockedIncrement(&mut (*This).m_RefCount) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn INonDelegatedUnknown_fnRelease(
    iface: *mut INonDelegatedUnknown,
) -> ULONG {
    let This = CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtbl);
    InterlockedDecrement(&mut (*This).m_RefCount);
    return (*This).m_RefCount;
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
    ((*(*This).m_UnknownOuter).lpVtbl)
        .QueryInterface
        .expect("non-null function pointer")((*This).m_UnknownOuter, refiid, Output)
}

#[no_mangle]
pub unsafe extern "system" fn IIndirectedUnknown_fnAddRef(
    iface: *mut IIndirectedUnknown,
) -> ULONG {
    let This = CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
    return ((*(*This).m_UnknownOuter).lpVtbl.AddRef)((*This).m_UnknownOuter);
}

#[no_mangle]
pub unsafe extern "system" fn IIndirectedUnknown_fnRelease(
    iface: *mut IIndirectedUnknown,
) -> ULONG {
    let This = ((iface as usize)
        - core::mem::offset_of!(CBaseUnknownImpl, lpVtblIndirectedUnknown))
        as *mut CBaseUnknownImpl;
    let outer = (*This).m_UnknownOuter;
    ((*(*outer).lpVtbl).Release)(outer)
}

#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_ConstructorWithGUID(
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

#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_Constructor(
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
    return This;
}
#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_Destructor(
    This: *mut CBaseUnknownImpl,
    OuterUnknown: *mut IUnknown,
) {
    (*This).lpVtbl = &raw mut vt_INonDelegatedUnknownVtbl;
    (*This).lpVtblIndirectedUnknown = &raw mut vt_IIndirectedUnknownVtbl;
    if !OuterUnknown.is_null() {
        (*This).m_UnknownOuter = OuterUnknown;
    } else {
        (*This).m_UnknownOuter = &raw mut (*This).lpVtbl as *mut IUnknown;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CBaseUnknown_DefaultDestructor(This: *mut CBaseUnknownImpl) {
    (*This).lpVtbl = core::ptr::addr_of_mut!(vt_INonDelegatedUnknownVtbl);
    (*This).lpVtblIndirectedUnknown = core::ptr::addr_of_mut!(vt_IIndirectedUnknownVtbl);
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
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtbl);
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
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtbl);
//
//     return InterlockedIncrement(&This->m_RefCount);
// }
//
// ULONG
// NTAPI
// INonDelegatedUnknown_fnRelease(
//     INonDelegatedUnknown * iface)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtbl);
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
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
//
//     return This->m_UnknownOuter->lpVtbl->QueryInterface(This->m_UnknownOuter, refiid, Output);
// }
//
// ULONG
// NTAPI
// IIndirectedUnknown_fnAddRef(
//     IIndirectedUnknown * iface)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
//
//     return This->m_UnknownOuter->lpVtbl->AddRef(This->m_UnknownOuter);
// }
//
// ULONG
// NTAPI
// IIndirectedUnknown_fnRelease(
//     IIndirectedUnknown * iface)
// {
//     CBaseUnknownImpl * This = (CBaseUnknownImpl*)CONTAINING_RECORD(iface, CBaseUnknownImpl, lpVtblIndirectedUnknown);
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
