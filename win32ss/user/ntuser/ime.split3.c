/* ime.split3.c — #include fragment of ime.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original ime.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


// Searchs a non-IME-related window of the same thread of pwndTarget,
// other than pwndTarget, around pwndParent. Returns TRUE if found.
BOOL IntFindNonImeRelatedWndOfSameThread(PWND pwndParent, PWND pwndTarget)
{
    PWND pwnd, pwndOwner, pwndNode;
    PTHREADINFO ptiTarget = pwndTarget->head.pti;

    // For all the children of pwndParent, ...
    for (pwnd = pwndParent->spwndChild; pwnd; pwnd = pwnd->spwndNext)
    {
        if (pwnd == pwndTarget || pwnd->head.pti != ptiTarget || IS_WND_MENU(pwnd))
            continue;

        if (!IS_WND_CHILD(pwnd))
        {
            // Check if any IME-like owner.
            BOOL bFound1 = FALSE;
            for (pwndOwner = pwnd; pwndOwner; pwndOwner = pwndOwner->spwndOwner)
            {
                if (IS_WND_IMELIKE(pwndOwner))
                {
                    bFound1 = TRUE;
                    break;
                }
            }
            if (bFound1)
                continue; // Skip if any IME-like owner.
        }

        pwndNode = pwnd;

        if (IS_WND_CHILD(pwndNode))
        {
            // Check if any same-thread IME-like ancestor.
            BOOL bFound2 = FALSE;
            for (; IS_WND_CHILD(pwndNode); pwndNode = pwndNode->spwndParent)
            {
                if (pwndNode->head.pti != ptiTarget)
                    break;

                if (IS_WND_IMELIKE(pwndNode))
                {
                    bFound2 = TRUE;
                    break;
                }
            }
            if (bFound2)
                continue;
            // Now, pwndNode is non-child or non-same-thread window.
        }

        if (!IS_WND_CHILD(pwndNode)) // pwndNode is non-child
        {
            // Check if any same-thread IME-like owner.
            BOOL bFound3 = FALSE;
            for (; pwndNode; pwndNode = pwndNode->spwndOwner)
            {
                if (pwndNode->head.pti != ptiTarget)
                    break;

                if (IS_WND_IMELIKE(pwndNode))
                {
                    bFound3 = TRUE;
                    break;
                }
            }
            if (bFound3)
                continue;
        }

        return TRUE;
    }

    return FALSE;
}

// Determines whether the target window needs the IME window.
BOOL FASTCALL
IntWantImeWindow(_In_ PWND pwndTarget)
{
    PDESKTOP rpdesk;
    PWINSTATION_OBJECT rpwinstaParent;
    PWND pwndNode, pwndParent = pwndTarget->spwndParent;

    if (gptiCurrent->TIF_flags & TIF_DISABLEIME)
        return FALSE;

    if (IS_WND_IMELIKE(pwndTarget))
        return FALSE;

    if (pwndTarget->fnid == FNID_DESKTOP || pwndTarget->fnid == FNID_MESSAGEWND)
        return FALSE;

    if (pwndTarget->state & WNDS_SERVERSIDEWINDOWPROC)
        return FALSE;

    rpdesk = pwndTarget->head.rpdesk;
    if (!rpdesk)
        return FALSE;

    rpwinstaParent = rpdesk->rpwinstaParent;
    if (!rpwinstaParent || (rpwinstaParent->Flags & WSS_NOIO))
        return FALSE;

    for (pwndNode = pwndParent; pwndNode; pwndNode = pwndNode->spwndParent)
    {
        if (rpdesk != pwndNode->head.rpdesk)
            break;

        if (pwndNode == rpdesk->spwndMessage)
            return FALSE;
    }

    return TRUE;
}

// Create the default IME window for the target window.
PWND FASTCALL
co_IntCreateDefaultImeWindow(
    _In_ PWND pwndTarget,
    _In_ HINSTANCE hInst)
{
    LARGE_UNICODE_STRING WindowName;
    UNICODE_STRING ClassName;
    PWND pImeWnd;
    PIMEUI pimeui;
    CREATESTRUCTW Cs;
    USER_REFERENCE_ENTRY Ref;
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
    HANDLE pid = PsGetThreadProcessId(pti->pEThread);
    NTSTATUS Status;

    if (!(pti->spDefaultImc) && pid == gpidLogon)
        UserCreateInputContext(0);

    if (!(pti->spDefaultImc) || IS_WND_IMELIKE(pwndTarget) || !(pti->rpdesk->pheapDesktop))
        return NULL;

    if (IS_WND_CHILD(pwndTarget) && !(pwndTarget->style & WS_VISIBLE) &&
        pwndTarget->spwndParent->head.pti->ppi != pti->ppi)
    {
        return NULL;
    }

    RtlInitLargeUnicodeString(&WindowName, L"Default IME", 0);

    ClassName.Buffer = UlongToPtr(gpsi->atomSysClass[ICLS_IME]);
    ClassName.Length = 0;
    ClassName.MaximumLength = 0;

    UserRefObjectCo(pwndTarget, &Ref);

    RtlZeroMemory(&Cs, sizeof(Cs));
    Cs.style = WS_POPUP | WS_DISABLED;
    Cs.hInstance = hInst;
    Cs.hwndParent = UserHMGetHandle(pwndTarget);
    Cs.lpszName = WindowName.Buffer;
    Cs.lpszClass = ClassName.Buffer;

    // NOTE: LARGE_UNICODE_STRING is compatible to LARGE_STRING.
    pImeWnd = co_UserCreateWindowEx(&Cs, &ClassName, (PLARGE_STRING)&WindowName, NULL, WINVER);
    if (pImeWnd)
    {
        pimeui = IntGetImeUIFromWnd(pImeWnd);
        ASSERT(pimeui);
        Status = SafeProbeForWrite(pimeui, sizeof(*pimeui), 1);
        if (!NT_SUCCESS(Status))
        {
            ERR("Exception in co_IntCreateDefaultImeWindow: %p\n", pimeui);
        }
        else
        {
            pimeui->fDefault = TRUE;
            if (IS_WND_CHILD(pwndTarget) && pwndTarget->spwndParent->head.pti != pti)
                pimeui->fChildThreadDef = TRUE;
        }
    }

    UserDerefObjectCo(pwndTarget);
    return pImeWnd;
}


// Determines whether the system can destroy the default IME window for the target child window.
BOOL FASTCALL
IntImeCanDestroyDefIMEforChild(
    _In_ PWND pImeWnd,
    _In_ PWND pwndTarget)
{
    PWND pwndNode;
    PIMEUI pimeui;
    IMEUI SafeImeUI;
    NTSTATUS Status;

    pimeui = IntGetImeUIFromWnd(pImeWnd);
    if (!pimeui)
        return FALSE;

    // Check IMEUI.fChildThreadDef
    Status = SafeProbeForRead(pimeui, sizeof(IMEUI), 1);
    if (!NT_SUCCESS(Status))
    {
        ERR("Exception in IntImeCanDestroyDefIMEforChild: %p\n", pimeui);
    }
    else
    {
        SafeImeUI = *pimeui;
        if (!SafeImeUI.fChildThreadDef)
            return FALSE;
    }

    // The parent of pwndTarget is NULL or of the same thread of pwndTarget?
    if (pwndTarget->spwndParent == NULL ||
        pwndTarget->head.pti == pwndTarget->spwndParent->head.pti)
    {
        return FALSE;
    }

    for (pwndNode = pwndTarget; pwndNode; pwndNode = pwndNode->spwndParent)
    {
        if (pwndNode == pwndNode->head.rpdesk->pDeskInfo->spwnd)
            break;

        if (IntFindNonImeRelatedWndOfSameThread(pwndNode->spwndParent, pwndTarget))
            return FALSE;
    }

    return TRUE;
}


// Determines whether the system can destroy the default IME window for the non-child target window.
BOOL FASTCALL
IntImeCanDestroyDefIME(
    _In_ PWND pImeWnd,
    _In_ PWND pwndTarget)
{
    PWND pwndNode;
    PIMEUI pimeui;
    IMEUI SafeImeUI;
    NTSTATUS Status;

    pimeui = IntGetImeUIFromWnd(pImeWnd);
    if (!pimeui)
        return FALSE;

    // Check IMEUI.fDestroy
    Status = SafeProbeForRead(pimeui, sizeof(IMEUI), 1);
    if (!NT_SUCCESS(Status))
    {
        ERR("Exception in IntImeCanDestroyDefIME: %p\n", pimeui);
    }
    else
    {
        SafeImeUI = *pimeui;
        if (SafeImeUI.fDestroy)
            return FALSE;
    }

    // Any ancestor of pImeWnd is pwndTarget?
    if (pImeWnd->spwndOwner)
    {
        for (pwndNode = pImeWnd->spwndOwner; pwndNode; pwndNode = pwndNode->spwndOwner)
        {
            if (pwndNode == pwndTarget)
                break;
        }

        if (!pwndNode)
            return FALSE;
    }

    // Any ancestor of pwndTarget is IME-like?
    for (pwndNode = pwndTarget; pwndNode; pwndNode = pwndNode->spwndOwner)
    {
        if (IS_WND_IMELIKE(pwndNode))
            return FALSE;
    }

    // Adjust the ordering and top-mode status
    IntImeSetFutureOwner(pImeWnd, pwndTarget);
    for (pwndNode = pImeWnd->spwndOwner; pwndNode; pwndNode = pwndNode->spwndNext)
    {
        if (pwndNode == pImeWnd)
            break;
    }
    if (pwndNode == pImeWnd)
        IntImeCheckTopmost(pImeWnd);

    // Is the owner of pImeWnd NULL or pwndTarget?
    if (pImeWnd->spwndOwner && pwndTarget != pImeWnd->spwndOwner)
        return FALSE;

    WndSetOwner(pImeWnd, NULL);
    return TRUE;
}


// Update IMEUI.fShowStatus flags and Send the WM_IME_NOTIFY messages.
BOOL FASTCALL
IntCheckImeShowStatus(
    _In_ PWND pwndIme,
    _In_ PTHREADINFO pti)
{
    BOOL ret = FALSE, bDifferent;
    PWINDOWLIST pwl;
    HWND *phwnd;
    PWND pwndNode, pwndIMC;
    PTHREADINFO ptiCurrent = GetW32ThreadInfo();
    PIMEUI pimeui;
    IMEUI SafeImeUI;

    if (pwndIme->state2 & WNDS2_INDESTROY)
        return FALSE;

    // Build a window list
    pwl = IntBuildHwndList(pwndIme->spwndParent->spwndChild, IACE_LIST, NULL);
    if (!pwl)
        return FALSE;

    ret = TRUE;
    for (phwnd = pwl->ahwnd; *phwnd != HWND_TERMINATOR; ++phwnd)
    {
        pwndNode = ValidateHwndNoErr(*phwnd);

        if (!pwndNode || pwndIme == pwndNode)
            continue;

        if (pwndNode->pcls->atomClassName != gpsi->atomSysClass[ICLS_IME] ||
            (pwndNode->state2 & WNDS2_INDESTROY))
        {
            continue;
        }

        pimeui = IntGetImeUIFromWnd(pwndNode);
        if (!pimeui)
            continue;

        if (pti && pti != pwndNode->head.pti)
            continue;

        // Attach to the process if necessary
        bDifferent = FALSE;
        if (pwndNode->head.pti->ppi != ptiCurrent->ppi)
        {
            KeAttachProcess(&(pwndNode->head.pti->ppi->peProcess->Pcb));
            bDifferent = TRUE;
        }

        // Get pwndIMC and update IMEUI.fShowStatus flag
        _SEH2_TRY
        {
            ProbeForWrite(pimeui, sizeof(IMEUI), 1);
            SafeImeUI = *pimeui;
            if (SafeImeUI.fShowStatus)
            {
                pwndIMC = ValidateHwndNoErr(pimeui->hwndIMC);
                if (pwndIMC)
                    pimeui->fShowStatus = FALSE;
            }
            else
            {
                pwndIMC = NULL;
            }
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            ERR("Exception in IntCheckImeShowStatus: %p\n", pimeui);
            pwndIMC = NULL;
        }
        _SEH2_END;

        // Detach from the process if necessary
        if (bDifferent)
            KeDetachProcess();

        // Send the WM_IME_NOTIFY message
        if (pwndIMC && pwndIMC->head.pti && !(pwndIMC->head.pti->TIF_flags & TIF_INCLEANUP))
        {
            HWND hImeWnd;
            USER_REFERENCE_ENTRY Ref;

            UserRefObjectCo(pwndIMC, &Ref);

            hImeWnd = UserHMGetHandle(pwndIMC);
            co_IntSendMessage(hImeWnd, WM_IME_NOTIFY, IMN_CLOSESTATUSWINDOW, 0);

            UserDerefObjectCo(pwndIMC);
        }
    }

    // Free the window list
    IntFreeHwndList(pwl);
    return ret;
}

// Send a UI message.
LRESULT FASTCALL
IntSendMessageToUI(
    _In_ PTHREADINFO ptiIME,
    _In_ PIMEUI pimeui,
    _In_ UINT uMsg,
    _In_ WPARAM wParam,
    _In_ LPARAM lParam)
{
    PWND pwndUI;
    LRESULT ret = 0;
    IMEUI SafeImeUI;
    BOOL bDifferent = FALSE;
    USER_REFERENCE_ENTRY Ref;

    // Attach to the process if necessary
    if (ptiIME != GetW32ThreadInfo())
    {
        bDifferent = TRUE;
        KeAttachProcess(&(ptiIME->ppi->peProcess->Pcb));
    }

    // Get the pwndUI
    _SEH2_TRY
    {
        ProbeForRead(pimeui, sizeof(IMEUI), 1);
        SafeImeUI = *pimeui;
        pwndUI = ValidateHwndNoErr(SafeImeUI.hwndUI);
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception in IntSendMessageToUI: %p\n", pimeui);
        pwndUI = NULL;
    }
    _SEH2_END;

    if (!pwndUI)
        goto Quit;

    // Increment the recursion count of the IME procedure.
    // See also ImeWndProc_common of user32.
    _SEH2_TRY
    {
        ProbeForWrite(&pimeui->nCntInIMEProc, sizeof(LONG), 1);
        InterlockedIncrement(&pimeui->nCntInIMEProc);
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception in IntSendMessageToUI: %p\n", pimeui);
        _SEH2_YIELD(goto Quit);
    }
    _SEH2_END;

    // Detach from the process if necessary
    if (bDifferent)
        KeDetachProcess();

    UserRefObjectCo(pwndUI, &Ref);
    ret = co_IntSendMessage(UserHMGetHandle(pwndUI), uMsg, wParam, lParam);
    UserDerefObjectCo(pwndUI);

    // Attach to the process if necessary
    if (bDifferent)
        KeAttachProcess(&(ptiIME->ppi->peProcess->Pcb));

    // Decrement the recursion count of the IME procedure
    _SEH2_TRY
    {
        ProbeForWrite(&pimeui->nCntInIMEProc, sizeof(LONG), 1);
        InterlockedDecrement(&pimeui->nCntInIMEProc);
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception in IntSendMessageToUI: %p\n", pimeui);
        _SEH2_YIELD(goto Quit);
    }
    _SEH2_END;

Quit:
    // Detach from the process if necessary
    if (bDifferent)
        KeDetachProcess();

    return ret;
}

// Send the open status notification.
VOID FASTCALL
IntSendOpenStatusNotify(PTHREADINFO ptiIME, PIMEUI pimeui, PWND pWnd, BOOL bOpen)
{
    WPARAM wParam = (bOpen ? IMN_OPENSTATUSWINDOW : IMN_CLOSESTATUSWINDOW);
    PTHREADINFO ptiWnd = pWnd->head.pti;
    USER_REFERENCE_ENTRY Ref;

    if (ptiWnd->dwExpWinVer >= WINVER_WINNT4 && pWnd->hImc)
    {
        UserRefObjectCo(pWnd, &Ref);
        co_IntSendMessage(UserHMGetHandle(pWnd), WM_IME_NOTIFY, wParam, 0);
        UserDerefObjectCo(pWnd);
    }
    else
    {
        IntSendMessageToUI(ptiIME, pimeui, WM_IME_NOTIFY, wParam, 0);
    }
}

// Update the IME toolbar visibility and send a notification
VOID FASTCALL
IntNotifyImeShowStatus(_In_ PWND pImeWnd)
{
    PIMEUI pimeui;
    PWND pWnd;
    PTHREADINFO pti, ptiIME;
    BOOL bShow, bSendNotify = FALSE;
    IMEUI SafeImeUI;

    if (!IS_IMM_MODE() || (pImeWnd->state2 & WNDS2_INDESTROY))
        return;

    pti = PsGetCurrentThreadWin32Thread();
    ptiIME = pImeWnd->head.pti;

    pimeui = IntGetImeUIFromWnd(pImeWnd);
    if (!pimeui)
    {
        ERR("Invalid IMEWND %p\n", pImeWnd);
        return;
    }

    // Attach to the process if necessary
    if (pti != ptiIME)
        KeAttachProcess(&(ptiIME->ppi->peProcess->Pcb));

    // Get an IMEUI and check whether hwndIMC is valid and update fShowStatus
    _SEH2_TRY
    {
        ProbeForWrite(pimeui, sizeof(*pimeui), 1);
        SafeImeUI = *pimeui;

        bShow = (gfIMEShowStatus == TRUE) && SafeImeUI.fCtrlShowStatus;

        pWnd = ValidateHwndNoErr(SafeImeUI.hwndIMC);
        if (!pWnd)
            pWnd = ptiIME->MessageQueue->spwndFocus;

        if (pWnd)
        {
            bSendNotify = TRUE;
            pimeui->fShowStatus = bShow;
        }
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception in IntNotifyImeShowStatus: %p, %p, %p, %d, %d\n",
            pImeWnd, pimeui, ptiIME, SafeImeUI.fCtrlShowStatus, gfIMEShowStatus);

        if (pti != ptiIME)
            KeDetachProcess();

        _SEH2_YIELD(return);
    }
    _SEH2_END;

    // Detach from the process if necessary
    if (pti != ptiIME)
        KeDetachProcess();

    if (bSendNotify)
        IntSendOpenStatusNotify(ptiIME, &SafeImeUI, pWnd, bShow);

    if (!(pImeWnd->state2 & WNDS2_INDESTROY))
        IntCheckImeShowStatus(pImeWnd, NULL);
}

BOOL FASTCALL
IntBroadcastImeShowStatusChange(
    _In_ PWND pImeWnd,
    _In_ BOOL bShow)
{
    if (gfIMEShowStatus == bShow || !IS_IMM_MODE())
        return TRUE;

    gfIMEShowStatus = bShow;
    IntNotifyImeShowStatus(pImeWnd);
    return TRUE;
}

VOID FASTCALL
IntCheckImeShowStatusInThread(_In_ PWND pImeWnd)
{
    if (IS_IMM_MODE() && !(pImeWnd->state2 & WNDS2_INDESTROY))
        IntCheckImeShowStatus(pImeWnd, pImeWnd->head.pti);
}
