/* ime.split2.c — #include fragment of ime.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original ime.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


DWORD
NTAPI
NtUserGetAppImeLevel(_In_ HWND hWnd)
{
    DWORD ret = 0;
    PWND pWnd;
    PTHREADINFO pti;

    UserEnterShared();

    pWnd = ValidateHwndNoErr(hWnd);
    if (!pWnd)
    {
        ERR("Invalid HWND %p\n", hWnd);
        goto Quit;
    }

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        goto Quit;
    }

    pti = PsGetCurrentThreadWin32Thread();
    if (pWnd->head.pti->ppi == pti->ppi)
        ret = HandleToUlong(UserGetProp(pWnd, AtomImeLevel, TRUE));

Quit:
    UserLeave();
    return ret;
}

BOOL FASTCALL
UserGetImeInfoEx(
    _Inout_ PWINSTATION_OBJECT pWinSta,
    _Inout_ PIMEINFOEX pInfoEx,
    _In_ IMEINFOEXCLASS SearchType)
{
    PKL pkl, pklHead;

    if (!pWinSta || !gspklBaseLayout)
    {
        ERR("pWinSta:%p, gspklBaseLayout:%p\n", pWinSta, gspklBaseLayout);
        return FALSE;
    }

    pkl = pklHead = gspklBaseLayout;

    /* Find the matching entry from the list and get info */
    if (SearchType == ImeInfoExKeyboardLayout)
    {
        do
        {
            if (pInfoEx->hkl == pkl->hkl)
            {
                if (!pkl->piiex)
                {
                    ERR("!pkl->piiex at %p\n", pkl->hkl);
                    break;
                }

                *pInfoEx = *pkl->piiex;
                return TRUE;
            }

            pkl = pkl->pklNext;
        } while (pkl != pklHead);

        ERR("HKL not found: %p\n", pInfoEx->hkl);
    }
    else if (SearchType == ImeInfoExImeFileName)
    {
        do
        {
            if (pkl->piiex &&
                _wcsnicmp(pkl->piiex->wszImeFile, pInfoEx->wszImeFile,
                          RTL_NUMBER_OF(pkl->piiex->wszImeFile)) == 0)
            {
                *pInfoEx = *pkl->piiex;
                return TRUE;
            }

            pkl = pkl->pklNext;
        } while (pkl != pklHead);

        ERR("wszImeFile not found: '%S'\n", pInfoEx->wszImeFile);
    }
    else
    {
        ERR("SearchType: %d\n", SearchType);
    }

    return FALSE;
}

BOOL
NTAPI
NtUserGetImeInfoEx(
    _Inout_ PIMEINFOEX pImeInfoEx,
    _In_ IMEINFOEXCLASS SearchType)
{
    IMEINFOEX ImeInfoEx;
    BOOL ret = FALSE;
    PWINSTATION_OBJECT pWinSta;
    NTSTATUS Status;

    UserEnterShared();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        goto Quit;
    }

    {
        NTSTATUS __SehStatus = SafeProbeForRead(pImeInfoEx, sizeof(*pImeInfoEx), 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            ERR("Exception in NtUserGetImeInfoEx: %p\n", pImeInfoEx);
            goto Quit;
        }
    }
    ImeInfoEx = *pImeInfoEx;

    pWinSta = IntGetProcessWindowStation(NULL);
    ret = UserGetImeInfoEx(pWinSta, &ImeInfoEx, SearchType);
    if (!ret)
        goto Quit;

    Status = SafeProbeForWrite(pImeInfoEx, sizeof(*pImeInfoEx), 1);
    if (!NT_SUCCESS(Status))
    {
        ERR("Exception in NtUserGetImeInfoEx: %p\n", pImeInfoEx);
        ret = FALSE;
    }
    else
    {
        *pImeInfoEx = ImeInfoEx;
    }

Quit:
    UserLeave();
    return ret;
}


BOOL
NTAPI
NtUserSetAppImeLevel(
    _In_ HWND hWnd,
    _In_ DWORD dwLevel)
{
    BOOL ret = FALSE;
    PWND pWnd;
    PTHREADINFO pti;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        goto Quit;
    }

    pWnd = ValidateHwndNoErr(hWnd);
    if (!pWnd)
    {
        ERR("Invalid HWND: %p\n", hWnd);
        goto Quit;
    }

    pti = PsGetCurrentThreadWin32Thread();
    if (pWnd->head.pti->ppi == pti->ppi)
        ret = UserSetProp(pWnd, AtomImeLevel, UlongToHandle(dwLevel), TRUE);

Quit:
    UserLeave();
    return ret;
}

BOOL FASTCALL
UserSetImeInfoEx(
    _Inout_ PWINSTATION_OBJECT pWinSta,
    _Inout_ PIMEINFOEX pImeInfoEx)
{
    PKL pklHead, pkl;

    if (!pWinSta || !gspklBaseLayout)
    {
        ERR("pWinSta:%p, gspklBaseLayout:%p\n", pWinSta, gspklBaseLayout);
        return FALSE;
    }

    pkl = pklHead = gspklBaseLayout;

    do
    {
        if (pkl->hkl != pImeInfoEx->hkl)
        {
            pkl = pkl->pklNext;
            continue;
        }

        if (!pkl->piiex)
        {
            ERR("!pkl->piiex at %p\n", pkl->hkl);
            return FALSE;
        }

        if (!pkl->piiex->fLoadFlag)
            *pkl->piiex = *pImeInfoEx;

        return TRUE;
    } while (pkl != pklHead);

    ERR("HKL not found: %p\n", pImeInfoEx->hkl);
    return FALSE;
}

BOOL
NTAPI
NtUserSetImeInfoEx(_In_ const IMEINFOEX *pImeInfoEx)
{
    BOOL ret = FALSE;
    IMEINFOEX ImeInfoEx;
    PWINSTATION_OBJECT pWinSta;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        goto Quit;
    }

    {
        NTSTATUS __SehStatus = SafeProbeForRead(pImeInfoEx, sizeof(*pImeInfoEx), 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            ERR("Exception in NtUserSetImeInfoEx: pImeInfoEx=%p\n", pImeInfoEx);
            goto Quit;
        }
    }
    ImeInfoEx = *pImeInfoEx;

    pWinSta = IntGetProcessWindowStation(NULL);
    ret = UserSetImeInfoEx(pWinSta, &ImeInfoEx);

Quit:
    UserLeave();
    return ret;
}

// Choose the preferred owner of the IME window.
VOID FASTCALL
IntImeSetFutureOwner(PWND pImeWnd, PWND pwndOwner)
{
    PWND pwndNode, pwndNextOwner, pwndParent, pwndSibling;
    PTHREADINFO pti = pImeWnd->head.pti;

    if (!pwndOwner || (pwndOwner->style & WS_CHILD)) // invalid owner
        return;

    // Get the top-level owner of the same thread
    for (pwndNode = pwndOwner; ; pwndNode = pwndNextOwner)
    {
        pwndNextOwner = pwndNode->spwndOwner;
        if (!pwndNextOwner || pwndNextOwner->head.pti != pti)
            break;
    }

    // Don't choose the IME-like windows and the bottom-most windows unless necessary.
    if (IS_WND_IMELIKE(pwndNode) ||
        ((pwndNode->state2 & WNDS2_BOTTOMMOST) && !(pwndOwner->state2 & WNDS2_BOTTOMMOST)))
    {
        pwndNode = pwndOwner;
    }

    pwndParent = pwndNode->spwndParent;
    if (!pwndParent || pwndOwner != pwndNode)
    {
        WndSetOwner(pImeWnd, pwndNode);
        return;
    }

    for (pwndSibling = pwndParent->spwndChild; pwndSibling; pwndSibling = pwndSibling->spwndNext)
    {
        if (pwndNode->head.pti != pwndSibling->head.pti)
            continue;

        if (IS_WND_MENU(pwndSibling) || IS_WND_IMELIKE(pwndSibling))
            continue;

        if (pwndSibling->state2 & WNDS2_INDESTROY)
            continue;

        if (pwndNode == pwndSibling || (pwndSibling->style & WS_CHILD))
            continue;

        if (pwndSibling->spwndOwner == NULL ||
            pwndSibling->head.pti != pwndSibling->spwndOwner->head.pti)
        {
            pwndNode = pwndSibling;
            break;
        }
    }

    WndSetOwner(pImeWnd, pwndNode);
}

// Get the last non-IME-like top-most window on the desktop.
static PWND FASTCALL
IntGetLastTopMostWindowNoIME(_In_ PWND pImeWnd)
{
    PWND pwndNode, pwndOwner, pwndLastTopMost = NULL;
    BOOL bFound;

    pwndNode = UserGetDesktopWindow();
    if (!pwndNode || pwndNode->spwndChild == NULL)
        return NULL;

    for (pwndNode = pwndNode->spwndChild;
         pwndNode && (pwndNode->ExStyle & WS_EX_TOPMOST);
         pwndNode = pwndNode->spwndNext)
    {
        bFound = FALSE;

        if (IS_WND_IMELIKE(pwndNode)) // An IME-like window
        {
            // Search the IME window from owners
            for (pwndOwner = pwndNode; pwndOwner; pwndOwner = pwndOwner->spwndOwner)
            {
                if (pImeWnd == pwndOwner)
                {
                    bFound = TRUE;
                    break;
                }
            }
        }

        if (!bFound)
            pwndLastTopMost = pwndNode;
    }

    return pwndLastTopMost;
}

// Adjust the ordering of the windows around the IME window.
static VOID FASTCALL
IntImeSetTopMost(_In_ PWND pImeWnd, _In_ BOOL bTopMost, _In_ PWND pwndInsertBefore)
{
    PWND pwndParent, pwndChild, pwndNode, pwndNext, pwndInsertAfter = NULL;
    PWND pwndInsertAfterSave;

    pwndParent = pImeWnd->spwndParent;
    if (!pwndParent)
        return;

    pwndChild = pwndParent->spwndChild;

    if (!bTopMost)
    {
        // Calculate pwndInsertAfter
        pwndInsertAfter = IntGetLastTopMostWindowNoIME(pImeWnd);
        if (pwndInsertBefore)
        {
            for (pwndNode = pwndInsertAfter; pwndNode; pwndNode = pwndNode->spwndNext)
            {
                if (pwndNode->spwndNext == pwndInsertBefore)
                    break;

                if (pwndNode == pImeWnd)
                    return;
            }

            if (!pwndNode)
                return;

            pwndInsertAfter = pwndNode;
        }

        // Adjust pwndInsertAfter if the owner is bottom-most
        if (pImeWnd->spwndOwner->state2 & WNDS2_BOTTOMMOST)
        {
            for (pwndNode = pwndInsertAfter; pwndNode; pwndNode = pwndNode->spwndNext)
            {
                if (pwndNode == pImeWnd->spwndOwner)
                    break;

                if (!IS_WND_IMELIKE(pwndNode))
                    pwndInsertAfter = pwndNode;
            }
        }
    }

    pwndInsertAfterSave = pwndInsertAfter;

    while (pwndChild)
    {
        pwndNext = pwndChild->spwndNext;

        // If pwndChild is a good IME-like window, ...
        if (IS_WND_IMELIKE(pwndChild) && pwndChild != pwndInsertAfter &&
            pwndChild->head.pti == pImeWnd->head.pti)
        {
            // Find pImeWnd from the owners
            for (pwndNode = pwndChild; pwndNode; pwndNode = pwndNode->spwndOwner)
            {
                if (pwndNode != pImeWnd)
                    continue;

                // Adjust the ordering and the linking
                IntUnlinkWindow(pwndChild);

                if (bTopMost)
                    pwndChild->ExStyle |= WS_EX_TOPMOST;
                else
                    pwndChild->ExStyle &= ~WS_EX_TOPMOST;

                if (!pwndInsertAfter)
                    IntLinkHwnd(pwndChild, HWND_TOP);
                else
                    IntLinkHwnd(pwndChild, UserHMGetHandle(pwndInsertAfter));

                // Update the preferred position
                pwndInsertAfter = pwndChild;
                break;
            }
        }

        // Get the next child, with ignoring pwndInsertAfterSave
        pwndChild = pwndNext;
        if (pwndChild && pwndChild == pwndInsertAfterSave && pwndInsertAfter)
            pwndChild = pwndInsertAfter->spwndNext;
    }
}

// Make the IME window top-most if necessary.
static VOID FASTCALL
IntImeCheckTopmost(_In_ PWND pImeWnd)
{
    BOOL bTopMost;
    PWND pwndOwner = pImeWnd->spwndOwner, pwndInsertBefore = NULL;

    if (!pwndOwner)
        return;

    if (pImeWnd->head.pti != gptiForeground)
        pwndInsertBefore = pwndOwner;

    bTopMost = !!(pwndOwner->ExStyle & WS_EX_TOPMOST);
    IntImeSetTopMost(pImeWnd, bTopMost, pwndInsertBefore);
}

BOOL NTAPI
NtUserSetImeOwnerWindow(
    _In_ HWND hImeWnd,
    _In_opt_ HWND hwndFocus)
{
    BOOL ret = FALSE;
    PWND pImeWnd, pwndFocus, pwndTopLevel, pwndNode, pwndActive;
    PTHREADINFO ptiIme;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        goto Quit;
    }

    pImeWnd = ValidateHwndNoErr(hImeWnd);
    if (!pImeWnd || pImeWnd->fnid != FNID_IME)
    {
        ERR("Not IME window: %p\n", hImeWnd);
        goto Quit;
    }

    pwndFocus = ValidateHwndNoErr(hwndFocus);
    if (pwndFocus)
    {
        if (IS_WND_IMELIKE(pwndFocus))
            goto Quit;

        pwndTopLevel = IntGetTopLevelWindow(pwndFocus);

        for (pwndNode = pwndTopLevel; pwndNode; pwndNode = pwndNode->spwndOwner)
        {
            if (pwndNode->pcls->atomClassName == gpsi->atomSysClass[ICLS_IME])
            {
                pwndTopLevel = NULL;
                break;
            }
        }

        WndSetOwner(pImeWnd, pwndTopLevel);
        IntImeCheckTopmost(pImeWnd);
    }
    else
    {
        ptiIme = pImeWnd->head.pti;
        pwndActive = ptiIme->MessageQueue->spwndActive;

        if (!pwndActive || pwndActive != pImeWnd->spwndOwner)
        {
            if (pwndActive && ptiIme == pwndActive->head.pti && !IS_WND_IMELIKE(pwndActive))
            {
                WndSetOwner(pImeWnd, pwndActive);
            }
            else
            {
                IntImeSetFutureOwner(pImeWnd, pImeWnd->spwndOwner);
            }

            IntImeCheckTopmost(pImeWnd);
        }
    }

    ret = TRUE;

Quit:
    UserLeave();
    return ret;
}

PVOID
AllocInputContextObject(
    _In_ PDESKTOP pDesk,
    _In_ PTHREADINFO pti,
    _In_ SIZE_T Size,
    _Out_ PVOID* HandleOwner)
{
    PTHRDESKHEAD ObjHead;

    ASSERT(Size > sizeof(*ObjHead));
    ASSERT(pti != NULL);

    if (!pDesk)
        pDesk = pti->rpdesk;

    ObjHead = DesktopHeapAlloc(pDesk, Size);
    if (!ObjHead)
        return NULL;

    RtlZeroMemory(ObjHead, Size);

    ObjHead->pSelf = ObjHead;
    ObjHead->rpdesk = pDesk;
    ObjHead->pti = pti;
    IntReferenceThreadInfo(pti);
    *HandleOwner = pti;
    pti->ppi->UserHandleCount++;

    return ObjHead;
}

VOID
UserFreeInputContext(_In_opt_ PVOID Object)
{
    PTHRDESKHEAD ObjHead = Object;
    PDESKTOP pDesk = ObjHead->rpdesk;
    PIMC pNode, pIMC = Object;
    PTHREADINFO pti;

    if (!pIMC)
        return;

    // Remove pIMC from the list except spDefaultImc
    pti = pIMC->head.pti;
    for (pNode = pti->spDefaultImc; pNode; pNode = pNode->pImcNext)
    {
        if (pNode->pImcNext == pIMC)
        {
            pNode->pImcNext = pIMC->pImcNext;
            break;
        }
    }

    DesktopHeapFree(pDesk, Object);

    pti->ppi->UserHandleCount--;
    IntDereferenceThreadInfo(pti);
}

BOOLEAN
UserDestroyInputContext(_In_opt_ PVOID Object)
{
    PIMC pIMC = Object;
    if (!pIMC)
        return TRUE;

    UserMarkObjectDestroy(pIMC);
    UserDeleteObject(UserHMGetHandle(pIMC), TYPE_INPUTCONTEXT);
    return TRUE;
}

static BOOL
IntDestroyInputContext(_In_ PIMC pIMC)
{
    HIMC hIMC = UserHMGetHandle(pIMC);
    PTHREADINFO pti = pIMC->head.pti;
    PWND pwndChild;
    PWINDOWLIST pwl;
    HWND *phwnd;
    PWND pWnd;

    if (pti != gptiCurrent)
    {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return FALSE;
    }

    if (pIMC == pti->spDefaultImc)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    pwndChild = pti->rpdesk->pDeskInfo->spwnd->spwndChild;
    pwl = IntBuildHwndList(pwndChild, IACE_LIST | IACE_CHILDREN, pti);
    if (pwl)
    {
        for (phwnd = pwl->ahwnd; *phwnd != HWND_TERMINATOR; ++phwnd)
        {
            pWnd = UserGetObjectNoErr(gHandleTable, *phwnd, TYPE_WINDOW);
            if (pWnd && pWnd->hImc == hIMC)
                IntAssociateInputContext(pWnd, pti->spDefaultImc);
        }

        IntFreeHwndList(pwl);
    }

    UserDeleteObject(hIMC, TYPE_INPUTCONTEXT);
    return TRUE;
}

BOOL NTAPI
NtUserDestroyInputContext(_In_ HIMC hIMC)
{
    BOOL ret = FALSE;
    PIMC pIMC;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        goto Quit;
    }

    pIMC = UserGetObjectNoErr(gHandleTable, hIMC, TYPE_INPUTCONTEXT);
    if (pIMC)
        ret = IntDestroyInputContext(pIMC);

Quit:
    UserLeave();
    return ret;
}

PIMC FASTCALL
UserCreateInputContext(_In_ ULONG_PTR dwClientImcData)
{
    PIMC pIMC;
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
    PDESKTOP pdesk = pti->rpdesk;

    if (!IS_IMM_MODE() || (pti->TIF_flags & TIF_DISABLEIME)) // Disabled?
    {
        ERR("IME is disabled\n");
        return NULL;
    }

    if (!pdesk) // No desktop?
        return NULL;

    // pti->spDefaultImc should be already set if non-first time.
    if (dwClientImcData && !pti->spDefaultImc)
        return NULL;

    // Create an input context user object.
    pIMC = UserCreateObject(gHandleTable, pdesk, pti, NULL, TYPE_INPUTCONTEXT, sizeof(IMC));
    if (!pIMC)
        return NULL;

    // Release the extra reference (UserCreateObject added 2 references).
    UserDereferenceObject(pIMC);
    ASSERT(pIMC->head.cLockObj == 1);

    if (dwClientImcData) // Non-first time.
    {
        // Insert pIMC to the second position (non-default) of the list.
        pIMC->pImcNext = pti->spDefaultImc->pImcNext;
        pti->spDefaultImc->pImcNext = pIMC;
    }
    else // First time. It's the default IMC.
    {
        // Add the first one (default) to the list.
        UserAssignmentLock((PVOID*)&pti->spDefaultImc, pIMC);
        pIMC->pImcNext = NULL;
        ASSERT(pIMC->head.cLockObj == 2); // UserAssignmentUnlock'ed at ExitThreadCallback
    }

    pIMC->dwClientImcData = dwClientImcData; // Set it.
    return pIMC;
}

HIMC
NTAPI
NtUserCreateInputContext(_In_ ULONG_PTR dwClientImcData)
{
    PIMC pIMC;
    HIMC ret = NULL;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        goto Quit;
    }

    if (!dwClientImcData)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        goto Quit;
    }

    pIMC = UserCreateInputContext(dwClientImcData);
    if (pIMC)
        ret = UserHMGetHandle(pIMC);

Quit:
    UserLeave();
    return ret;
}

static DWORD FASTCALL
IntAssociateInputContextEx(_In_ PWND pWnd, _In_ PIMC pIMC, _In_ DWORD dwFlags)
{
    DWORD ret = 0;
    PWINDOWLIST pwl;
    BOOL bIgnoreNullImc = (dwFlags & IACE_IGNORENOCONTEXT);
    PTHREADINFO pti = pWnd->head.pti;
    PWND pwndTarget, pwndFocus = pti->MessageQueue->spwndFocus;
    HWND *phwnd;
    HIMC hIMC;

    if (dwFlags & IACE_DEFAULT)
    {
        pIMC = pti->spDefaultImc;
    }
    else
    {
        if (pIMC && pti != pIMC->head.pti)
            return 2;
    }

    if (pWnd->head.pti->ppi != GetW32ThreadInfo()->ppi ||
        (pIMC && pIMC->head.rpdesk != pWnd->head.rpdesk))
    {
        return 2;
    }

    if ((dwFlags & IACE_CHILDREN) && pWnd->spwndChild)
    {
        pwl = IntBuildHwndList(pWnd->spwndChild, IACE_CHILDREN | IACE_LIST, pti);
        if (pwl)
        {
            for (phwnd = pwl->ahwnd; *phwnd != HWND_TERMINATOR; ++phwnd)
            {
                pwndTarget = ValidateHwndNoErr(*phwnd);
                if (!pwndTarget)
                    continue;

                hIMC = (pIMC ? UserHMGetHandle(pIMC) : NULL);
                if (pwndTarget->hImc == hIMC || (bIgnoreNullImc && !pwndTarget->hImc))
                    continue;

                IntAssociateInputContext(pwndTarget, pIMC);
                if (pwndTarget == pwndFocus)
                    ret = 1;
            }

            IntFreeHwndList(pwl);
        }
    }

    if (!bIgnoreNullImc || pWnd->hImc)
    {
        hIMC = (pIMC ? UserHMGetHandle(pIMC) : NULL);
        if (pWnd->hImc != hIMC)
        {
            IntAssociateInputContext(pWnd, pIMC);
            if (pWnd == pwndFocus)
                ret = 1;
        }
    }

    return ret;
}

DWORD
NTAPI
NtUserAssociateInputContext(
    _In_ HWND hWnd,
    _In_opt_ HIMC hIMC,
    _In_ DWORD dwFlags)
{
    DWORD ret = 2;
    PWND pWnd;
    PIMC pIMC;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        goto Quit;
    }

    pWnd = ValidateHwndNoErr(hWnd);
    if (!pWnd)
    {
        ERR("Invalid HWND: %p\n", hWnd);
        goto Quit;
    }

    pIMC = (hIMC ? UserGetObjectNoErr(gHandleTable, hIMC, TYPE_INPUTCONTEXT) : NULL);
    ret = IntAssociateInputContextEx(pWnd, pIMC, dwFlags);

Quit:
    UserLeave();
    return ret;
}

static BOOL FASTCALL
UserUpdateInputContext(_In_ PIMC pIMC, _In_ DWORD dwType, _In_ DWORD_PTR dwValue)
{
    PTHREADINFO pti = GetW32ThreadInfo();
    PTHREADINFO ptiIMC = pIMC->head.pti;

    if (pti->ppi != ptiIMC->ppi) // Different process?
        return FALSE;

    switch (dwType)
    {
        case UIC_CLIENTIMCDATA:
            if (pIMC->dwClientImcData)
                return FALSE; // Already set

            pIMC->dwClientImcData = dwValue;
            break;

        case UIC_IMEWINDOW:
            if (!ValidateHwndNoErr((HWND)dwValue))
                return FALSE; // Invalid HWND

            pIMC->hImeWnd = (HWND)dwValue;
            break;

        default:
            ERR("Unhandled dwType: %lu\n", dwType);
            return FALSE;
    }

    return TRUE;
}

BOOL
NTAPI
NtUserUpdateInputContext(
    HIMC hIMC,
    DWORD dwType,
    DWORD_PTR dwValue)
{
    PIMC pIMC;
    BOOL ret = FALSE;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        goto Quit;
    }

    pIMC = UserGetObject(gHandleTable, hIMC, TYPE_INPUTCONTEXT);
    if (!pIMC)
    {
        ERR("Invalid HIMC %p\n", hIMC);
        goto Quit;
    }

    ret = UserUpdateInputContext(pIMC, dwType, dwValue);

Quit:
    UserLeave();
    return ret;
}

DWORD_PTR
NTAPI
NtUserQueryInputContext(HIMC hIMC, DWORD dwType)
{
    PIMC pIMC;
    PTHREADINFO ptiIMC;
    DWORD_PTR ret = 0;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
        goto Quit;

    pIMC = UserGetObject(gHandleTable, hIMC, TYPE_INPUTCONTEXT);
    if (!pIMC)
    {
        ERR("Invalid HIMC %p\n", hIMC);
        goto Quit;
    }

    ptiIMC = pIMC->head.pti;

    switch (dwType)
    {
        case QIC_INPUTPROCESSID:
            ret = (DWORD_PTR)PsGetThreadProcessId(ptiIMC->pEThread);
            break;

        case QIC_INPUTTHREADID:
            ret = (DWORD_PTR)PsGetThreadId(ptiIMC->pEThread);
            break;

        case QIC_DEFAULTWINDOWIME:
            if (ptiIMC->spwndDefaultIme)
                ret = (DWORD_PTR)UserHMGetHandle(ptiIMC->spwndDefaultIme);
            break;

        case QIC_DEFAULTIMC:
            if (ptiIMC->spDefaultImc)
                ret = (DWORD_PTR)UserHMGetHandle(ptiIMC->spDefaultImc);
            break;

        default:
        {
            FIXME("dwType: %ld\n", dwType);
            break;
        }
    }

Quit:
    UserLeave();
    return ret;
}