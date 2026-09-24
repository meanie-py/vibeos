use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::rtl::RtlCopyMemory;

// GDI metafile type tags — multichar DWORD constants from win32ss/include/ntgdityp.h
// (C multichar literals are big-endian): 'XEFM' / '_PFM'.
const GDITAG_TYPE_EMF: DWORD = u32::from_be_bytes(*b"XEFM");
const GDITAG_TYPE_MFP: DWORD = u32::from_be_bytes(*b"_PFM");

// SafeProbeForRead/Write: NTAPI (extern "system" = stdcall i386 / cdecl else), Length = SIZE_T.
extern "system" {
    fn SafeProbeForRead(Address: LPVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: LPVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _METATYPEOBJ {
    pub BaseObject: BASEOBJECT,
    pub iType: DWORD,
    pub mm: DWORD,
    pub xExt: DWORD,
    pub yExt: DWORD,
    pub cjData: ULONG,
    pub pjData: [PBYTE; 4],
}
pub type METATYPEOBJ = _METATYPEOBJ;
pub type PMETATYPEOBJ = *mut _METATYPEOBJ;
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateMetafileDC(hdc: HDC) -> HDC {
    GreCreateCompatibleDC(hdc, 1)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateServerMetaFile(
    iType: DWORD,
    cjData: ULONG,
    pjData: PBYTE,
    mm: DWORD,
    xExt: DWORD,
    yExt: DWORD,
) -> HANDLE {
    let mut Pass: BOOL = 1;
    let pmto: PMETATYPEOBJ;
    let Status: NTSTATUS;

    if (iType == GDITAG_TYPE_EMF || iType == GDITAG_TYPE_MFP) && cjData != 0 && !pjData.is_null() {
        pmto = GDIOBJ_AllocObjWithHandle(
            GDILoObjType_LO_META_TYPE as _,
            (core::mem::size_of::<METATYPEOBJ>() as ULONG + cjData) as _,
        ) as PMETATYPEOBJ;
        if !pmto.is_null() {
            (*pmto).iType = iType;
            (*pmto).mm = mm;
            (*pmto).xExt = xExt;
            (*pmto).yExt = yExt;
            (*pmto).cjData = cjData;

            Status = SafeProbeForRead(pjData as _, cjData as _, 1);
            if !NT_SUCCESS(Status) {
                Pass = 0;
            } else {
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*pmto).pjData) as _,
                    pjData as _,
                    cjData as _,
                );
            }

            if Pass != 0 {
                GDIOBJ_vSetObjectOwner(
                    core::ptr::addr_of_mut!((*pmto).BaseObject) as _,
                    GDI_OBJ_HMGR_PUBLIC as _,
                );
                GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*pmto).BaseObject) as _);
                return (*pmto).BaseObject.hHmgr;
            } else {
                GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pmto).BaseObject) as _);
            }
        }
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetServerMetaFileBits(
    hmo: HANDLE,
    cjData: ULONG,
    pjData: PBYTE,
    piType: PDWORD,
    pmm: PDWORD,
    pxExt: PDWORD,
    pyExt: PDWORD,
) -> ULONG {
    let mut cjRet: ULONG = 0;
    let pmto: PMETATYPEOBJ;
    let mut Status: NTSTATUS;

    pmto = GDIOBJ_LockObject(hmo, GDIObjType_META_TYPE as u8) as PMETATYPEOBJ;

    if pmto.is_null() {
        return 0;
    }

    if (*pmto).iType == GDITAG_TYPE_EMF || (*pmto).iType == GDITAG_TYPE_MFP {
        cjRet = (*pmto).cjData;

        if cjData != 0 {
            if cjData == (*pmto).cjData {
                Status = SafeProbeForWrite(piType as *mut core::ffi::c_void, core::mem::size_of::<DWORD>() as _, 1);
                if NT_SUCCESS(Status) {
                    Status = SafeProbeForWrite(pmm as *mut core::ffi::c_void, core::mem::size_of::<DWORD>() as _, 1);
                }
                if NT_SUCCESS(Status) {
                    Status = SafeProbeForWrite(pxExt as *mut core::ffi::c_void, core::mem::size_of::<DWORD>() as _, 1);
                }
                if NT_SUCCESS(Status) {
                    Status = SafeProbeForWrite(pyExt as *mut core::ffi::c_void, core::mem::size_of::<DWORD>() as _, 1);
                }
                if NT_SUCCESS(Status) {
                    Status = SafeProbeForWrite(pjData as *mut core::ffi::c_void, cjData as _, 1);
                }
                if !NT_SUCCESS(Status) {
                    cjRet = 0;
                } else {
                    // Probes are validate-only (USE_DUMMY_PSEH); the C had every
                    // probe AND every store inside one _SEH2_TRY, so the stores
                    // must be the fault-tolerant copy.
                    let ok = crate::copy::publish_to_user(
                        piType as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*pmto).iType) as *const core::ffi::c_void,
                        core::mem::size_of::<DWORD>(),
                    ) && crate::copy::publish_to_user(
                        pmm as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*pmto).mm) as *const core::ffi::c_void,
                        core::mem::size_of::<DWORD>(),
                    ) && crate::copy::publish_to_user(
                        pxExt as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*pmto).xExt) as *const core::ffi::c_void,
                        core::mem::size_of::<DWORD>(),
                    ) && crate::copy::publish_to_user(
                        pyExt as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*pmto).yExt) as *const core::ffi::c_void,
                        core::mem::size_of::<DWORD>(),
                    ) && crate::copy::publish_to_user(
                        pjData as *mut core::ffi::c_void,
                        (*pmto).pjData.as_ptr() as *const core::ffi::c_void,
                        cjData as usize,
                    );
                    if !ok {
                        cjRet = 0;
                    }
                }
            } else {
                cjRet = 0;
            }
        }
    }

    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*pmto).BaseObject));
    cjRet
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiConvertMetafileRect(hDC: HDC, pRect: PRECTL) -> LONG {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32k Subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/metafile.c
//  * PURPOSE:         Metafile Implementations, Metafile Clipboard Data Xfers
//  * PROGRAMMERS:     ...
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// // Internal Use Only
// typedef struct _METATYPEOBJ
// {
//     BASEOBJECT  BaseObject;
//     DWORD iType;
//     DWORD mm;
//     DWORD xExt;
//     DWORD yExt;
//     ULONG cjData;
//     PBYTE pjData[4];
// } METATYPEOBJ, *PMETATYPEOBJ;
//
//
// /* System Service Calls ******************************************************/
//
// /*
//  * @implemented
//  */
// HDC
// __stdcall
// NtGdiCreateMetafileDC(IN HDC hdc)
// {
//     //if (hdc)
//         /* Call the internal function to create an alternative info DC */
//         return GreCreateCompatibleDC(hdc, TRUE);
//     // No HDC call NtUser.
//     //return UserGetDesktopDC(DCTYPE_INFO, TRUE, FALSE);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// __stdcall
// NtGdiCreateServerMetaFile(
//     IN DWORD iType,
//     IN ULONG cjData,
//     IN PBYTE pjData,
//     IN DWORD mm,
//     IN DWORD xExt,
//     IN DWORD yExt
// )
// {
//     BOOL Pass = TRUE;
//     PMETATYPEOBJ pmto;
//     NTSTATUS Status;
//
//     if ( ( iType == GDITAG_TYPE_EMF || iType == GDITAG_TYPE_MFP ) &&
//         cjData &&
//         pjData )
//     {
//         pmto = (PMETATYPEOBJ)GDIOBJ_AllocObjWithHandle(GDILoObjType_LO_META_TYPE, sizeof(METATYPEOBJ) + cjData);
//         if ( pmto )
//         {
//             pmto->iType  = iType;
//             pmto->mm     = mm;
//             pmto->xExt   = xExt;
//             pmto->yExt   = yExt;
//             pmto->cjData = cjData;
//
//             Status = SafeProbeForRead(pjData, cjData, 1);
//             if (!NT_SUCCESS(Status))
//             {
//                 Pass = FALSE;
//             }
//             else
//             {
//                 RtlCopyMemory( pmto->pjData, pjData, cjData) ;
//             }
//
//             if (Pass)
//             {
//                GDIOBJ_vSetObjectOwner(&pmto->BaseObject, GDI_OBJ_HMGR_PUBLIC);
//                GDIOBJ_vUnlockObject(&pmto->BaseObject);
//                return pmto->BaseObject.hHmgr;
//             }
//             else
//             {
//                GDIOBJ_vDeleteObject(&pmto->BaseObject);
//             }
//         }
//     }
//     return NULL;
// }
//
//
// /*
//  * @implemented
//  */
// ULONG
// __stdcall
// NtGdiGetServerMetaFileBits(
//     IN HANDLE hmo,
//     IN ULONG cjData,
//     OUT OPTIONAL PBYTE pjData,
//     OUT PDWORD piType,
//     OUT PDWORD pmm,
//     OUT PDWORD pxExt,
//     OUT PDWORD pyExt
// )
// {
//     ULONG cjRet = 0;
//     PMETATYPEOBJ pmto;
//     NTSTATUS Status;
//
//     pmto = (PMETATYPEOBJ) GDIOBJ_LockObject( hmo, GDIObjType_META_TYPE);
//
//     if (!pmto)
//         return 0;
//
//     if ( pmto->iType == GDITAG_TYPE_EMF || pmto->iType == GDITAG_TYPE_MFP )
//     {
//         cjRet = pmto->cjData;
//
//         if ( cjData )
//         {
//             if (cjData == pmto->cjData)
//             {
//                 Status = SafeProbeForWrite(piType, sizeof(DWORD), 1);
//                 if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pmm, sizeof(DWORD), 1);
//                 if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pxExt, sizeof(DWORD), 1);
//                 if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pyExt, sizeof(DWORD), 1);
//                 if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pjData, cjData, 1);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     cjRet = 0;
//                 }
//                 else
//                 {
//                     *piType = pmto->iType;
//                     *pmm = pmto->mm;
//                     *pxExt = pmto->xExt;
//                     *pyExt = pmto->yExt;
//                     RtlCopyMemory( pjData, pmto->pjData, cjData) ;
//                 }
//             }
//             else
//             {
//                 cjRet = 0;
//             }
//         }
//     }
//
//     GDIOBJ_vUnlockObject(&pmto->BaseObject);
//     return cjRet;
// }
//
//
// /*
//  * @unimplemented
//  */
// LONG
// __stdcall
// NtGdiConvertMetafileRect(IN HDC hDC,
//                          IN OUT PRECTL pRect)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
