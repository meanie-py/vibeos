/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS Win32k subsystem
 * PURPOSE:          Cursor and icon functions
 * FILE:             win32ss/user/ntuser/cursoricon.c
 * PROGRAMER:        ReactOS Team
 */
/*
 * We handle two types of cursors/icons:
 * - Private
 *   Loaded without LR_SHARED flag
 *   Private to a process
 *   Can be deleted by calling NtDestroyCursorIcon()
 *   CurIcon->hModule, CurIcon->hRsrc and CurIcon->hGroupRsrc set to NULL
 * - Shared
 *   Loaded with LR_SHARED flag
 *   Possibly shared by multiple processes
 *   Immune to NtDestroyCursorIcon()
 *   CurIcon->hModule, CurIcon->hRsrc and CurIcon->hGroupRsrc are valid
 */

#include <win32k.h>
#include <safe_user_io.h>
DBG_DEFAULT_CHANNEL(UserIcon);

SYSTEM_CURSORINFO gSysCursorInfo;

PCURICON_OBJECT gcurFirst = NULL; // After all is done, this should be WINLOGO!

//
//   System Cursors
//
SYSTEMCURICO gasyscur[] = {
    {OCR_NORMAL,     NULL},
    {OCR_IBEAM,      NULL},
    {OCR_WAIT,       NULL},
    {OCR_CROSS,      NULL},
    {OCR_UP,         NULL},
    {OCR_ICON,       NULL},
    {OCR_SIZE,       NULL},
    {OCR_SIZENWSE,   NULL},
    {OCR_SIZENESW,   NULL},
    {OCR_SIZEWE,     NULL},
    {OCR_SIZENS,     NULL},
    {OCR_SIZEALL,    NULL},
    {OCR_NO,         NULL},
    {OCR_HAND,       NULL},
    {OCR_APPSTARTING,NULL},
    {OCR_HELP,       NULL},
    };

//
//   System Icons
//
SYSTEMCURICO gasysico[] = {
    {OIC_SAMPLE, NULL},
    {OIC_HAND,   NULL},
    {OIC_QUES,   NULL},
    {OIC_BANG,   NULL},
    {OIC_NOTE,   NULL},
    {OIC_WINLOGO,NULL},
    };

BOOL
InitCursorImpl(VOID)
{
    gSysCursorInfo.Enabled = FALSE;
    gSysCursorInfo.ButtonsDown = 0;
    gSysCursorInfo.bClipped = FALSE;
    gSysCursorInfo.LastBtnDown = 0;
    gSysCursorInfo.CurrentCursorObject = NULL;
    gSysCursorInfo.ShowingCursor = -1;
    gSysCursorInfo.ClickLockActive = FALSE;
    gSysCursorInfo.ClickLockTime = 0;

    return TRUE;
}

static
VOID
IntInsertCursorIntoList(
    _Inout_ PCURICON_OBJECT pcur)
{
    PPROCESSINFO ppi = pcur->head.ppi;
    PCURICON_OBJECT *ppcurHead;
    NT_ASSERT((pcur->CURSORF_flags & (CURSORF_GLOBAL|CURSORF_LRSHARED)) != 0);
    NT_ASSERT((pcur->CURSORF_flags & CURSORF_LINKED) == 0);

    /* Get the right list head */
    ppcurHead = (pcur->CURSORF_flags & CURSORF_GLOBAL) ?
        &gcurFirst : &ppi->pCursorCache;

    UserReferenceObject(pcur);
    pcur->pcurNext = *ppcurHead;
    *ppcurHead = pcur;
    pcur->CURSORF_flags |= CURSORF_LINKED;
}

// FIXME: should think about using a LIST_ENTRY!
static
VOID
IntRemoveCursorFromList(
    _Inout_ PCURICON_OBJECT pcur)
{
    PPROCESSINFO ppi = pcur->head.ppi;
    PCURICON_OBJECT *ppcurHead;
    PCURICON_OBJECT *ppcur;
    NT_ASSERT((pcur->CURSORF_flags & (CURSORF_GLOBAL|CURSORF_LRSHARED)) != 0);
    NT_ASSERT((pcur->CURSORF_flags & CURSORF_LINKED) != 0);

    /* Get the right list head */
    ppcurHead = (pcur->CURSORF_flags & CURSORF_GLOBAL) ?
        &gcurFirst : &ppi->pCursorCache;

    /* Loop all cursors in the cache */
    for (ppcur = ppcurHead;
         (*ppcur) != NULL;
         ppcur = &(*ppcur)->pcurNext)
    {
        /* Check if this is the one we are looking for */
        if ((*ppcur) == pcur)
        {
            /* Remove it from the list */
            (*ppcur) = pcur->pcurNext;

            /* Dereference it */
            UserDereferenceObject(pcur);
            pcur->CURSORF_flags &= ~CURSORF_LINKED;
            return;
        }
    }

    /* We did not find it, this must not happen */
    NT_ASSERT(FALSE);
}

VOID
IntLoadSystenIcons(HICON hcur, DWORD id)
{
    PCURICON_OBJECT pcur;
    int i;
    PPROCESSINFO ppi;

    if (hcur)
    {
        pcur = UserGetCurIconObject(hcur);
        if (!pcur)
        {
            EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
            return;
        }

        ppi = PsGetCurrentProcessWin32Process();

        if (!(ppi->W32PF_flags & W32PF_CREATEDWINORDC))
           return;

        // Set Small Window Icon and do not link.
        if ( id == OIC_INTERNAL_WINSMALL )
        {
            pcur->CURSORF_flags |= CURSORF_GLOBAL;
            UserReferenceObject(pcur);
            pcur->head.ppi = NULL;
            return;
        }

        for (i = 0 ; i < 6; i++)
        {
            if (gasysico[i].type == id)
            {
                gasysico[i].handle = pcur;
                pcur->CURSORF_flags |= CURSORF_GLOBAL;

                //
                //  The active switch between LR shared and Global public.
                //  This is hacked around to support this while at the initial system start up.
                //
                pcur->head.ppi = NULL;

                IntInsertCursorIntoList(pcur);
                return;
            }
        }
    }
}

PSYSTEM_CURSORINFO
IntGetSysCursorInfo(VOID)
{
    return &gSysCursorInfo;
}

FORCEINLINE
BOOL
is_icon(PCURICON_OBJECT object)
{
    return MAKEINTRESOURCE(object->rt) == RT_ICON;
}

/* This function creates a reference for the object! */
PCURICON_OBJECT FASTCALL UserGetCurIconObject(HCURSOR hCurIcon)
{
    PCURICON_OBJECT CurIcon;

    if (!hCurIcon)
    {
        EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
        return NULL;
    }

    if (UserObjectInDestroy(hCurIcon))
    {
        WARN("Requesting invalid/destroyed cursor.\n");
        EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
        return NULL;
    }

    CurIcon = (PCURICON_OBJECT)UserReferenceObjectByHandle(hCurIcon, TYPE_CURSOR);
    if (!CurIcon)
    {
        /* We never set ERROR_INVALID_ICON_HANDLE. lets hope noone ever checks for it */
        EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
        return NULL;
    }

    ASSERT(CurIcon->head.cLockObj >= 1);
    return CurIcon;
}

PCURICON_OBJECT
IntSystemSetCursor(PCURICON_OBJECT pcurNew)
{
    PCURICON_OBJECT pcurOld = UserSetCursor(pcurNew, FALSE);
    if (pcurNew) UserReferenceObject(pcurNew);
    if (pcurOld) UserDereferenceObject(pcurOld);
    return pcurOld;
}

BOOL UserSetCursorPos( INT x, INT y, DWORD flags, ULONG_PTR dwExtraInfo, BOOL Hook)
{
    PWND DesktopWindow;
    PSYSTEM_CURSORINFO CurInfo;
    MSG Msg;
    RECTL rcClip;
    POINT pt;

    if (!(DesktopWindow = UserGetDesktopWindow()))
    {
        return FALSE;
    }

    CurInfo = IntGetSysCursorInfo();

    /* Clip cursor position */
    if (!CurInfo->bClipped)
        rcClip = DesktopWindow->rcClient;
    else
        rcClip = CurInfo->rcClip;

    if (x >= rcClip.right)  x = rcClip.right - 1;
    if (x < rcClip.left)    x = rcClip.left;
    if (y >= rcClip.bottom) y = rcClip.bottom - 1;
    if (y < rcClip.top)     y = rcClip.top;

    /* Nothing to do if position did not actually change */
    if (x == gpsi->ptCursor.x && y == gpsi->ptCursor.y)
        return TRUE;

    pt.x = x;
    pt.y = y;

    /* 1. Generate a mouse move message, this sets the htEx and Track Window too */
    Msg.message = WM_MOUSEMOVE;
    Msg.wParam = UserGetMouseButtonsState();
    Msg.lParam = MAKELPARAM(x, y);
    Msg.pt = pt;
    co_MsqInsertMouseMessage(&Msg, flags, dwExtraInfo, Hook);

    /* 2. Store the new cursor position */
    gpsi->ptCursor = pt;

    return TRUE;
}

HANDLE
IntCreateCurIconHandle(BOOLEAN Animated)
{
    PCURICON_OBJECT CurIcon;
    HANDLE hCurIcon;

    CurIcon = UserCreateObject(
        gHandleTable,
        NULL,
        GetW32ThreadInfo(),
        &hCurIcon,
        TYPE_CURSOR,
        Animated ? sizeof(ACON) : sizeof(CURICON_OBJECT));

    if (!CurIcon)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }

    if (Animated)
    {
        /* We MUST set this flag, to track whether this is an ACON! */
        CurIcon->CURSORF_flags |= CURSORF_ACON;
    }

    NT_ASSERT(CurIcon->pcurNext == NULL);
    UserDereferenceObject(CurIcon);

    return hCurIcon;
}

BOOLEAN
IntDestroyCurIconObject(
    _In_ PVOID Object)
{
    PCURICON_OBJECT CurIcon = Object;

    /* Check if the cursor is in a list */
    if (CurIcon->CURSORF_flags & CURSORF_LINKED)
    {
        /* Remove the cursor from it's list */
        IntRemoveCursorFromList(CurIcon);
    }

    /* We just mark the handle as being destroyed.
     * Deleting all the stuff will be deferred to the actual struct free. */
    UserDeleteObject(UserHMGetHandle(CurIcon), TYPE_CURSOR);
    return TRUE;
}

VOID
FreeCurIconObject(
    _In_ PVOID Object)
{
    PCURICON_OBJECT CurIcon = Object;

    if (!(CurIcon->CURSORF_flags & CURSORF_ACON))
    {
        HBITMAP bmpMask = CurIcon->hbmMask;
        HBITMAP bmpColor = CurIcon->hbmColor;
        HBITMAP bmpAlpha = CurIcon->hbmAlpha;

        /* Delete bitmaps */
        if (bmpMask)
        {
            GreSetObjectOwner(bmpMask, GDI_OBJ_HMGR_POWNED);
            NT_VERIFY(GreDeleteObject(bmpMask) == TRUE);
            CurIcon->hbmMask = NULL;
        }
        if (bmpColor)
        {
            GreSetObjectOwner(bmpColor, GDI_OBJ_HMGR_POWNED);
            NT_VERIFY(GreDeleteObject(bmpColor) == TRUE);
            CurIcon->hbmColor = NULL;
        }
        if (bmpAlpha)
        {
            GreSetObjectOwner(bmpAlpha, GDI_OBJ_HMGR_POWNED);
            NT_VERIFY(GreDeleteObject(bmpAlpha) == TRUE);
            CurIcon->hbmAlpha = NULL;
        }
    }
    else
    {
        PACON AniCurIcon = (PACON)CurIcon;
        UINT i;

        for (i = 0; i < AniCurIcon->cpcur; i++)
        {
            UserDereferenceObject(AniCurIcon->aspcur[i]);
            NT_VERIFY(IntDestroyCurIconObject(AniCurIcon->aspcur[i]) == TRUE);
        }
        ExFreePoolWithTag(AniCurIcon->aspcur, USERTAG_CURSOR);
    }

    if (CurIcon->CURSORF_flags & CURSORF_LRSHARED)
    {
        if (!IS_INTRESOURCE(CurIcon->strName.Buffer))
            ExFreePoolWithTag(CurIcon->strName.Buffer, TAG_STRING);
        if (CurIcon->atomModName)
            RtlDeleteAtomFromAtomTable(gAtomTable, CurIcon->atomModName);
        CurIcon->strName.Buffer = NULL;
        CurIcon->atomModName = 0;
    }

    /* Finally free the thing */
    FreeProcMarkObject(CurIcon);
}

VOID FASTCALL
IntCleanupCurIconCache(PPROCESSINFO Win32Process)
{
    PCURICON_OBJECT CurIcon;

    /* Run through the list of icon objects */
    while (Win32Process->pCursorCache)
    {
        CurIcon = Win32Process->pCursorCache;
        Win32Process->pCursorCache = CurIcon->pcurNext;
        UserDereferenceObject(CurIcon);
    }
}

/*
 * @implemented
 */
_Success_(return != FALSE)
BOOL
NTAPI
NtUserGetIconInfo(
    _In_ HANDLE hCurIcon,
    _Out_opt_ PICONINFO IconInfo,
    _Inout_opt_ PUNICODE_STRING lpModule,
    _Inout_opt_ PUNICODE_STRING lpResName,
    _Out_opt_ LPDWORD pbpp,
    _In_ BOOL bInternal)
{
    ICONINFO ii;
    PCURICON_OBJECT CurIcon;
    NTSTATUS Status = STATUS_SUCCESS;
    BOOL Ret = FALSE;
    DWORD colorBpp = 0;

    TRACE("Enter NtUserGetIconInfo\n");

    /* Check if something was actually asked */
    if (!IconInfo && !lpModule && !lpResName)
    {
        WARN("Nothing to fill.\n");
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    UserEnterExclusive();

    if (!(CurIcon = UserGetCurIconObject(hCurIcon)))
    {
        WARN("UserGetIconObject(0x%p) Failed.\n", hCurIcon);
        UserLeave();
        return FALSE;
    }

    /* Give back the icon information */
    if (IconInfo)
    {
        PCURICON_OBJECT FrameCurIcon = CurIcon;
        if (CurIcon->CURSORF_flags & CURSORF_ACON)
        {
            /* Get information from first frame. */
            FrameCurIcon = ((PACON)CurIcon)->aspcur[0];
        }

        /* Fill data */
        ii.fIcon = is_icon(FrameCurIcon);
        ii.xHotspot = FrameCurIcon->xHotspot;
        ii.yHotspot = FrameCurIcon->yHotspot;

        /* Copy bitmaps */
        ii.hbmMask = BITMAP_CopyBitmap(FrameCurIcon->hbmMask);
        GreSetObjectOwner(ii.hbmMask, GDI_OBJ_HMGR_POWNED);
        ii.hbmColor = BITMAP_CopyBitmap(FrameCurIcon->hbmColor);
        GreSetObjectOwner(ii.hbmColor, GDI_OBJ_HMGR_POWNED);
        colorBpp = FrameCurIcon->bpp;

        /* Copy fields */
        Status = SafeProbeForWrite(IconInfo, sizeof(ICONINFO), 1);
        if (NT_SUCCESS(Status))
            Status = SafeProbeForWrite(pbpp, sizeof(DWORD), 1);
        if (NT_SUCCESS(Status))
        {
            RtlCopyMemory(IconInfo, &ii, sizeof(ICONINFO));

            if (pbpp)
            {

                *pbpp = colorBpp;
            }
        }

        if (!NT_SUCCESS(Status))
        {
            WARN("Status: 0x%08lx\n", Status);
            SetLastNtError(Status);
            goto leave;
        }
    }

    /* Give back the module name */
    if (lpModule)
    {
        ULONG BufLen = 0;
        if (!CurIcon->atomModName)
            goto leave;

        RtlQueryAtomInAtomTable(gAtomTable, CurIcon->atomModName, NULL, NULL, NULL, &BufLen);
        /* Get the module name from the atom table */
        _SEH2_TRY
        {
            BufLen += sizeof(WCHAR);
            if (BufLen > (lpModule->MaximumLength))
            {
                lpModule->Length = 0;
                lpModule->MaximumLength = BufLen;
            }
            else
            {
                ProbeForWrite(lpModule->Buffer, lpModule->MaximumLength, 1);
                BufLen = lpModule->MaximumLength;
                RtlQueryAtomInAtomTable(gAtomTable, CurIcon->atomModName, NULL, NULL, lpModule->Buffer, &BufLen);
                lpModule->Length = BufLen;
            }
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            Status = _SEH2_GetExceptionCode();
        }
        _SEH2_END

        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
            goto leave;
        }
    }

    if (lpResName)
    {
        if (!CurIcon->strName.Buffer)
            goto leave;

        /* Copy it */
        Status = SafeProbeForWrite(lpResName, sizeof(UNICODE_STRING), 1);
        if (NT_SUCCESS(Status))
            Status = SafeProbeForWrite(lpResName->Buffer, lpResName->MaximumLength, 1);
        if (NT_SUCCESS(Status))
        {
            if (IS_INTRESOURCE(CurIcon->strName.Buffer))
            {
                lpResName->Buffer = CurIcon->strName.Buffer;
                lpResName->Length = 0;
                lpResName->MaximumLength = 0;
            }
            else if (lpResName->MaximumLength < CurIcon->strName.MaximumLength)
            {
                lpResName->Length = 0;
                lpResName->MaximumLength = CurIcon->strName.MaximumLength;
            }
            else
            {

                RtlCopyMemory(lpResName->Buffer, CurIcon->strName.Buffer, CurIcon->strName.Length);
                lpResName->Length = CurIcon->strName.Length;
            }
        }
    }

    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        goto leave;
    }

    Ret = TRUE;

leave:
    UserDereferenceObject(CurIcon);

    TRACE("Leave NtUserGetIconInfo, ret=%i\n", Ret);
    UserLeave();

    return Ret;
}


/*
 * @implemented
 */
BOOL
APIENTRY
NtUserGetIconSize(
    HANDLE hCurIcon,
    UINT istepIfAniCur,
    PLONG plcx,       // &size.cx
    PLONG plcy)       // &size.cy
{
    PCURICON_OBJECT CurIcon;
    NTSTATUS Status = STATUS_SUCCESS;
    BOOL bRet = FALSE;

    TRACE("Enter NtUserGetIconSize\n");
    UserEnterShared();

    if (!(CurIcon = UserGetCurIconObject(hCurIcon)))
    {
        goto cleanup;
    }

    if (CurIcon->CURSORF_flags & CURSORF_ACON)
    {
        /* Use first frame for animated cursors */
        PACON AniCurIcon = (PACON)CurIcon;
        CurIcon = AniCurIcon->aspcur[0];
        UserDereferenceObject(AniCurIcon);
        UserReferenceObject(CurIcon);
    }

    Status = SafeProbeForWrite(plcx, sizeof(LONG), 1);
    if (NT_SUCCESS(Status))
        Status = SafeProbeForWrite(plcy, sizeof(LONG), 1);
    if (NT_SUCCESS(Status))
    {
        *plcx = CurIcon->cx;

        *plcy = CurIcon->cy;
    }

    if (NT_SUCCESS(Status))
        bRet = TRUE;
    else
        SetLastNtError(Status); // Maybe not, test this

    UserDereferenceObject(CurIcon);

cleanup:
    TRACE("Leave NtUserGetIconSize, ret=%i\n", bRet);
    UserLeave();
    return bRet;
}


/*
 * @implemented
 */
BOOL
APIENTRY
NtUserGetCursorInfo(
    PCURSORINFO pci)
{
    CURSORINFO SafeCi;
    PSYSTEM_CURSORINFO CurInfo;
    NTSTATUS Status = STATUS_SUCCESS;
    PCURICON_OBJECT CurIcon;
    BOOL Ret = FALSE;

    TRACE("Enter NtUserGetCursorInfo\n");
    UserEnterShared();

    CurInfo = IntGetSysCursorInfo();
    CurIcon = (PCURICON_OBJECT)CurInfo->CurrentCursorObject;

    SafeCi.cbSize = sizeof(CURSORINFO);
    SafeCi.flags = ((CurIcon && CurInfo->ShowingCursor >= 0) ? CURSOR_SHOWING : 0);
    SafeCi.hCursor = (CurIcon ? UserHMGetHandle(CurIcon) : NULL);

    SafeCi.ptScreenPos = gpsi->ptCursor;

    _SEH2_TRY
    {
        ProbeForWrite(pci, sizeof(CURSORINFO), 1);
        
        if (pci->cbSize == sizeof(CURSORINFO))
        {
            RtlCopyMemory(pci, &SafeCi, sizeof(CURSORINFO));
            Ret = TRUE;
        }
        else
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = _SEH2_GetExceptionCode();
    }
    _SEH2_END;
    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
    }

    UserLeave();
    TRACE("Leave NtUserGetCursorInfo, ret=%i\n", Ret);
    
    return Ret;
}

BOOL
APIENTRY
UserClipCursor(
    RECTL *prcl)
{
    PSYSTEM_CURSORINFO CurInfo;
    PWND DesktopWindow = NULL;

    if (!CheckWinstaAttributeAccess(WINSTA_WRITEATTRIBUTES))
    {
        return FALSE;
    }

    CurInfo = IntGetSysCursorInfo();

    DesktopWindow = UserGetDesktopWindow();

    if (prcl != NULL && DesktopWindow != NULL)
    {
        if (prcl->right < prcl->left || prcl->bottom < prcl->top)
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;
        }

        CurInfo->bClipped = TRUE;

        /* Set nw cliping region. Note: we can't use RECTL_bIntersectRect because
           it sets rect to 0 0 0 0 when it's empty. For more info see monitor winetest */
        CurInfo->rcClip.left = max(prcl->left, DesktopWindow->rcWindow.left);
        CurInfo->rcClip.right = min(prcl->right, DesktopWindow->rcWindow.right);
        if (CurInfo->rcClip.right < CurInfo->rcClip.left)
            CurInfo->rcClip.right = CurInfo->rcClip.left;

        CurInfo->rcClip.top = max(prcl->top, DesktopWindow->rcWindow.top);
        CurInfo->rcClip.bottom = min(prcl->bottom, DesktopWindow->rcWindow.bottom);
        if (CurInfo->rcClip.bottom < CurInfo->rcClip.top)
            CurInfo->rcClip.bottom = CurInfo->rcClip.top;

        /* Make sure cursor is in clipping region */
        UserSetCursorPos(gpsi->ptCursor.x, gpsi->ptCursor.y, 0, 0, FALSE);
    }
    else
    {
        CurInfo->bClipped = FALSE;
    }

    return TRUE;
}

/*
 * @implemented
 */
BOOL
APIENTRY
NtUserClipCursor(
    RECTL *prcl)
{
    RECTL rclLocal;
    BOOL bResult;

    if (prcl)
    {
        {
            NTSTATUS __SehStatus = SafeProbeForRead(prcl, sizeof(RECTL), 1);
            if (!NT_SUCCESS(__SehStatus))
            {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                return FALSE;
            }
        }
        /* Probe and copy rect */

        rclLocal = *prcl;

        prcl = &rclLocal;
    }

    UserEnterExclusive();

    /* Call the internal function */
    bResult = UserClipCursor(prcl);

    UserLeave();

    return bResult;
}


/*
 * @implemented
 */
BOOL
APIENTRY
NtUserDestroyCursor(
  _In_   HANDLE hCurIcon,
  _In_   BOOL bForce)
{
    BOOL ret;
    PCURICON_OBJECT CurIcon = NULL;

    TRACE("Enter NtUserDestroyCursorIcon (%p, %i)\n", hCurIcon, bForce);
    UserEnterExclusive();

    CurIcon = UserGetCurIconObject(hCurIcon);
    if (!CurIcon)
    {
        ret = FALSE;
        goto leave;
    }

    if (!bForce)
    {
        /* Can not destroy global objects */
        if (CurIcon->head.ppi == NULL)
        {
           ERR("Trying to delete global cursor!\n");
           ret = TRUE;
           goto leave;
        }

        /* Maybe we have good reasons not to destroy this object */
        if (CurIcon->head.ppi != PsGetCurrentProcessWin32Process())
        {
            /* No way, you're not touching my cursor */
            ret = FALSE;
            goto leave;
        }

        if (CurIcon->CURSORF_flags & CURSORF_CURRENT)
        {
            WARN("Trying to delete current cursor!\n");
            ret = FALSE;
            goto leave;
        }

        if (CurIcon->CURSORF_flags & CURSORF_LRSHARED)
        {
            WARN("Trying to delete shared cursor.\n");
            /* This one is not an error */
            ret = TRUE;
            goto leave;
        }
    }

    /* Destroy the handle */
    ret = IntDestroyCurIconObject(CurIcon);

leave:
    if (CurIcon)
        UserDereferenceObject(CurIcon);
    TRACE("Leave NtUserDestroyCursorIcon, ret=%i\n", ret);
    UserLeave();
    return ret;
}


/*
 * @implemented
 */
HICON
NTAPI
NtUserFindExistingCursorIcon(
  _In_  PUNICODE_STRING pustrModule,
  _In_  PUNICODE_STRING pustrRsrc,
  _In_  FINDEXISTINGCURICONPARAM* param)
{
    PCURICON_OBJECT CurIcon;
    HICON Ret = NULL;
    UNICODE_STRING ustrModuleSafe, ustrRsrcSafe;
    FINDEXISTINGCURICONPARAM paramSafe;
    NTSTATUS Status;
    PPROCESSINFO pProcInfo = PsGetCurrentProcessWin32Process();
    RTL_ATOM atomModName;

    TRACE("Enter NtUserFindExistingCursorIcon\n");

    Status = SafeProbeForRead(param, sizeof(*param), 1);
    if (NT_SUCCESS(Status))
    {
        RtlCopyMemory(&paramSafe, param, sizeof(paramSafe));
    }

    /* Capture resource name (it can be an INTRESOURCE == ATOM) */
    Status = ProbeAndCaptureUnicodeStringOrAtom(&ustrRsrcSafe, pustrRsrc);
    if (!NT_SUCCESS(Status))
        return NULL;
    Status = ProbeAndCaptureUnicodeString(&ustrModuleSafe, UserMode, pustrModule);
    if (!NT_SUCCESS(Status))
        goto done;
    Status = RtlLookupAtomInAtomTable(gAtomTable, ustrModuleSafe.Buffer, &atomModName);
    ReleaseCapturedUnicodeString(&ustrModuleSafe, UserMode);
    if (!NT_SUCCESS(Status))
    {
        /* The module is not in the atom table. No chance to find the cursor */
        goto done;
    }

    UserEnterShared();
    CurIcon = pProcInfo->pCursorCache;
    while (CurIcon)
    {
        /* Icon/cursor */
        if (paramSafe.bIcon != is_icon(CurIcon))
        {
            CurIcon = CurIcon->pcurNext;
            continue;
        }
        /* See if module names match */
        if (atomModName == CurIcon->atomModName)
        {
            /* They do. Now see if this is the same resource */
            if (IS_INTRESOURCE(CurIcon->strName.Buffer) != IS_INTRESOURCE(ustrRsrcSafe.Buffer))
            {
                /* One is an INT resource and the other is not -> no match */
                CurIcon = CurIcon->pcurNext;
                continue;
            }

            if (IS_INTRESOURCE(CurIcon->strName.Buffer))
            {
                if (CurIcon->strName.Buffer == ustrRsrcSafe.Buffer)
                {
                    /* INT resources match */
                    break;
                }
            }
            else if (RtlCompareUnicodeString(&ustrRsrcSafe, &CurIcon->strName, TRUE) == 0)
            {
                /* Resource name strings match */
                break;
            }
        }
        CurIcon = CurIcon->pcurNext;
    }

    /* Now search Global Cursors or Icons. */
    if (CurIcon == NULL)
    {
        CurIcon = gcurFirst;
        while (CurIcon)
        {
            /* Icon/cursor */
            if (paramSafe.bIcon != is_icon(CurIcon))
            {
                CurIcon = CurIcon->pcurNext;
                continue;
            }
            /* See if module names match */
            if (atomModName == CurIcon->atomModName)
            {
                /* They do. Now see if this is the same resource */
                if (IS_INTRESOURCE(CurIcon->strName.Buffer) != IS_INTRESOURCE(ustrRsrcSafe.Buffer))
                {
                    /* One is an INT resource and the other is not -> no match */
                    CurIcon = CurIcon->pcurNext;
                    continue;
                }
                if (IS_INTRESOURCE(CurIcon->strName.Buffer))
                {
                    if (CurIcon->strName.Buffer == ustrRsrcSafe.Buffer)
                    {
                        /* INT resources match */
                        break;
                    }
                }
                else if (RtlCompareUnicodeString(&ustrRsrcSafe, &CurIcon->strName, TRUE) == 0)
                {
                    /* Resource name strings match */
                    break;
                }
            }
            CurIcon = CurIcon->pcurNext;
        }
    }
    if (CurIcon)
        Ret = UserHMGetHandle(CurIcon);
    UserLeave();

done:
    if (!IS_INTRESOURCE(ustrRsrcSafe.Buffer))
        ExFreePoolWithTag(ustrRsrcSafe.Buffer, TAG_STRING);

    return Ret;
}


/*
 * @implemented
 */
BOOL
APIENTRY
NtUserGetClipCursor(
    RECTL *lpRect)
{
    PSYSTEM_CURSORINFO CurInfo;
    RECTL Rect;
    NTSTATUS Status;
    BOOL Ret = FALSE;

    TRACE("Enter NtUserGetClipCursor\n");
    UserEnterShared();

    if (!CheckWinstaAttributeAccess(WINSTA_READATTRIBUTES))
    {
        goto Exit; // Return FALSE
    }

    if (!lpRect)
        goto Exit; // Return FALSE

    CurInfo = IntGetSysCursorInfo();
    if (CurInfo->bClipped)
    {
        Rect = CurInfo->rcClip;
    }
    else
    {
        Rect.left = 0;
        Rect.top = 0;
        Rect.right = UserGetSystemMetrics(SM_CXSCREEN);
        Rect.bottom = UserGetSystemMetrics(SM_CYSCREEN);
    }

    Status = MmCopyToCaller(lpRect, &Rect, sizeof(RECT));
    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        goto Exit; // Return FALSE
    }

    Ret = TRUE;

Exit:
    TRACE("Leave NtUserGetClipCursor, ret=%i\n", Ret);
    UserLeave();
    return Ret;
}
/* --- pretranslation split: the remaining function groups live in cursoricon.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "cursoricon.split2.c"
