/* cursoricon.split2.c — #include fragment of cursoricon.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original cursoricon.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */



/*
 * @implemented
 */
HCURSOR
APIENTRY
NtUserSetCursor(
    HCURSOR hCursor)
{
    PCURICON_OBJECT pcurOld, pcurNew;
    HCURSOR hOldCursor = NULL;

    TRACE("Enter NtUserSetCursor: %p\n", hCursor);
    UserEnterExclusive();

    if (hCursor)
    {
        pcurNew = UserGetCurIconObject(hCursor);
        if (!pcurNew)
        {
            EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
            goto leave;
        }
        pcurNew->CURSORF_flags |= CURSORF_CURRENT;
    }
    else
    {
        pcurNew = NULL;
    }

    pcurOld = UserSetCursor(pcurNew, FALSE);

    // If returning an old cursor than validate it, Justin Case!
    if ( pcurOld &&
        (pcurOld = UserGetObjectNoErr(gHandleTable, UserHMGetHandle(pcurOld), TYPE_CURSOR)))
    {
        hOldCursor = UserHMGetHandle(pcurOld);
    /*
        Problem:

        System Global Cursors start out having at least 2 lock counts. If a system
        cursor is the default cursor and is returned to the caller twice in its
        life, the count will reach zero. Causing an assert to occur in objects.

        This fixes a SeaMonkey crash while the mouse crosses a boundary.
     */
        if (pcurOld->CURSORF_flags & CURSORF_GLOBAL)
        {
           TRACE("Returning Global Cursor hcur %p\n",hOldCursor);

           /*if (pcurOld->head.cLockObj > 2) // Throttle down to 2.
           {
              UserDereferenceObject(pcurOld);
           }

           goto leave;*/
        }

        /* See if it was destroyed in the meantime */
        if (UserObjectInDestroy(hOldCursor))
            hOldCursor = NULL;
        pcurOld->CURSORF_flags &= ~CURSORF_CURRENT;
        UserDereferenceObject(pcurOld);
    }

leave:
    UserLeave();
    return hOldCursor;
}


/*
 * @unimplemented
 */
BOOL
APIENTRY
NtUserSetCursorContents(
    HANDLE hCurIcon,
    PICONINFO UnsafeIconInfo)
{
    FIXME(" is UNIMPLEMENTED.\n");
    return FALSE;
}


static
BOOL
IntSetCursorData(
    _Inout_ PCURICON_OBJECT pcur,
    _In_opt_ PUNICODE_STRING pustrName,
    _In_ ATOM atomModName,
    _In_ const CURSORDATA* pcursordata)
{
    /* Check if the CURSORF_ACON is also set in the cursor data */
    if (pcursordata->CURSORF_flags & CURSORF_ACON)
    {
        ERR("Mismatch in CURSORF_flags! cursor: 0x%08lx, data:  0x%08lx\n",
            pcur->CURSORF_flags, pcursordata->CURSORF_flags);
        return FALSE;
    }

    /* Check if this cursor was already set */
    if (pcur->hbmMask != NULL)
    {
        ERR("Cursor data already set!\n");
        return FALSE;
    }

    /* We need a mask */
    if (pcursordata->hbmMask == NULL)
    {
        ERR("NtUserSetCursorIconData was got no hbmMask.\n");
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    /* Take ownership of the mask bitmap */
    if (!GreSetBitmapOwner(pcursordata->hbmMask, GDI_OBJ_HMGR_PUBLIC))
    {
        ERR("Failed to set ownership of hbmMask %p.\n", pcursordata->hbmMask);
        return FALSE;
    }

    /* Check if we have a color bitmap */
    if (pcursordata->hbmColor)
    {
        /* Take ownership of the color bitmap */
        if (!GreSetBitmapOwner(pcursordata->hbmColor, GDI_OBJ_HMGR_PUBLIC))
        {
            ERR("Failed to set ownership of hbmColor %p.\n", pcursordata->hbmColor);
            GreSetBitmapOwner(pcursordata->hbmMask, GDI_OBJ_HMGR_POWNED);
            return FALSE;
        }
    }

    /* Check if we have an alpha bitmap */
    if (pcursordata->hbmAlpha)
    {
        /* Take ownership of the alpha bitmap */
        if (!GreSetBitmapOwner(pcursordata->hbmAlpha, GDI_OBJ_HMGR_PUBLIC))
        {
            ERR("Failed to set ownership of hbmAlpha %p.\n", pcursordata->hbmAlpha);
            GreSetBitmapOwner(pcursordata->hbmMask, GDI_OBJ_HMGR_POWNED);
            if (pcursordata->hbmColor)
            {
                GreSetBitmapOwner(pcursordata->hbmColor, GDI_OBJ_HMGR_POWNED);
            }
            return FALSE;
        }
    }

    /* Free the old name (Must be NULL atm, but later we might allow this) */
    NT_ASSERT(pcur->strName.Buffer == NULL);
    if (pcur->strName.Buffer != NULL)
    {
        if (!IS_INTRESOURCE(pcur->strName.Buffer))
        {
            ExFreePoolWithTag(pcur->strName.Buffer, TAG_STRING);
        }
        RtlInitEmptyUnicodeString(&pcur->strName, NULL, 0);
    }

    /* Free the module atom */
    if (pcur->atomModName != 0)
    {
        NT_VERIFY(NT_SUCCESS(RtlDeleteAtomFromAtomTable(gAtomTable, pcur->atomModName)));
    }

    /* Now set the new cursor data */
    pcur->atomModName = atomModName;
    pcur->rt = pcursordata->rt;
    pcur->CURSORF_flags = pcursordata->CURSORF_flags & CURSORF_USER_MASK;
    pcur->xHotspot = pcursordata->xHotspot;
    pcur->yHotspot = pcursordata->yHotspot;
    pcur->hbmMask = pcursordata->hbmMask;
    pcur->hbmColor = pcursordata->hbmColor;
    pcur->hbmAlpha = pcursordata->hbmAlpha;
    pcur->rcBounds.left = 0;
    pcur->rcBounds.top = 0;
    pcur->rcBounds.right = pcursordata->cx;
    pcur->rcBounds.bottom = pcursordata->cy;
    pcur->hbmUserAlpha = pcursordata->hbmUserAlpha;
    pcur->bpp = pcursordata->bpp;
    pcur->cx = pcursordata->cx;
    pcur->cy = pcursordata->cy;
    if (pustrName != NULL)
    {
        pcur->strName = *pustrName;
    }

    return TRUE;
}

static
BOOL
IntSetAconData(
    _Inout_ PACON pacon,
    _In_opt_ PUNICODE_STRING pustrName,
    _In_ ATOM atomModName,
    _In_ const CURSORDATA *pcursordata)
{
    PCURICON_OBJECT *aspcur;
    DWORD *aicur;
    INT *ajifRate;
    PCURSORDATA pcdFrame;
    HCURSOR hcurFrame;
    UINT cjSize, i;

    NT_ASSERT((pacon->CURSORF_flags & CURSORF_ACON) != 0);
    NT_ASSERT((pacon->CURSORF_flags & CURSORF_ACONFRAME) == 0);
    NT_ASSERT((ULONG_PTR)pcursordata->aspcur > MmUserProbeAddress);
    NT_ASSERT((ULONG_PTR)pcursordata->aicur > MmUserProbeAddress);
    NT_ASSERT((ULONG_PTR)pcursordata->ajifRate > MmUserProbeAddress);
    NT_ASSERT((pcursordata->CURSORF_flags & ~CURSORF_USER_MASK) == 0);
    NT_ASSERT(pcursordata->cpcur > 0);
    NT_ASSERT(pcursordata->cicur > 0);

    /* Check if the CURSORF_ACON is also set in the cursor data */
    if (!(pcursordata->CURSORF_flags & CURSORF_ACON))
    {
        ERR("Mismatch in CURSORF_flags! acon: 0x%08lx, data:  0x%08lx\n",
            pacon->CURSORF_flags, pcursordata->CURSORF_flags);
        return FALSE;
    }

    /* Check if this acon was already set */
    if (pacon->aspcur != NULL)
    {
        ERR("Acon data already set!\n");
        return FALSE;
    }

    /* Loop all frames indexes */
    for (i = 0; i < pcursordata->cicur; i++)
    {
        /* Check if the index is within the range of the frames */
        if (pcursordata->aicur[i] >= pcursordata->cpcur)
        {
            ERR("aicur[%lu] is out or range. Got %lu, cpcur = %u\n",
                i, pcursordata->aicur[i], pcursordata->cpcur);
            return FALSE;
        }

        /* FIXME: check the JIF rates? */
    }

    /* Calculate size: one cursor object for each frame, and a frame
       index and jiffies for each "step" */
    cjSize = (pcursordata->cpcur * sizeof(CURICON_OBJECT*)) +
             (pcursordata->cicur * sizeof(DWORD)) +
             (pcursordata->cicur * sizeof(INT));

    /* Allocate a buffer */
    aspcur = ExAllocatePoolWithTag(PagedPool, cjSize, USERTAG_CURSOR);
    if (aspcur == NULL)
    {
        ERR("Failed to allocate memory (cpcur = %u, cicur = %u)\n",
            pcursordata->cpcur, pcursordata->cicur);
        return FALSE;
    }

    /* Set the pointers */
    aicur = (DWORD*)&aspcur[pcursordata->cpcur];
    ajifRate = (INT*)&aicur[pcursordata->cicur];

    /* Copy the values */
    RtlCopyMemory(aicur, pcursordata->aicur, pcursordata->cicur * sizeof(DWORD));
    RtlCopyMemory(ajifRate, pcursordata->ajifRate, pcursordata->cicur * sizeof(INT));

    /* Zero out the array, so we can handle cleanup */
    RtlZeroMemory(aspcur, pcursordata->cpcur * sizeof(PCURICON_OBJECT));

    /* Get a pointer to the cursor data for each frame */
    pcdFrame = pcursordata->aspcur;

    /* Create the cursors */
    for (i = 0; i < pcursordata->cpcur; i++)
    {
        /* Create a cursor for this frame */
        hcurFrame = IntCreateCurIconHandle(FALSE);
        if (hcurFrame == NULL)
        {
            ERR("Failed to create a cursor for frame %u\n", i);
            goto Cleanup;
        }

        /* Get a pointer to the frame cursor */
        aspcur[i] = UserGetCurIconObject(hcurFrame);
        _PRAGMA_WARNING_SUPPRESS(__WARNING_READ_OVERRUN);
        NT_ASSERT(aspcur[i] != NULL);

        /* Check if the flags are valid */
        if (pcdFrame->CURSORF_flags & ~(CURSORF_USER_MASK|CURSORF_ACONFRAME))
        {
            ERR("Invalid flags for acon frame %u: 0x%08lx\n",
                i, pcdFrame->CURSORF_flags);
            goto Cleanup;
        }

        /* Set the cursor data for this frame */
        if (!IntSetCursorData(aspcur[i], NULL, 0, &pcdFrame[i]))
        {
            ERR("Failed to set cursor data for frame %u\n", i);
            goto Cleanup;
        }

        /* Mark this cursor as an acon frame */
        aspcur[i]->CURSORF_flags |= CURSORF_ACONFRAME;
    }

    /* Free the old name (Must be NULL atm.) */
    NT_ASSERT(pacon->strName.Buffer == NULL);
    if (pacon->strName.Buffer != NULL)
    {
        if (!IS_INTRESOURCE(pacon->strName.Buffer))
        {
            ExFreePoolWithTag(pacon->strName.Buffer, TAG_STRING);
        }
        RtlInitEmptyUnicodeString(&pacon->strName, NULL, 0);
    }

    /* Free the module atom */
    if (pacon->atomModName != 0)
    {
        NT_VERIFY(NT_SUCCESS(RtlDeleteAtomFromAtomTable(gAtomTable, pacon->atomModName)));
    }

    /* Free the previous frames */
    if (pacon->aspcur != NULL)
    {
        for (i = 0; i < pacon->cpcur; i++)
        {
            UserDereferenceObject(pacon->aspcur[i]);
            NT_VERIFY(IntDestroyCurIconObject(pacon->aspcur[i]) == TRUE);
        }
        ExFreePoolWithTag(pacon->aspcur, USERTAG_CURSOR);
    }

    /* Finally set the data in the acon */
    pacon->atomModName = atomModName;
    pacon->rt = pcursordata->rt;
    pacon->CURSORF_flags = pcursordata->CURSORF_flags & CURSORF_USER_MASK;
    pacon->cpcur = pcursordata->cpcur;
    pacon->cicur = pcursordata->cicur;
    pacon->aspcur = aspcur;
    pacon->aicur = aicur;
    pacon->ajifRate = ajifRate;
    pacon->iicur = 0;
    if (pustrName != NULL)
    {
        pacon->strName = *pustrName;
    }

    return TRUE;

Cleanup:

    /* Clean up the cursors we created */
    for (i = 0; i < pcursordata->cpcur; i++)
    {
        if (aspcur[i] == NULL)
            break;

        /* Destroy this cursor */
        UserDereferenceObject(aspcur[i]);
        NT_VERIFY(IntDestroyCurIconObject(aspcur[i]) == TRUE);
    }

    /* Delete the allocated structure */
    ExFreePoolWithTag(aspcur, USERTAG_CURSOR);

    return FALSE;
}

BOOL
APIENTRY
UserSetCursorIconData(
    _In_ HCURSOR hcursor,
    _In_opt_ PUNICODE_STRING pustrModule,
    _In_opt_ PUNICODE_STRING pustrRsrc,
    _In_ PCURSORDATA pcursordata)
{
    PCURICON_OBJECT pcur;
    ATOM atomModName;
    NTSTATUS status;
    BOOL bResult;

    /* Do we have a module name? */
    if (pustrModule != NULL)
    {
        /* Create an atom for the module name */
        status = RtlAddAtomToAtomTable(gAtomTable,
                                       pustrModule->Buffer,
                                       &atomModName);
        if (!NT_SUCCESS(status))
        {
            ERR("Failed to create atom from module name '%wZ': 0x%08lx\n",
                pustrModule, status);
            return FALSE;
        }
    }
    else
    {
        /* No module name atom */
        atomModName = 0;
    }

    /* Reference the cursor */
    pcur = UserGetCurIconObject(hcursor);
    if (pcur == NULL)
    {
        ERR("Failed to reference cursor %p\n", hcursor);
        bResult = FALSE;
        goto Exit;
    }

    /* Check if this is an acon */
    if (pcur->CURSORF_flags & CURSORF_ACON)
    {
        bResult = IntSetAconData((PACON)pcur,
                                 pustrRsrc,
                                 atomModName,
                                 pcursordata);
    }
    else
    {
        bResult = IntSetCursorData(pcur,
                                   pustrRsrc,
                                   atomModName,
                                   pcursordata);
    }

Exit:

    /* Check if we had success */
    if (bResult != FALSE)
    {
        /* Check if this is an LRSHARED cursor now */
        if (pcur->CURSORF_flags & CURSORF_LRSHARED)
        {
            /* Insert the cursor into the list. */
            IntInsertCursorIntoList(pcur);
        }
    }
    else
    {
        /* Cleanup on failure */
        if (atomModName != 0)
        {
            NT_VERIFY(NT_SUCCESS(RtlDeleteAtomFromAtomTable(gAtomTable, atomModName)));
        }
    }

    /* Dereference the cursor and return the result */
    if (pcur)
        UserDereferenceObject(pcur);

    return bResult;
}


/*
 * @implemented
 */
__kernel_entry
BOOL
APIENTRY
NtUserSetCursorIconData(
    _In_ HCURSOR hcursor,
    _In_opt_ PUNICODE_STRING pustrModule,
    _In_opt_ PUNICODE_STRING pustrRsrc,
    _In_ const CURSORDATA* pCursorData)
{
    CURSORDATA cursordata;
    UNICODE_STRING ustrModule, ustrRsrc;
    _SEH2_VOLATILE PVOID pvBuffer;
    CURSORDATA* aspcur;
    DWORD* aicur;
    PINT ajifRate;
    UINT cjSize;
    NTSTATUS status;
    BOOL bResult = FALSE;

    TRACE("Enter NtUserSetCursorIconData\n");

    /* Initialize buffer, so we can handle cleanup */
    ustrRsrc.Buffer = NULL;
    ustrModule.Buffer = NULL;
    pvBuffer = NULL;

    _SEH2_TRY
    {
        /* Probe and capture the cursor data structure */
        ProbeForRead(pCursorData, sizeof(*pCursorData), 1);
        cursordata = *pCursorData;

        /* Check if this is an animated cursor */
        if (cursordata.CURSORF_flags & CURSORF_ACON)
        {
            /* Check of the range is ok */
            if ((cursordata.cpcur == 0) || (cursordata.cicur == 0) ||
                (cursordata.cpcur > 1000) || (cursordata.cicur > 1000))
            {
                ERR("Range error (cpcur = %u, cicur = %u)\n",
                    cursordata.cpcur, cursordata.cicur);
                goto Exit;
            }

            /* Calculate size: one cursor data structure for each frame,
               and a frame index and jiffies for each "step" */
            cjSize = (cursordata.cpcur * sizeof(CURSORDATA)) +
                     (cursordata.cicur * sizeof(DWORD)) +
                     (cursordata.cicur * sizeof(INT));

            /* Allocate a buffer */
            pvBuffer = ExAllocatePoolWithTag(PagedPool, cjSize, USERTAG_CURSOR);
            if (pvBuffer == NULL)
            {
                ERR("Failed to allocate memory (cpcur = %u, cicur = %u)\n",
                    cursordata.cpcur, cursordata.cicur);
                goto Exit;
            }

            /* Calculate the kernel mode pointers */
            aspcur = (CURSORDATA*)pvBuffer;
            aicur = (DWORD*)&aspcur[cursordata.cpcur];
            ajifRate = (INT*)&aicur[cursordata.cicur];

            /* Probe and copy aspcur */
            ProbeForRead(cursordata.aspcur, cursordata.cpcur * sizeof(CURSORDATA), 1);
            RtlCopyMemory(aspcur,
                          cursordata.aspcur,
                          cursordata.cpcur * sizeof(CURSORDATA));

            /* Probe and copy aicur */
            ProbeForRead(cursordata.aicur, cursordata.cicur * sizeof(DWORD), 1);
            RtlCopyMemory(aicur,
                          cursordata.aicur,
                          cursordata.cicur * sizeof(DWORD));

            /* Probe and copy ajifRate */
            ProbeForRead(cursordata.ajifRate, cursordata.cicur * sizeof(INT), 1);
            RtlCopyMemory(ajifRate,
                          cursordata.ajifRate,
                          cursordata.cicur * sizeof(INT));

            /* Set the new pointers */
            cursordata.aspcur = aspcur;
            cursordata.aicur = aicur;
            cursordata.ajifRate = ajifRate;
        }
        else
        {
            /* This is a standard cursor, we don't use the pointers */
            cursordata.aspcur = NULL;
            cursordata.aicur = NULL;
            cursordata.ajifRate = NULL;
        }
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        SetLastNtError(_SEH2_GetExceptionCode());
        goto Exit;
    }
    _SEH2_END

    /* Check if we got a module name */
    if (pustrModule != NULL)
    {
        /* Capture the name */
        status = ProbeAndCaptureUnicodeString(&ustrModule, UserMode, pustrModule);
        if (!NT_SUCCESS(status))
        {
            ERR("Failed to copy pustrModule: status 0x%08lx\n", status);
            goto Exit;
        }
    }

    /* Check if we got a resource name */
    if (pustrRsrc != NULL)
    {
        /* We use this function, because INTRESOURCEs and ATOMs are the same */
        status = ProbeAndCaptureUnicodeStringOrAtom(&ustrRsrc, pustrRsrc);
        if (!NT_SUCCESS(status))
        {
            ERR("Failed to copy pustrRsrc: status 0x%08lx\n", status);
            goto Exit;
        }
    }

    /* Make sure the caller doesn't give us invalid flags */
    if (cursordata.CURSORF_flags & ~CURSORF_USER_MASK)
    {
        ERR("Invalid cursor flags: 0x%08lx\n", cursordata.CURSORF_flags);
        goto Exit;
    }

    /* Acquire the global user lock */
    UserEnterExclusive();

    /* Call the internal function */
    bResult = UserSetCursorIconData(hcursor,
                                    pustrModule ? &ustrModule : NULL,
                                    pustrRsrc ? &ustrRsrc : NULL,
                                    &cursordata);

    /* Release the global user lock */
    UserLeave();

Exit:

    /* Free the captured module name */
    if ((ustrModule.Buffer != NULL) && !IS_INTRESOURCE(ustrModule.Buffer))
    {
        ReleaseCapturedUnicodeString(&ustrModule, UserMode);
    }

    if (pvBuffer != NULL)
    {
        ExFreePoolWithTag(pvBuffer, USERTAG_CURSOR);
    }

    /* Additional cleanup on failure */
    if (bResult == FALSE)
    {
        if ((ustrRsrc.Buffer != NULL) &&
            !IS_INTRESOURCE(ustrRsrc.Buffer))
        {
            ExFreePoolWithTag(ustrRsrc.Buffer, TAG_STRING);
        }
    }

    TRACE("Leave NtUserSetCursorIconData, bResult = %i\n", bResult);

    return bResult;
}

/* Mostly inspired from wine code.
 * We use low level functions because:
 *  - at this point, the icon bitmap could have a different bit depth than the DC,
 *    making it thus impossible to use NtCreateCompatibleDC and selecting the bitmap.
 *    This happens after a mode setting change.
 *  - it avoids massive GDI objects locking when only the destination surface needs it.
 *  - It makes (small) performance gains.
 */
BOOL
UserDrawIconEx(
    HDC hDc,
    INT xLeft,
    INT yTop,
    PCURICON_OBJECT pIcon,
    INT cxWidth,
    INT cyHeight,
    UINT istepIfAniCur,
    HBRUSH hbrFlickerFreeDraw,
    UINT diFlags)
{
    PSURFACE psurfDest, psurfMask, psurfColor; //, psurfOffScreen = NULL;
    PDC pdc = NULL;
    BOOL Ret = FALSE;
    HBITMAP hbmMask, hbmColor, hbmAlpha;
    BOOL bOffScreen;
    RECTL rcDest, rcSrc;
    CLIPOBJ* pdcClipObj = NULL;
    EXLATEOBJ exlo;

    /* Stupid case */
    if ((diFlags & DI_NORMAL) == 0)
    {
        ERR("DrawIconEx called without mask or color bitmap to draw.\n");
        return FALSE;
    }

    if (pIcon->CURSORF_flags & CURSORF_ACON)
    {
        ACON* pAcon = (ACON*)pIcon;
        if (istepIfAniCur >= pAcon->cicur)
        {
            ERR("NtUserDrawIconEx: istepIfAniCur too big!\n");
            return FALSE;
        }
        pIcon = pAcon->aspcur[pAcon->aicur[istepIfAniCur]];
    }

    hbmMask = pIcon->hbmMask;
    hbmColor = pIcon->hbmColor;
    hbmAlpha = pIcon->hbmAlpha;

    /*
     * Get our objects.
     * Shared locks are enough, we are only reading those bitmaps
     */
    psurfMask = SURFACE_ShareLockSurface(hbmMask);
    if (psurfMask == NULL)
    {
        ERR("Unable to lock the mask surface.\n");
        return FALSE;
    }

    /* Color bitmap is not mandatory */
    if (hbmColor == NULL)
    {
        /* But then the mask bitmap must have the information in it's bottom half */
        ASSERT(psurfMask->SurfObj.sizlBitmap.cy == 2*pIcon->cy);
        psurfColor = NULL;
    }
    else if ((psurfColor = SURFACE_ShareLockSurface(hbmColor)) == NULL)
    {
        ERR("Unable to lock the color bitmap.\n");
        SURFACE_ShareUnlockSurface(psurfMask);
        return FALSE;
    }

    pdc = DC_LockDc(hDc);
    if (!pdc)
    {
        ERR("Could not lock the destination DC.\n");
        SURFACE_ShareUnlockSurface(psurfMask);
        if (psurfColor) SURFACE_ShareUnlockSurface(psurfColor);
        return FALSE;
    }

    /* Fix width parameter, if needed */
    if (!cxWidth)
    {
        if (diFlags & DI_DEFAULTSIZE)
            cxWidth = is_icon(pIcon) ?
                UserGetSystemMetrics(SM_CXICON) : UserGetSystemMetrics(SM_CXCURSOR);
        else
            cxWidth = pIcon->cx;
    }

    /* Fix height parameter, if needed */
    if (!cyHeight)
    {
        if (diFlags & DI_DEFAULTSIZE)
            cyHeight = is_icon(pIcon) ?
                UserGetSystemMetrics(SM_CYICON) : UserGetSystemMetrics(SM_CYCURSOR);
        else
            cyHeight = pIcon->cy;
    }

    /* Calculate destination rectangle */
    RECTL_vSetRect(&rcDest, xLeft, yTop, xLeft + cxWidth, yTop + cyHeight);
    IntLPtoDP(pdc, (LPPOINT)&rcDest, 2);
    RECTL_vOffsetRect(&rcDest, pdc->ptlDCOrig.x, pdc->ptlDCOrig.y);

    /* Prepare the underlying surface */
    DC_vPrepareDCsForBlit(pdc, &rcDest, NULL, NULL);

    /* We now have our destination surface and rectangle */
    psurfDest = pdc->dclevel.pSurface;

    if (psurfDest == NULL)
    {
        /* Empty DC */
        DC_vFinishBlit(pdc, NULL);
        DC_UnlockDc(pdc);
        SURFACE_ShareUnlockSurface(psurfMask);
        if (psurfColor) SURFACE_ShareUnlockSurface(psurfColor);
        return FALSE;
    }

    /* Set source rect */
    RECTL_vSetRect(&rcSrc, 0, 0, pIcon->cx, pIcon->cy);

    /* Should we render off-screen? */
    bOffScreen = hbrFlickerFreeDraw &&
        (GDI_HANDLE_GET_TYPE(hbrFlickerFreeDraw) == GDI_OBJECT_TYPE_BRUSH);

    if (bOffScreen)
    {
        /* Yes: Allocate and paint the offscreen surface */
        EBRUSHOBJ eboFill;
        PBRUSH pbrush = BRUSH_ShareLockBrush(hbrFlickerFreeDraw);

        TRACE("Performing off-screen rendering.\n");

        if (!pbrush)
        {
            ERR("Failed to get brush object.\n");
            goto Cleanup;
        }

#if 0 //We lock the hdc surface during the whole function it makes no sense to use an offscreen surface for "flicker free" drawing
        psurfOffScreen = SURFACE_AllocSurface(STYPE_BITMAP,
            cxWidth, cyHeight, psurfDest->SurfObj.iBitmapFormat,
            0, 0, NULL);
        if (!psurfOffScreen)
        {
            ERR("Failed to allocate the off-screen surface.\n");
            BRUSH_ShareUnlockBrush(pbrush);
            goto Cleanup;
        }

        /* Paint the brush */
        EBRUSHOBJ_vInit(&eboFill, pbrush, psurfOffScreen, 0x00FFFFFF, 0, NULL);
        RECTL_vSetRect(&rcDest, 0, 0, cxWidth, cyHeight);

        Ret = IntEngBitBlt(&psurfOffScreen->SurfObj,
            NULL,
            NULL,
            NULL,
            NULL,
            &rcDest,
            NULL,
            NULL,
            &eboFill.BrushObject,
            &pbrush->ptOrigin,
            ROP4_PATCOPY);

        /* Clean up everything */
        EBRUSHOBJ_vCleanup(&eboFill);
        BRUSH_ShareUnlockBrush(pbrush);

        if (!Ret)
        {
            ERR("Failed to paint the off-screen surface.\n");
            goto Cleanup;
        }

        /* We now have our destination surface */
        psurfDest = psurfOffScreen;
#else
        pdcClipObj = (CLIPOBJ *)&pdc->co;
        /* Paint the brush */
        EBRUSHOBJ_vInit(&eboFill, pbrush, psurfDest, 0x00FFFFFF, 0, NULL);

        Ret = IntEngBitBlt(&psurfDest->SurfObj,
            NULL,
            NULL,
            pdcClipObj,
            NULL,
            &rcDest,
            NULL,
            NULL,
            &eboFill.BrushObject,
            &pbrush->ptOrigin,
            ROP4_PATCOPY);

        /* Clean up everything */
        EBRUSHOBJ_vCleanup(&eboFill);
        BRUSH_ShareUnlockBrush(pbrush);

        if (!Ret)
        {
            ERR("Failed to paint the off-screen surface.\n");
            goto Cleanup;
        }
#endif
    }
    else
    {
        /* We directly draw to the DC */
        TRACE("Performing on screen rendering.\n");
        pdcClipObj = (CLIPOBJ *)&pdc->co;
        // psurfOffScreen = NULL;
    }

    /* Now do the rendering */
    if (hbmAlpha && ((diFlags & DI_NORMAL) == DI_NORMAL))
    {
        BLENDOBJ blendobj = { {AC_SRC_OVER, 0, 255, AC_SRC_ALPHA } };
        PSURFACE psurf = NULL;

        psurf = SURFACE_ShareLockSurface(hbmAlpha);
        if (!psurf)
        {
            ERR("SURFACE_LockSurface failed!\n");
            goto NoAlpha;
        }

        /* Initialize color translation object */
        EXLATEOBJ_vInitialize(&exlo, psurf->ppal, psurfDest->ppal, 0xFFFFFFFF, 0xFFFFFFFF, 0);

        /* Now do it */
        Ret = IntEngAlphaBlend(&psurfDest->SurfObj,
                               &psurf->SurfObj,
                               pdcClipObj,
                               &exlo.xlo,
                               &rcDest,
                               &rcSrc,
                               &blendobj);

        EXLATEOBJ_vCleanup(&exlo);
        SURFACE_ShareUnlockSurface(psurf);
        if (Ret) goto done;
        ERR("NtGdiAlphaBlend failed!\n");
    }
NoAlpha:
    if (diFlags & DI_MASK)
    {
        DWORD rop4 = (diFlags & DI_IMAGE) ? ROP4_SRCAND : ROP4_SRCCOPY;

        EXLATEOBJ_vInitSrcMonoXlate(&exlo, psurfDest->ppal, 0x00FFFFFF, 0);

        Ret = IntEngStretchBlt(&psurfDest->SurfObj,
                               &psurfMask->SurfObj,
                               NULL,
                               pdcClipObj,
                               &exlo.xlo,
                               NULL,
                               &rcDest,
                               &rcSrc,
                               NULL,
                               NULL,
                               NULL,
                               rop4);

        EXLATEOBJ_vCleanup(&exlo);

        if (!Ret)
        {
            ERR("Failed to mask the bitmap data.\n");
            goto Cleanup;
        }
    }

    if (diFlags & DI_IMAGE)
    {
        if (psurfColor)
        {
            DWORD rop4 = (diFlags & DI_MASK) ? ROP4_SRCINVERT : ROP4_SRCCOPY ;

            EXLATEOBJ_vInitialize(&exlo, psurfColor->ppal, psurfDest->ppal, 0x00FFFFFF, 0x00FFFFFF, 0);

            Ret = IntEngStretchBlt(&psurfDest->SurfObj,
                                   &psurfColor->SurfObj,
                                   NULL,
                                   pdcClipObj,
                                   &exlo.xlo,
                                   NULL,
                                   &rcDest,
                                   &rcSrc,
                                   NULL,
                                   NULL,
                                   NULL,
                                   rop4);

            EXLATEOBJ_vCleanup(&exlo);

            if (!Ret)
            {
                ERR("Failed to render the icon bitmap.\n");
                goto Cleanup;
            }
        }
        else
        {
            /* Mask bitmap holds the information in its bottom half */
            DWORD rop4 = (diFlags & DI_MASK) ? ROP4_SRCINVERT : ROP4_SRCCOPY;
            RECTL_vOffsetRect(&rcSrc, 0, pIcon->cy);

            EXLATEOBJ_vInitSrcMonoXlate(&exlo, psurfDest->ppal, 0x00FFFFFF, 0);

            Ret = IntEngStretchBlt(&psurfDest->SurfObj,
                                   &psurfMask->SurfObj,
                                   NULL,
                                   pdcClipObj,
                                   &exlo.xlo,
                                   NULL,
                                   &rcDest,
                                   &rcSrc,
                                   NULL,
                                   NULL,
                                   NULL,
                                   rop4);

            EXLATEOBJ_vCleanup(&exlo);

            if (!Ret)
            {
                ERR("Failed to render the icon bitmap.\n");
                goto Cleanup;
            }
        }
    }

done:
#if 0
    /* We're done. Was it a double buffered draw ? */
    if (bOffScreen)
    {
        /* Yes. Draw it back to our DC */
        POINTL ptSrc = {0, 0};

        /* Calculate destination rectangle */
        RECTL_vSetRect(&rcDest, xLeft, yTop, xLeft + cxWidth, yTop + cyHeight);
        IntLPtoDP(pdc, (LPPOINT)&rcDest, 2);
        RECTL_vOffsetRect(&rcDest, pdc->ptlDCOrig.x, pdc->ptlDCOrig.y);

        /* Get the clip object */
        pdcClipObj = pdc->rosdc.CombinedClip;

        /* We now have our destination surface and rectangle */
        psurfDest = pdc->dclevel.pSurface;

        /* Color translation */
        EXLATEOBJ_vInitialize(&exlo, psurfOffScreen->ppal, psurfDest->ppal, 0x00FFFFFF, 0x00FFFFFF, 0);

        /* Blt it! */
        Ret = IntEngBitBlt(&psurfDest->SurfObj,
                           &psurfOffScreen->SurfObj,
                           NULL,
                           pdcClipObj,
                           &exlo.xlo,
                           &rcDest,
                           &ptSrc,
                           NULL,
                           NULL,
                           NULL,
                           ROP4_SRCCOPY);

        EXLATEOBJ_vCleanup(&exlo);
    }
#endif
Cleanup:
    if (pdc)
    {
        DC_vFinishBlit(pdc, NULL);
        DC_UnlockDc(pdc);
    }

#if 0
    /* Delete off screen rendering surface */
    if (psurfOffScreen)
        GDIOBJ_vDeleteObject(&psurfOffScreen->BaseObject);
#endif

    /* Unlock other surfaces */
    SURFACE_ShareUnlockSurface(psurfMask);
    if (psurfColor) SURFACE_ShareUnlockSurface(psurfColor);

    return Ret;
}

/*
 * @implemented
 */
BOOL
APIENTRY
NtUserDrawIconEx(
    HDC hdc,
    int xLeft,
    int yTop,
    HICON hIcon,
    int cxWidth,
    int cyHeight,
    UINT istepIfAniCur,
    HBRUSH hbrFlickerFreeDraw,
    UINT diFlags,
    BOOL bMetaHDC, // When TRUE, GDI functions need to be handled in User32!
    PVOID pDIXData)
{
    PCURICON_OBJECT pIcon;
    BOOL Ret;

    TRACE("Enter NtUserDrawIconEx\n");
    UserEnterExclusive();

    if (!(pIcon = UserGetCurIconObject(hIcon)))
    {
        ERR("UserGetCurIconObject(0x%p) failed!\n", hIcon);
        UserLeave();
        return FALSE;
    }

    Ret = UserDrawIconEx(hdc,
                         xLeft,
                         yTop,
                         pIcon,
                         cxWidth,
                         cyHeight,
                         istepIfAniCur,
                         hbrFlickerFreeDraw,
                         diFlags);

    UserDereferenceObject(pIcon);

    UserLeave();
    return Ret;
}

/*
 * @unimplemented
 */
HCURSOR
NTAPI
NtUserGetCursorFrameInfo(
    HCURSOR hCursor,
    DWORD istep,
    INT* rate_jiffies,
    DWORD* num_steps)
{
    PCURICON_OBJECT CurIcon;
    HCURSOR ret;
    INT jiffies = 0;
    DWORD steps = 1;
    NTSTATUS Status = STATUS_SUCCESS;

    TRACE("Enter NtUserGetCursorFrameInfo\n");
    UserEnterShared();

    if (!(CurIcon = UserGetCurIconObject(hCursor)))
    {
        UserLeave();
        return NULL;
    }

    ret = UserHMGetHandle(CurIcon);

    if (CurIcon->CURSORF_flags & CURSORF_ACON)
    {
        PACON AniCurIcon = (PACON)CurIcon;
        if (istep >= AniCurIcon->cicur)
        {
            UserDereferenceObject(CurIcon);
            UserLeave();
            return NULL;
        }
        jiffies = AniCurIcon->ajifRate[istep];
        steps = AniCurIcon->cicur;
        ret = UserHMGetHandle(AniCurIcon->aspcur[AniCurIcon->aicur[istep]]);
    }

    Status = SafeProbeForWrite(rate_jiffies, sizeof(INT), 1);
    if (NT_SUCCESS(Status))
        Status = SafeProbeForWrite(num_steps, sizeof(DWORD), 1);
    if (NT_SUCCESS(Status))
    {
        *rate_jiffies = jiffies;
        *num_steps = steps;
    }

    if (!NT_SUCCESS(Status))
    {
        WARN("Status: 0x%08lx\n", Status);
        SetLastNtError(Status);
        ret = NULL;
    }

    UserDereferenceObject(CurIcon);
    UserLeave();

    TRACE("Leaving NtUserGetCursorFrameInfo, ret = 0x%p\n", ret);

    return ret;
}

/*
 * @implemented
 */
BOOL
APIENTRY
NtUserSetSystemCursor(
    HCURSOR hcur,
    DWORD id)
{
    PCURICON_OBJECT pcur, pcurOrig = NULL;
    int i;
    PPROCESSINFO ppi;
    BOOL Ret = FALSE;
    UserEnterExclusive();

    if (!CheckWinstaAttributeAccess(WINSTA_WRITEATTRIBUTES))
    {
        goto Exit;
    }

    if (hcur)
    {
        pcur = UserGetCurIconObject(hcur);
        if (!pcur)
        {
            EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
            goto Exit;
        }

        ppi = PsGetCurrentProcessWin32Process();

        for (i = 0 ; i < 16; i++)
        {
           if (gasyscur[i].type == id)
           {
              pcurOrig = gasyscur[i].handle;

              if (pcurOrig) break;

              if (ppi->W32PF_flags & W32PF_CREATEDWINORDC)
              {
                 gasyscur[i].handle = pcur;
                 pcur->CURSORF_flags |= CURSORF_GLOBAL;
                 pcur->head.ppi = NULL;
                 IntInsertCursorIntoList(pcur);
                 Ret = TRUE;
              }
              break;
           }
        }
        if (pcurOrig)
        {
           FIXME("Need to copy cursor data or do something! pcurOrig %p new pcur %p\n",pcurOrig,pcur);
        }
    }
Exit:
    UserLeave();
    return Ret;
}

/* EOF */
