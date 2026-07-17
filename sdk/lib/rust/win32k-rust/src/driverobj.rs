use win32k_sys::win32k::*;

// drop_gdiobj_macro_externs: dropped 3 bogus extern decl(s) for GDI-object macro(s): DRIVEROBJ_AllocObjectWithHandle, DRIVEROBJ_UnlockObject, DRIVEROBJ_TryLockObject — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// Macro expansions (from win32ss/gdi/eng/driverobj.h:17-25):
//   DRIVEROBJ_AllocObjectWithHandle()  -> (PEDRIVEROBJ)GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_DRIVEROBJ, sizeof(EDRIVEROBJ))
//   DRIVEROBJ_UnlockObject(pdo)        -> GDIOBJ_vUnlockObject((POBJ)pdo)
//   DRIVEROBJ_TryLockObject(hdo)       -> (PEDRIVEROBJ)GDIOBJ_TryLockObject(hdo, GDIObjType_DRVOBJ_TYPE)
// Note: GDI_OBJECT_TYPE_DRIVEROBJ (0x001C0000, ULONG) is the full type-with-flags code;
//       GDIObjType_DRVOBJ_TYPE       (0x1C, UCHAR) is the small enum index.
// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn DRIVEROBJ_vCleanup(pObject: PVOID) {
    let pedo: PEDRIVEROBJ = pObject as PEDRIVEROBJ;
    let pFreeProc: FREEOBJPROC = (*pedo).drvobj.pFreeProc;
    if !pFreeProc.is_none() {
        // NT_VERIFY(expr) is a release-build no-op that still evaluates expr.
        let _ = pFreeProc.unwrap()(&mut (*pedo).drvobj);
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngCreateDriverObj(
    pvObj: *mut core::ffi::c_void,
    pFreeObjProc: FREEOBJPROC,
    hdev: HDEV,
) -> HDRVOBJ {
    // Macro DRIVEROBJ_AllocObjectWithHandle / DRIVEROBJ_UnlockObject — see gdi/eng/driverobj.h:17-18
    let pedo: PEDRIVEROBJ;
    let hdo: HDRVOBJ;
    let ppdev = hdev as *mut PDEVOBJ;

    pedo = GDIOBJ_AllocObjWithHandle(
        GDI_OBJECT_TYPE_DRIVEROBJ as ULONG,
        core::mem::size_of::<EDRIVEROBJ>() as ULONG,
    ) as PEDRIVEROBJ;
    if pedo.is_null() {
        return core::ptr::null_mut();
    }
    hdo = (*pedo).baseobj.hHmgr as HDRVOBJ;

    (*pedo).drvobj.pvObj = pvObj;
    (*pedo).drvobj.pFreeProc = pFreeObjProc;
    (*pedo).drvobj.hdev = hdev;
    (*pedo).drvobj.dhpdev = (*ppdev).dhpdev;

    GDIOBJ_vUnlockObject(pedo as POBJ);

    return hdo;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngCreateDriverObj(
    pvObj: *mut core::ffi::c_void,
    pFreeObjProc: FREEOBJPROC,
    hdev: HDEV,
) -> HDRVOBJ {
    // Macro DRIVEROBJ_AllocObjectWithHandle / DRIVEROBJ_UnlockObject — see gdi/eng/driverobj.h:17-18
    let pedo: PEDRIVEROBJ;
    let hdo: HDRVOBJ;
    let ppdev = hdev as *mut PDEVOBJ;

    pedo = GDIOBJ_AllocObjWithHandle(
        GDI_OBJECT_TYPE_DRIVEROBJ as ULONG,
        core::mem::size_of::<EDRIVEROBJ>() as ULONG,
    ) as PEDRIVEROBJ;
    if pedo.is_null() {
        return core::ptr::null_mut();
    }
    hdo = (*pedo).baseobj.hHmgr as HDRVOBJ;

    (*pedo).drvobj.pvObj = pvObj;
    (*pedo).drvobj.pFreeProc = pFreeObjProc;
    (*pedo).drvobj.hdev = hdev;
    (*pedo).drvobj.dhpdev = (*ppdev).dhpdev;

    GDIOBJ_vUnlockObject(pedo as POBJ);

    return hdo;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngDeleteDriverObj(
    hdo: HDRVOBJ,
    bCallBack: i32,
    bLocked: i32,
) -> i32 {
    // Macros DRIVEROBJ_TryLockObject / DRIVEROBJ_UnlockObject — see gdi/eng/driverobj.h:18,22

    let pedo: PEDRIVEROBJ = GDIOBJ_TryLockObject(hdo as HGDIOBJ, GDIObjType_DRVOBJ_TYPE as UCHAR) as PEDRIVEROBJ;
    if pedo.is_null() {
        return 0;
    }

    if bCallBack != 0 {
        if (*pedo).drvobj.pFreeProc.is_some() {
            let callback = (*pedo).drvobj.pFreeProc.unwrap();
            if callback(core::ptr::addr_of_mut!((*pedo).drvobj)) == 0 {
                GDIOBJ_vUnlockObject(pedo as POBJ);
                return 0;
            }
        }
    }

    (*pedo).drvobj.pFreeProc = None;

    if bLocked != 0 {
        GDIOBJ_vUnlockObject(pedo as POBJ);
    }

    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pedo).baseobj));
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngDeleteDriverObj(
    hdo: HDRVOBJ,
    bCallBack: i32,
    bLocked: i32,
) -> i32 {
    // Macros DRIVEROBJ_TryLockObject / DRIVEROBJ_UnlockObject — see gdi/eng/driverobj.h:18,22

    let pedo: PEDRIVEROBJ = GDIOBJ_TryLockObject(hdo as HGDIOBJ, GDIObjType_DRVOBJ_TYPE as UCHAR) as PEDRIVEROBJ;
    if pedo.is_null() {
        return 0;
    }

    if bCallBack != 0 {
        if (*pedo).drvobj.pFreeProc.is_some() {
            let callback = (*pedo).drvobj.pFreeProc.unwrap();
            if callback(core::ptr::addr_of_mut!((*pedo).drvobj)) == 0 {
                GDIOBJ_vUnlockObject(pedo as POBJ);
                return 0;
            }
        }
    }

    (*pedo).drvobj.pFreeProc = None;

    if bLocked != 0 {
        GDIOBJ_vUnlockObject(pedo as POBJ);
    }

    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pedo).baseobj));
    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngLockDriverObj(
    hdo: HDRVOBJ
) -> PDRIVEROBJ {
    // Macro DRIVEROBJ_TryLockObject — see gdi/eng/driverobj.h:22

    let pedo: PEDRIVEROBJ = GDIOBJ_TryLockObject(hdo as HGDIOBJ, GDIObjType_DRVOBJ_TYPE as UCHAR) as PEDRIVEROBJ;

    if !pedo.is_null() {
        core::ptr::addr_of_mut!((*pedo).drvobj)
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngLockDriverObj(
    hdo: HDRVOBJ
) -> PDRIVEROBJ {
    // Macro DRIVEROBJ_TryLockObject — see gdi/eng/driverobj.h:22

    let pedo: PEDRIVEROBJ = GDIOBJ_TryLockObject(hdo as HGDIOBJ, GDIObjType_DRVOBJ_TYPE as UCHAR) as PEDRIVEROBJ;

    if !pedo.is_null() {
        core::ptr::addr_of_mut!((*pedo).drvobj)
    } else {
        core::ptr::null_mut()
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngUnlockDriverObj(
    hdo: HDRVOBJ
) -> i32 {
    // Macros DRIVEROBJ_TryLockObject / DRIVEROBJ_UnlockObject — see gdi/eng/driverobj.h:18,22

    let pedo: PEDRIVEROBJ;
    let cLocks: ULONG;

    pedo = GDIOBJ_TryLockObject(hdo as HGDIOBJ, GDIObjType_DRVOBJ_TYPE as UCHAR) as PEDRIVEROBJ;
    if pedo.is_null() {
        return 0;
    }

    cLocks = (*pedo).baseobj.cExclusiveLock as ULONG;
    GDIOBJ_vUnlockObject(pedo as POBJ);

    if cLocks < 2 {
        return 0;
    }

    GDIOBJ_vUnlockObject(pedo as POBJ);

    return 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngUnlockDriverObj(
    hdo: HDRVOBJ
) -> i32 {
    // Macros DRIVEROBJ_TryLockObject / DRIVEROBJ_UnlockObject — see gdi/eng/driverobj.h:18,22

    let pedo: PEDRIVEROBJ;
    let cLocks: ULONG;

    pedo = GDIOBJ_TryLockObject(hdo as HGDIOBJ, GDIObjType_DRVOBJ_TYPE as UCHAR) as PEDRIVEROBJ;
    if pedo.is_null() {
        return 0;
    }

    cLocks = (*pedo).baseobj.cExclusiveLock as ULONG;
    GDIOBJ_vUnlockObject(pedo as POBJ);

    if cLocks < 2 {
        return 0;
    }

    GDIOBJ_vUnlockObject(pedo as POBJ);

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         GPL, see COPYING in the top level directory
//  * PROJECT:           ReactOS win32 kernel mode sunsystem
//  * PURPOSE:           GDI DRIVEROBJ Functions
//  * FILE:              win32ss/gdi/eng/driverobj.c
//  * PROGRAMER:         Timo Kreuzer
//  */
//
// /** Includes ******************************************************************/
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /** Internal interface ********************************************************/
//
// /*!
//  * \brief DRIVEROBJ cleanup function
//  */
// VOID
// NTAPI
// DRIVEROBJ_vCleanup(PVOID pObject)
// {
//     PEDRIVEROBJ pedo = pObject;
//     FREEOBJPROC pFreeProc;
//
//     pFreeProc = pedo->drvobj.pFreeProc;
//     if (pFreeProc)
//     {
//         NT_VERIFY(pFreeProc(&pedo->drvobj));
//     }
// }
//
// /** Public interface **********************************************************/
//
// _Must_inspect_result_
// HDRVOBJ
// APIENTRY
// EngCreateDriverObj(
//     _In_ PVOID       pvObj,
//     _In_opt_ FREEOBJPROC pFreeObjProc,
//     _In_ HDEV        hdev)
// {
//     PEDRIVEROBJ pedo;
//     HDRVOBJ hdo;
//     PDEVOBJ *ppdev = (PDEVOBJ*)hdev;
//
//     /* Allocate a new DRIVEROBJ */
//     pedo = DRIVEROBJ_AllocObjectWithHandle();
//     if (!pedo)
//     {
//         return NULL;
//     }
//     hdo = pedo->baseobj.hHmgr;
//
//     /* Fill in fields */
//     pedo->drvobj.pvObj = pvObj;
//     pedo->drvobj.pFreeProc = pFreeObjProc;
//     pedo->drvobj.hdev = hdev;
//     pedo->drvobj.dhpdev = ppdev->dhpdev;
//
//     /* Unlock the object */
//     DRIVEROBJ_UnlockObject(pedo);
//
//     /* Return the handle */
//     return hdo;
// }
//
//
// BOOL
// APIENTRY
// EngDeleteDriverObj(
//     _In_ _Post_ptr_invalid_ HDRVOBJ hdo,
//     _In_ BOOL    bCallBack,
//     _In_ BOOL    bLocked)
// {
//     PEDRIVEROBJ pedo;
//
//     /* Lock the object */
//     pedo = DRIVEROBJ_TryLockObject(hdo);
//     if (!pedo)
//     {
//         return FALSE;
//     }
//
//     /* Manually call cleanup callback */
//     if (bCallBack)
//     {
//         if (!pedo->drvobj.pFreeProc(&pedo->drvobj))
//         {
//             /* Callback failed */
//             DRIVEROBJ_UnlockObject(pedo);
//             return FALSE;
//         }
//     }
//
//     /* Prevent cleanup callback from being called again */
//     pedo->drvobj.pFreeProc = NULL;
//
//     /* Unlock if the caller indicates it is locked */
//     if (bLocked)
//         DRIVEROBJ_UnlockObject(pedo);
//
//     /* Now delete the object */
//     GDIOBJ_vDeleteObject(&pedo->baseobj);
//     return TRUE;
// }
//
//
// PDRIVEROBJ
// APIENTRY
// EngLockDriverObj(
//     _In_ HDRVOBJ hdo)
// {
//     PEDRIVEROBJ pedo;
//
//     /* Lock the object */
//     pedo = DRIVEROBJ_TryLockObject(hdo);
//
//     /* Return pointer to the DRIVEROBJ structure */
//     return pedo ? &pedo->drvobj : NULL;
// }
//
//
// BOOL
// APIENTRY
// EngUnlockDriverObj(
//     _In_ _Post_ptr_invalid_ HDRVOBJ hdo)
// {
//     PEDRIVEROBJ pedo;
//     ULONG cLocks;
//
//     /* First lock to get a pointer to the object */
//     pedo = DRIVEROBJ_TryLockObject(hdo);
//     if(!pedo)
//     {
//         /* Object could not be locked, fail. */
//         return FALSE;
//     }
//
//     /* Unlock object */
//     cLocks = pedo->baseobj.cExclusiveLock;
//     DRIVEROBJ_UnlockObject(pedo);
//
//     /* Check if we still hold a lock */
//     if (cLocks < 2)
//     {
//         /* Object wasn't locked before, fail. */
//         return FALSE;
//     }
//
//     /* Unlock again */
//     DRIVEROBJ_UnlockObject(pedo);
//
//     /* Success */
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
