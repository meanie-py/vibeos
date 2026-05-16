/*
 * PROJECT:         ReactOS Win32k Subsystem
 * LICENSE:         GPL - See COPYING in the top level directory
 * FILE:            win32ss/gdi/ntgdi/metafile.c
 * PURPOSE:         Metafile Implementations, Metafile Clipboard Data Xfers
 * PROGRAMMERS:     ...
 */

/* INCLUDES ******************************************************************/

#include <win32k.h>
#include <safe_user_io.h>

#define NDEBUG
#include <debug.h>


// Internal Use Only
typedef struct _METATYPEOBJ
{
    BASEOBJECT  BaseObject;
    DWORD iType;
    DWORD mm;
    DWORD xExt;
    DWORD yExt;
    ULONG cjData;
    PBYTE pjData[4];
} METATYPEOBJ, *PMETATYPEOBJ;


/* System Service Calls ******************************************************/

/*
 * @implemented
 */
HDC
APIENTRY
NtGdiCreateMetafileDC(IN HDC hdc)
{
    //if (hdc)
        /* Call the internal function to create an alternative info DC */
        return GreCreateCompatibleDC(hdc, TRUE);
    // No HDC call NtUser.
    //return UserGetDesktopDC(DCTYPE_INFO, TRUE, FALSE);
}

/*
 * @implemented
 */
HANDLE
APIENTRY
NtGdiCreateServerMetaFile(
    IN DWORD iType,
    IN ULONG cjData,
    IN PBYTE pjData,
    IN DWORD mm,
    IN DWORD xExt,
    IN DWORD yExt
)
{
    BOOL Pass = TRUE;
    PMETATYPEOBJ pmto;
    NTSTATUS Status;

    if ( ( iType == GDITAG_TYPE_EMF || iType == GDITAG_TYPE_MFP ) &&
        cjData &&
        pjData )
    {
        pmto = (PMETATYPEOBJ)GDIOBJ_AllocObjWithHandle(GDILoObjType_LO_META_TYPE, sizeof(METATYPEOBJ) + cjData);
        if ( pmto )
        {
            pmto->iType  = iType;
            pmto->mm     = mm;
            pmto->xExt   = xExt;
            pmto->yExt   = yExt;
            pmto->cjData = cjData;

            Status = SafeProbeForRead(pjData, cjData, 1);
            if (!NT_SUCCESS(Status))
            {
                Pass = FALSE;
            }
            else
            {
                RtlCopyMemory( pmto->pjData, pjData, cjData) ;
            }

            if (Pass)
            {
               GDIOBJ_vSetObjectOwner(&pmto->BaseObject, GDI_OBJ_HMGR_PUBLIC);
               GDIOBJ_vUnlockObject(&pmto->BaseObject);
               return pmto->BaseObject.hHmgr;
            }
            else
            {
               GDIOBJ_vDeleteObject(&pmto->BaseObject);
            }
        }
    }
    return NULL;
}


/*
 * @implemented
 */
ULONG
APIENTRY
NtGdiGetServerMetaFileBits(
    IN HANDLE hmo,
    IN ULONG cjData,
    OUT OPTIONAL PBYTE pjData,
    OUT PDWORD piType,
    OUT PDWORD pmm,
    OUT PDWORD pxExt,
    OUT PDWORD pyExt
)
{
    ULONG cjRet = 0;
    PMETATYPEOBJ pmto;
    NTSTATUS Status;

    pmto = (PMETATYPEOBJ) GDIOBJ_LockObject( hmo, GDIObjType_META_TYPE);

    if (!pmto)
        return 0;

    if ( pmto->iType == GDITAG_TYPE_EMF || pmto->iType == GDITAG_TYPE_MFP )
    {
        cjRet = pmto->cjData;

        if ( cjData )
        {
            if (cjData == pmto->cjData)
            {
                Status = SafeProbeForWrite(piType, sizeof(DWORD), 1);
                if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pmm, sizeof(DWORD), 1);
                if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pxExt, sizeof(DWORD), 1);
                if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pyExt, sizeof(DWORD), 1);
                if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pjData, cjData, 1);
                if (!NT_SUCCESS(Status))
                {
                    cjRet = 0;
                }
                else
                {
                    *piType = pmto->iType;
                    *pmm = pmto->mm;
                    *pxExt = pmto->xExt;
                    *pyExt = pmto->yExt;
                    RtlCopyMemory( pjData, pmto->pjData, cjData) ;
                }
            }
            else
            {
                cjRet = 0;
            }
        }
    }

    GDIOBJ_vUnlockObject(&pmto->BaseObject);
    return cjRet;
}


/*
 * @unimplemented
 */
LONG
APIENTRY
NtGdiConvertMetafileRect(IN HDC hDC,
                         IN OUT PRECTL pRect)
{
    UNIMPLEMENTED;
    return 0;
}

/* EOF */
