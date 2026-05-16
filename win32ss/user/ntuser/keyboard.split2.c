/* keyboard.split2.c — #include fragment of keyboard.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original keyboard.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL NTAPI
UserSendKeyboardInput(KEYBDINPUT *pKbdInput, BOOL bInjected)
{
    WORD wScanCode, wVk;
    PKL pKl = NULL;
    PKBDTABLES pKbdTbl;
    PUSER_MESSAGE_QUEUE pFocusQueue;
    DWORD dwTime;
    BOOL bExt = (pKbdInput->dwFlags & KEYEVENTF_EXTENDEDKEY) ? TRUE : FALSE;

    gppiInputProvider = ((PTHREADINFO)PsGetCurrentThreadWin32Thread())->ppi;

    /* Find the target thread whose locale is in effect */
    pFocusQueue = IntGetFocusMessageQueue();

    if (pFocusQueue && pFocusQueue->ptiKeyboard)
    {
        pKl = pFocusQueue->ptiKeyboard->KeyboardLayout;
    }

    if (!pKl)
        pKl = W32kGetDefaultKeyLayout();
    if (!pKl)
    {
        ERR("No keyboard layout!\n");
        return FALSE;
    }

    pKbdTbl = pKl->spkf->pKbdTbl;

    /* Note: wScan field is always used */
    wScanCode = pKbdInput->wScan;

    if (pKbdInput->dwFlags & KEYEVENTF_UNICODE)
    {
        /* Generate WM_KEYDOWN msg with wParam == VK_PACKET and
           high order word of lParam == pKbdInput->wScan */
        wVk = VK_PACKET;
    }
    else
    {
        wScanCode &= 0x7F;
        if (pKbdInput->dwFlags & KEYEVENTF_SCANCODE)
        {
            /* Don't ignore invalid scan codes */
            wVk = IntVscToVk(wScanCode | (bExt ? 0xE000 : 0), pKbdTbl);
            if (!wVk) /* use 0xFF if vsc is invalid */
                wVk = 0xFF;
        }
        else
        {
            wVk = pKbdInput->wVk;
        }

        /* Remove all virtual key flags (KBDEXT, KBDMULTIVK, KBDSPECIAL, KBDNUMPAD) */
        wVk &= 0xFF;
    }

    /* If time is given, use it */
    if (pKbdInput->time)
        dwTime = pKbdInput->time;
    else
    {
        dwTime = EngGetTickCount32();
    }

    if (wVk == VK_RMENU && (pKbdTbl->fLocaleFlags & KLLF_ALTGR))
    {
        /* For AltGr keyboards RALT generates CTRL events */
        ProcessKeyEvent(VK_LCONTROL, 0, pKbdInput->dwFlags & KEYEVENTF_KEYUP, bInjected, dwTime, 0);
    }

    /* Finally process this key */
    return ProcessKeyEvent(wVk, wScanCode, pKbdInput->dwFlags, bInjected, dwTime, pKbdInput->dwExtraInfo);
}

/*
 * UserProcessKeyboardInput
 *
 * Process raw keyboard input data
 */
VOID NTAPI
UserProcessKeyboardInput(
    PKEYBOARD_INPUT_DATA pKbdInputData)
{
    WORD wScanCode, wVk;
    PKL pKl = NULL;
    PKBDTABLES pKbdTbl;
    PUSER_MESSAGE_QUEUE pFocusQueue;

    /* Calculate scan code with prefix */
    wScanCode = pKbdInputData->MakeCode & 0x7F;
    if (pKbdInputData->Flags & KEY_E0)
        wScanCode |= 0xE000;
    if (pKbdInputData->Flags & KEY_E1)
        wScanCode |= 0xE100;

    /* Find the target thread whose locale is in effect */
    pFocusQueue = IntGetFocusMessageQueue();

    if (pFocusQueue && pFocusQueue->ptiKeyboard)
    {
        pKl = pFocusQueue->ptiKeyboard->KeyboardLayout;
    }

    if (!pKl)
        pKl = W32kGetDefaultKeyLayout();
    if (!pKl)
    {
        ERR("UserProcessKeyboardInput: NO keyboard layout!\n");
        return;
    }

    pKbdTbl = pKl->spkf->pKbdTbl;

    /* Convert scan code to virtual key.
       Note: We could call UserSendKeyboardInput using scan code,
             but it wouldn't interpret E1 key(s) properly */
    wVk = IntVscToVk(wScanCode, pKbdTbl);

    if (wVk)
    {
        KEYBDINPUT KbdInput;

        /* Support numlock */
        if ((wVk & KBDNUMPAD) && IS_KEY_LOCKED(gafAsyncKeyState, VK_NUMLOCK))
        {
            wVk = IntTranslateNumpadKey(wVk & 0xFF);
        }

        /* Send keyboard input */
        KbdInput.wVk = wVk & 0xFF;
        KbdInput.wScan = wScanCode & 0x7F;
        KbdInput.dwFlags = 0;
        if (pKbdInputData->Flags & KEY_BREAK)
            KbdInput.dwFlags |= KEYEVENTF_KEYUP;

        if (wVk & KBDEXT)
            KbdInput.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        //
        // Based on wine input:test_Input_blackbox this is okay. It seems the
        // bit did not get set and more research is needed. Now the right
        // shift works.
        //
        if (wVk == VK_RSHIFT)
            KbdInput.dwFlags |= KEYEVENTF_EXTENDEDKEY;

        KbdInput.time = 0;
        KbdInput.dwExtraInfo = pKbdInputData->ExtraInformation;
        UserSendKeyboardInput(&KbdInput, FALSE);

        /* E1 keys don't have break code */
        if (pKbdInputData->Flags & KEY_E1)
        {
            /* Send key up event */
            KbdInput.dwFlags |= KEYEVENTF_KEYUP;
            UserSendKeyboardInput(&KbdInput, FALSE);
        }
    }
}

/*
 * IntTranslateKbdMessage
 *
 * Addes WM_(SYS)CHAR messages to message queue if message
 * describes key which produce character.
 */
BOOL FASTCALL
IntTranslateKbdMessage(LPMSG lpMsg,
                       UINT flags)
{
    PTHREADINFO pti;
    INT cch = 0, i;
    WCHAR wch[3] = { 0 };
    MSG NewMsg = { 0 };
    PKBDTABLES pKbdTbl;
    BOOL bResult = FALSE;

    switch(lpMsg->message)
    {
       case WM_KEYDOWN:
       case WM_KEYUP:
       case WM_SYSKEYDOWN:
       case WM_SYSKEYUP:
          break;
       default:
          return FALSE;
    }

    pti = PsGetCurrentThreadWin32Thread();

    if (!pti->KeyboardLayout)
    {
        PKL pDefKL = W32kGetDefaultKeyLayout();
        UserAssignmentLock((PVOID*)&(pti->KeyboardLayout), pDefKL);
        if (pDefKL)
        {
            pti->pClientInfo->hKL = pDefKL->hkl;
            pKbdTbl = pDefKL->spkf->pKbdTbl;
        }
        else
        {
            pti->pClientInfo->hKL = NULL;
            pKbdTbl = NULL;
        }
    }
    else
       pKbdTbl = pti->KeyboardLayout->spkf->pKbdTbl;
    if (!pKbdTbl)
        return FALSE;

    if (lpMsg->message != WM_KEYDOWN && lpMsg->message != WM_SYSKEYDOWN)
        return FALSE;

    /* Init pt, hwnd and time msg fields */
    NewMsg.pt = gpsi->ptCursor;
    NewMsg.hwnd = lpMsg->hwnd;
    NewMsg.time = EngGetTickCount32();

    TRACE("Enter IntTranslateKbdMessage msg %s, vk %x\n",
        lpMsg->message == WM_SYSKEYDOWN ? "WM_SYSKEYDOWN" : "WM_KEYDOWN",
        lpMsg->wParam);

    if (lpMsg->wParam == VK_PACKET)
    {
        NewMsg.message = (lpMsg->message == WM_KEYDOWN) ? WM_CHAR : WM_SYSCHAR;
        NewMsg.wParam = HIWORD(lpMsg->lParam);
        NewMsg.lParam = LOWORD(lpMsg->lParam);
        MsqPostMessage(pti, &NewMsg, FALSE, QS_KEY, 0, 0);
        return TRUE;
    }

    cch = IntToUnicodeEx(lpMsg->wParam,
                         HIWORD(lpMsg->lParam) & 0xFF,
                         pti->MessageQueue->afKeyState,
                         wch,
                         sizeof(wch) / sizeof(wch[0]),
                         0,
                         pKbdTbl);

    if (cch)
    {
        if (cch > 0) /* Normal characters */
            NewMsg.message = (lpMsg->message == WM_KEYDOWN) ? WM_CHAR : WM_SYSCHAR;
        else /* Dead character */
        {
            cch = -cch;
            NewMsg.message =
                (lpMsg->message == WM_KEYDOWN) ? WM_DEADCHAR : WM_SYSDEADCHAR;
        }
        NewMsg.lParam = lpMsg->lParam;

        /* Send all characters */
        for (i = 0; i < cch; ++i)
        {
            TRACE("Msg: %x '%lc' (%04x) %08x\n", NewMsg.message, wch[i], wch[i], NewMsg.lParam);
            NewMsg.wParam = wch[i];
            MsqPostMessage(pti, &NewMsg, FALSE, QS_KEY, 0, 0);
        }
        bResult = TRUE;
    }

    TRACE("Leave IntTranslateKbdMessage ret %d, cch %d, msg %x, wch %x\n",
        bResult, cch, NewMsg.message, NewMsg.wParam);
    return bResult;
}

/*
 * Map a virtual key code, or virtual scan code, to a scan code, key code,
 * or unshifted unicode character.
 *
 * Code: See Below
 * Type:
 * 0 -- Code is a virtual key code that is converted into a virtual scan code
 *      that does not distinguish between left and right shift keys.
 * 1 -- Code is a virtual scan code that is converted into a virtual key code
 *      that does not distinguish between left and right shift keys.
 * 2 -- Code is a virtual key code that is converted into an unshifted unicode
 *      character.
 * 3 -- Code is a virtual scan code that is converted into a virtual key code
 *      that distinguishes left and right shift keys.
 * KeyLayout: Keyboard layout handle
 *
 * @implemented
 */
static UINT
IntMapVirtualKeyEx(UINT uCode, UINT Type, PKBDTABLES pKbdTbl)
{
    UINT uRet = 0;

    switch (Type)
    {
        case MAPVK_VK_TO_VSC:
            uCode = IntFixVk(uCode, FALSE);
            uRet = IntVkToVsc(uCode, pKbdTbl);
            if (uRet > 0xFF) // Fail for scancodes with prefix (e0, e1)
                uRet = 0;
            break;

        case MAPVK_VSC_TO_VK:
            uRet = IntVscToVk(uCode, pKbdTbl) & 0xFF;
            uRet = IntSimplifyVk(uRet);
            break;

        case MAPVK_VK_TO_CHAR:
            uRet = (UINT)IntVkToChar(uCode, pKbdTbl);
        break;

        case MAPVK_VSC_TO_VK_EX:
            uRet = IntVscToVk(uCode, pKbdTbl) & 0xFF;
            break;

        case MAPVK_VK_TO_VSC_EX:
            uRet = IntVkToVsc(uCode, pKbdTbl);
            break;

        default:
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ERR("Wrong type value: %u\n", Type);
    }

    return uRet;
}

/*
 * NtUserMapVirtualKeyEx
 *
 * Map a virtual key code, or virtual scan code, to a scan code, key code,
 * or unshifted unicode character. See IntMapVirtualKeyEx.
 */
UINT
APIENTRY
NtUserMapVirtualKeyEx(UINT uCode, UINT uType, DWORD keyboardId, HKL dwhkl)
{
    PKBDTABLES pKbdTbl = NULL;
    UINT ret = 0;

    TRACE("Enter NtUserMapVirtualKeyEx\n");
    UserEnterShared();

    if (!dwhkl)
    {
        PTHREADINFO pti;

        pti = PsGetCurrentThreadWin32Thread();
        if (pti && pti->KeyboardLayout)
            pKbdTbl = pti->KeyboardLayout->spkf->pKbdTbl;
    }
    else
    {
        PKL pKl;

        pKl = UserHklToKbl(dwhkl);
        if (pKl)
            pKbdTbl = pKl->spkf->pKbdTbl;
    }

    if (pKbdTbl)
        ret = IntMapVirtualKeyEx(uCode, uType, pKbdTbl);

    UserLeave();
    TRACE("Leave NtUserMapVirtualKeyEx, ret=%u\n", ret);
    return ret;
}

/*
 * NtUserToUnicodeEx
 *
 * Translates virtual key to characters
 */
int
APIENTRY
NtUserToUnicodeEx(
    UINT wVirtKey,
    UINT wScanCode,
    PBYTE pKeyStateUnsafe,
    LPWSTR pwszBuffUnsafe,
    INT cchBuff,
    UINT wFlags,
    HKL dwhkl)
{
    PTHREADINFO pti;
    BYTE afKeyState[256 * 2 / 8] = {0};
    PWCHAR pwszBuff = NULL;
    INT i, iRet = 0;
    PKL pKl = NULL;
    NTSTATUS Status = STATUS_SUCCESS;

    TRACE("Enter NtUserSetKeyboardState\n");

    /* Return 0 if SC_KEY_UP bit is set */
    if (wScanCode & SC_KEY_UP || wVirtKey >= 0x100)
    {
        ERR("Invalid parameter\n");
        return 0;
    }

    Status = SafeProbeForRead(pKeyStateUnsafe, 256 * sizeof(BYTE), 1);
    if (!NT_SUCCESS(Status))
    {
        ERR("Cannot copy key state\n");
        SetLastNtError(Status);
        return 0;
    }
    else
    {
        /* Probe and copy key state to smaller bitmap */

        for (i = 0; i < 256; ++i)
        {
            if (pKeyStateUnsafe[i] & KS_DOWN_BIT)
                SET_KEY_DOWN(afKeyState, i, TRUE);
            if (pKeyStateUnsafe[i] & KS_LOCK_BIT)
                SET_KEY_LOCKED(afKeyState, i, TRUE);
        }
    }

    pwszBuff = ExAllocatePoolWithTag(NonPagedPool, sizeof(WCHAR) * cchBuff, TAG_STRING);
    if (!pwszBuff)
    {
        ERR("ExAllocatePoolWithTag(%u) failed\n", sizeof(WCHAR) * cchBuff);
        return 0;
    }
    RtlZeroMemory(pwszBuff, sizeof(WCHAR) * cchBuff);

    UserEnterExclusive(); // Note: We modify wchDead static variable

    if (dwhkl)
        pKl = UserHklToKbl(dwhkl);

    if (!pKl)
    {
        pti = PsGetCurrentThreadWin32Thread();
        pKl = pti->KeyboardLayout;
    }

    if (pKl)
    {
        iRet = IntToUnicodeEx(wVirtKey,
                            wScanCode,
                            afKeyState,
                            pwszBuff,
                            cchBuff,
                            wFlags,
                            pKl->spkf->pKbdTbl);

        if (iRet)
        {
            Status = MmCopyToCaller(pwszBuffUnsafe, pwszBuff, cchBuff * sizeof(WCHAR));
        }
    }
    else
    {
        ERR("No keyboard layout ?!\n");
        Status = STATUS_INVALID_HANDLE;
    }

    ExFreePoolWithTag(pwszBuff, TAG_STRING);

    if (!NT_SUCCESS(Status))
    {
        iRet = 0;
        SetLastNtError(Status);
    }

    UserLeave();
    TRACE("Leave NtUserSetKeyboardState, ret=%i\n", iRet);
    return iRet;
}

/*
 * NtUserGetKeyNameText
 *
 * Gets key name from keyboard layout
 */
DWORD
APIENTRY
NtUserGetKeyNameText(LONG lParam, LPWSTR lpString, int cchSize)
{
    PTHREADINFO pti;
    DWORD i, dwRet = 0;
    SIZE_T cchKeyName;
    WORD wScanCode = (lParam >> 16) & 0xFF;
    BOOL bExtKey = (HIWORD(lParam) & KF_EXTENDED) ? TRUE : FALSE;
    PKBDTABLES pKbdTbl;
    VSC_LPWSTR *pKeyNames = NULL;
    CONST WCHAR *pKeyName = NULL;
    WCHAR KeyNameBuf[2];
    NTSTATUS Status;

    TRACE("Enter NtUserGetKeyNameText\n");

    UserEnterShared();

    /* Get current keyboard layout */
    pti = PsGetCurrentThreadWin32Thread();
    pKbdTbl = pti ? pti->KeyboardLayout->spkf->pKbdTbl : 0;

    if (!pKbdTbl || cchSize < 1)
    {
        ERR("Invalid parameter\n");
        goto cleanup;
    }

    /* "Do not care" flag */
    if(lParam & LP_DO_NOT_CARE_BIT)
    {
        /* Note: We could do vsc -> vk -> vsc conversion, instead of using
                 hardcoded scan codes, but it's not what Windows does */
        if (wScanCode == SCANCODE_RSHIFT && !bExtKey)
            wScanCode = SCANCODE_LSHIFT;
        else if (wScanCode == SCANCODE_CTRL || wScanCode == SCANCODE_ALT)
            bExtKey = FALSE;
    }

    if (bExtKey)
        pKeyNames = pKbdTbl->pKeyNamesExt;
    else
        pKeyNames = pKbdTbl->pKeyNames;

    for (i = 0; pKeyNames[i].pwsz; i++)
    {
        if (pKeyNames[i].vsc == wScanCode)
        {
            pKeyName = pKeyNames[i].pwsz;
            break;
        }
    }

    if (!pKeyName)
    {
        WORD wVk = IntVscToVk(wScanCode, pKbdTbl);

        if (wVk)
        {
            KeyNameBuf[0] = IntVkToChar(wVk, pKbdTbl);
            KeyNameBuf[1] = 0;
            if (KeyNameBuf[0])
                pKeyName = KeyNameBuf;
        }
    }

    if (pKeyName)
    {
        cchKeyName = wcslen(pKeyName);
        if (cchKeyName > (cchSize - 1UL))
            cchKeyName = cchSize - 1UL; // Don't count '\0'

        Status = SafeProbeForWrite(lpString, (cchKeyName + 1) * sizeof(WCHAR), 1);
        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
        }
        else
        {
            RtlCopyMemory(lpString, pKeyName, cchKeyName * sizeof(WCHAR));
            lpString[cchKeyName] = UNICODE_NULL;
            dwRet = cchKeyName;
        }
    }
    else
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
    }

cleanup:
    UserLeave();
    TRACE("Leave NtUserGetKeyNameText, ret=%lu\n", dwRet);
    return dwRet;
}


/*
 * UserGetKeyboardType
 *
 * Returns some keyboard specific information
 */
DWORD FASTCALL
UserGetKeyboardType(
    DWORD dwTypeFlag)
{
    switch (dwTypeFlag)
    {
        case 0:        /* Keyboard type */
            return (DWORD)gKeyboardInfo.KeyboardIdentifier.Type;
        case 1:        /* Keyboard Subtype */
            return (DWORD)gKeyboardInfo.KeyboardIdentifier.Subtype;
        case 2:        /* Number of F-keys */
            return (DWORD)gKeyboardInfo.NumberOfFunctionKeys;
        default:
            ERR("Unknown type!\n");
            return 0;    /* Note: we don't have to set last error here */
    }
}

/*
 * NtUserVkKeyScanEx
 *
 * Based on IntTranslateChar, instead of processing VirtualKey match,
 * look for wChar match.
 */
DWORD
APIENTRY
NtUserVkKeyScanEx(
    WCHAR wch,
    HKL dwhkl,
    BOOL bUsehKL)
{
    PKBDTABLES pKbdTbl;
    PVK_TO_WCHAR_TABLE pVkToWchTbl;
    PVK_TO_WCHARS10 pVkToWch;
    PKL pKl = NULL;
    DWORD i, dwModBits = 0, dwModNumber = 0, Ret = (DWORD)-1;

    TRACE("NtUserVkKeyScanEx() wch %u, KbdLayout 0x%p\n", wch, dwhkl);
    UserEnterShared();

    if (bUsehKL)
    {
        // Use given keyboard layout
        if (dwhkl)
            pKl = UserHklToKbl(dwhkl);
    }
    else
    {
        // Use thread keyboard layout
        pKl = ((PTHREADINFO)PsGetCurrentThreadWin32Thread())->KeyboardLayout;
    }

    if (!pKl)
        goto Exit;

    pKbdTbl = pKl->spkf->pKbdTbl;

    // Interate through all VkToWchar tables while pVkToWchars is not NULL
    for (i = 0; pKbdTbl->pVkToWcharTable[i].pVkToWchars; i++)
    {
        pVkToWchTbl = &pKbdTbl->pVkToWcharTable[i];
        pVkToWch = (PVK_TO_WCHARS10)(pVkToWchTbl->pVkToWchars);

        // Interate through all virtual keys
        while (pVkToWch->VirtualKey)
        {
            for (dwModNumber = 0; dwModNumber < pVkToWchTbl->nModifications; dwModNumber++)
            {
                if (pVkToWch->wch[dwModNumber] == wch)
                {
                    dwModBits = pKbdTbl->pCharModifiers->ModNumber[dwModNumber];
                    TRACE("i %lu wC %04x: dwModBits %08x dwModNumber %08x MaxModBits %08x\n",
                          i, wch, dwModBits, dwModNumber, pKbdTbl->pCharModifiers->wMaxModBits);
                    Ret = (dwModBits << 8) | (pVkToWch->VirtualKey & 0xFF);
                    goto Exit;
                }
            }
            pVkToWch = (PVK_TO_WCHARS10)(((BYTE *)pVkToWch) + pVkToWchTbl->cbSize);
        }
    }
Exit:
    UserLeave();
    return Ret;
}

/* EOF */
