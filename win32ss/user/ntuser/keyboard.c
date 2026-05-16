/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS kernel
 * PURPOSE:          Keyboard functions
 * FILE:             win32ss/user/ntuser/keyboard.c
 * PROGRAMERS:       Casper S. Hornstrup (chorns@users.sourceforge.net)
 *                   Rafal Harabien (rafalh@reactos.org)
 */

#include <win32k.h>
#include <safe_user_io.h>
DBG_DEFAULT_CHANNEL(UserKbd);

BYTE gafAsyncKeyState[256 * 2 / 8]; // 2 bits per key
static BYTE gafAsyncKeyStateRecentDown[256 / 8]; // 1 bit per key
static PKEYBOARD_INDICATOR_TRANSLATION gpKeyboardIndicatorTrans = NULL;
static KEYBOARD_INDICATOR_PARAMETERS gIndicators = {0, 0};
KEYBOARD_ATTRIBUTES gKeyboardInfo;
INT gLanguageToggleKeyState = 0;
DWORD gdwLanguageToggleKey = 1;
INT gLayoutToggleKeyState = 0;
DWORD gdwLayoutToggleKey = 2;

/* FUNCTIONS *****************************************************************/

/*
 * InitKeyboardImpl
 *
 * Initialization -- Right now, just zero the key state
 */
CODE_SEG("INIT")
NTSTATUS
NTAPI
InitKeyboardImpl(VOID)
{
    RtlZeroMemory(&gafAsyncKeyState, sizeof(gafAsyncKeyState));
    RtlZeroMemory(&gafAsyncKeyStateRecentDown, sizeof(gafAsyncKeyStateRecentDown));
    // Clear and set default information.
    RtlZeroMemory(&gKeyboardInfo, sizeof(gKeyboardInfo));
    gKeyboardInfo.KeyboardIdentifier.Type = 4; /* AT-101 */
    gKeyboardInfo.NumberOfFunctionKeys = 12; /* We're doing an 101 for now, so return 12 F-keys */
    return STATUS_SUCCESS;
}

/*
 * IntKeyboardGetIndicatorTrans
 *
 * Asks the keyboard driver to send a small table that shows which
 * lights should connect with which scancodes
 */
//static
NTSTATUS APIENTRY
IntKeyboardGetIndicatorTrans(HANDLE hKeyboardDevice,
                             PKEYBOARD_INDICATOR_TRANSLATION *ppIndicatorTrans)
{
    NTSTATUS Status;
    DWORD dwSize = 0;
    IO_STATUS_BLOCK Block;
    PKEYBOARD_INDICATOR_TRANSLATION pRet;

    dwSize = sizeof(KEYBOARD_INDICATOR_TRANSLATION);

    pRet = ExAllocatePoolWithTag(PagedPool,
                                 dwSize,
                                 USERTAG_KBDTABLE);

    while (pRet)
    {
        Status = ZwDeviceIoControlFile(hKeyboardDevice,
                                       NULL,
                                       NULL,
                                       NULL,
                                       &Block,
                                       IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION,
                                       NULL, 0,
                                       pRet, dwSize);

        if (Status != STATUS_BUFFER_TOO_SMALL)
            break;

        ExFreePoolWithTag(pRet, USERTAG_KBDTABLE);

        dwSize += sizeof(KEYBOARD_INDICATOR_TRANSLATION);

        pRet = ExAllocatePoolWithTag(PagedPool,
                                     dwSize,
                                     USERTAG_KBDTABLE);
    }

    if (!pRet)
        return STATUS_INSUFFICIENT_RESOURCES;

    if (!NT_SUCCESS(Status))
    {
        ExFreePoolWithTag(pRet, USERTAG_KBDTABLE);
        return Status;
    }

    *ppIndicatorTrans = pRet;
    return Status;
}

/*
 * IntKeyboardUpdateLeds
 *
 * Sends the keyboard commands to turn on/off the lights
 */
static
NTSTATUS APIENTRY
IntKeyboardUpdateLeds(HANDLE hKeyboardDevice,
                      WORD wVk,
                      WORD wScanCode)
{
    NTSTATUS Status;
    UINT i;
    USHORT LedFlag = 0;
    IO_STATUS_BLOCK Block;

    if (!gpKeyboardIndicatorTrans)
        return STATUS_NOT_SUPPORTED;

    switch (wVk)
    {
        case VK_CAPITAL: LedFlag = KEYBOARD_CAPS_LOCK_ON; break;
        case VK_NUMLOCK: LedFlag = KEYBOARD_NUM_LOCK_ON; break;
        case VK_SCROLL: LedFlag = KEYBOARD_SCROLL_LOCK_ON; break;
        default:
            for (i = 0; i < gpKeyboardIndicatorTrans->NumberOfIndicatorKeys; i++)
            {
                if (gpKeyboardIndicatorTrans->IndicatorList[i].MakeCode == wScanCode)
                {
                    LedFlag = gpKeyboardIndicatorTrans->IndicatorList[i].IndicatorFlags;
                    break;
                }
            }
    }

    if (LedFlag)
    {
        gIndicators.LedFlags ^= LedFlag;

        /* Update the lights on the hardware */
        Status = ZwDeviceIoControlFile(hKeyboardDevice,
                                       NULL,
                                       NULL,
                                       NULL,
                                       &Block,
                                       IOCTL_KEYBOARD_SET_INDICATORS,
                                       &gIndicators, sizeof(gIndicators),
                                       NULL, 0);

        return Status;
    }

    return STATUS_SUCCESS;
}

/*
 * UserInitKeyboard
 *
 * Initializes keyboard indicators translation and their state
 */
VOID NTAPI
UserInitKeyboard(HANDLE hKeyboardDevice)
{
    NTSTATUS Status;
    IO_STATUS_BLOCK Block;

    IntKeyboardGetIndicatorTrans(hKeyboardDevice, &gpKeyboardIndicatorTrans);

    Status = ZwDeviceIoControlFile(hKeyboardDevice,
                                   NULL,
                                   NULL,
                                   NULL,
                                   &Block,
                                   IOCTL_KEYBOARD_QUERY_INDICATORS,
                                   NULL, 0,
                                   &gIndicators,
                                   sizeof(gIndicators));

    if (!NT_SUCCESS(Status))
    {
        WARN("NtDeviceIoControlFile() failed, ignored\n");
        gIndicators.LedFlags = 0;
        gIndicators.UnitId = 0;
    }

    SET_KEY_LOCKED(gafAsyncKeyState, VK_CAPITAL,
                   gIndicators.LedFlags & KEYBOARD_CAPS_LOCK_ON);
    SET_KEY_LOCKED(gafAsyncKeyState, VK_NUMLOCK,
                   gIndicators.LedFlags & KEYBOARD_NUM_LOCK_ON);
    SET_KEY_LOCKED(gafAsyncKeyState, VK_SCROLL,
                   gIndicators.LedFlags & KEYBOARD_SCROLL_LOCK_ON);

    // FIXME: Need device driver to work! HID support more than one!!!!
    Status = ZwDeviceIoControlFile(hKeyboardDevice,
                                   NULL,
                                   NULL,
                                   NULL,
                                   &Block,
                                   IOCTL_KEYBOARD_QUERY_ATTRIBUTES,
                                   NULL, 0,
                                   &gKeyboardInfo, sizeof(gKeyboardInfo));

    if (!NT_SUCCESS(Status))
    {
        ERR("NtDeviceIoControlFile() failed, ignored\n");
    }
    TRACE("Keyboard type %u, subtype %u and number of func keys %u\n",
             gKeyboardInfo.KeyboardIdentifier.Type,
             gKeyboardInfo.KeyboardIdentifier.Subtype,
             gKeyboardInfo.NumberOfFunctionKeys);
}

/*
 * IntSimplifyVk
 *
 * Changes virtual keys which distinguish between left and right hand, to keys which don't distinguish
 */
static
WORD
IntSimplifyVk(WORD wVk)
{
    switch (wVk)
    {
        case VK_LSHIFT:
        case VK_RSHIFT:
            return VK_SHIFT;

        case VK_LCONTROL:
        case VK_RCONTROL:
            return VK_CONTROL;

        case VK_LMENU:
        case VK_RMENU:
            return VK_MENU;

        default:
            return wVk;
    }
}

/*
 * IntFixVk
 *
 * Changes virtual keys which don't not distinguish between left and right hand to proper keys
 */
static
WORD
IntFixVk(WORD wVk, BOOL bExt)
{
    switch (wVk)
    {
        case VK_SHIFT:
            return bExt ? VK_RSHIFT : VK_LSHIFT;

        case VK_CONTROL:
            return bExt ? VK_RCONTROL : VK_LCONTROL;

        case VK_MENU:
            return bExt ? VK_RMENU : VK_LMENU;

        default:
            return wVk;
    }
}

/*
 * IntTranslateNumpadKey
 *
 * Translates numpad keys when numlock is enabled
 */
static
WORD
IntTranslateNumpadKey(WORD wVk)
{
    switch (wVk)
    {
        case VK_INSERT: return VK_NUMPAD0;
        case VK_END: return VK_NUMPAD1;
        case VK_DOWN: return VK_NUMPAD2;
        case VK_NEXT: return VK_NUMPAD3;
        case VK_LEFT: return VK_NUMPAD4;
        case VK_CLEAR: return VK_NUMPAD5;
        case VK_RIGHT: return VK_NUMPAD6;
        case VK_HOME: return VK_NUMPAD7;
        case VK_UP: return VK_NUMPAD8;
        case VK_PRIOR: return VK_NUMPAD9;
        case VK_DELETE: return VK_DECIMAL;
        default: return wVk;
    }
}

/*
 * IntGetModBits
 *
 * Gets layout specific modification bits, for example KBDSHIFT, KBDCTRL, KBDALT
 */
static
DWORD
IntGetModBits(PKBDTABLES pKbdTbl, PBYTE pKeyState)
{
    DWORD i, dwModBits = 0;

    /* DumpKeyState( KeyState ); */

    for (i = 0; pKbdTbl->pCharModifiers->pVkToBit[i].Vk; i++)
        if (IS_KEY_DOWN(pKeyState, pKbdTbl->pCharModifiers->pVkToBit[i].Vk))
            dwModBits |= pKbdTbl->pCharModifiers->pVkToBit[i].ModBits;

    TRACE("Current Mod Bits: %lx\n", dwModBits);

    return dwModBits;
}

/*
 * IntTranslateChar
 *
 * Translates virtual key to character
 */
static
BOOL
IntTranslateChar(WORD wVirtKey,
                 PBYTE pKeyState,
                 PBOOL pbDead,
                 PBOOL pbLigature,
                 PWCHAR pwcTranslatedChar,
                 PKBDTABLES pKbdTbl)
{
    PVK_TO_WCHAR_TABLE pVkToVchTbl;
    PVK_TO_WCHARS10 pVkToVch;
    DWORD i, dwModBits, dwVkModBits, dwModNumber = 0;
    WCHAR wch;
    BOOL bAltGr;
    WORD wCaplokAttr;

    dwModBits = pKeyState ? IntGetModBits(pKbdTbl, pKeyState) : 0;
    bAltGr = pKeyState && (pKbdTbl->fLocaleFlags & KLLF_ALTGR) && IS_KEY_DOWN(pKeyState, VK_RMENU);
    wCaplokAttr = bAltGr ? CAPLOKALTGR : CAPLOK;

    TRACE("TryToTranslate: %04x %x\n", wVirtKey, dwModBits);

    /* If ALT without CTRL has ben used, remove ALT flag */
    if ((dwModBits & (KBDALT|KBDCTRL)) == KBDALT)
        dwModBits &= ~KBDALT;

    if (dwModBits > pKbdTbl->pCharModifiers->wMaxModBits)
    {
        TRACE("dwModBits %x > wMaxModBits %x\n", dwModBits, pKbdTbl->pCharModifiers->wMaxModBits);
        return FALSE;
    }

    for (i = 0; pKbdTbl->pVkToWcharTable[i].pVkToWchars; i++)
    {
        pVkToVchTbl = &pKbdTbl->pVkToWcharTable[i];
        pVkToVch = (PVK_TO_WCHARS10)(pVkToVchTbl->pVkToWchars);
        while (pVkToVch->VirtualKey)
        {
            if (wVirtKey == (pVkToVch->VirtualKey & 0xFF))
            {
                dwVkModBits = dwModBits;

                /* If CapsLock is enabled for this key and locked, add SHIFT bit */
                if ((pVkToVch->Attributes & wCaplokAttr) &&
                    pKeyState &&
                    IS_KEY_LOCKED(pKeyState, VK_CAPITAL))
                {
                    /* Note: we use special value here instead of getting VK_SHIFT mod bit - it's verified */
                    dwVkModBits ^= KBDSHIFT;
                }

                if (dwVkModBits > pKbdTbl->pCharModifiers->wMaxModBits)
                    break;

                /* Get modification number */
                dwModNumber = pKbdTbl->pCharModifiers->ModNumber[dwVkModBits];
                if (dwModNumber >= pVkToVchTbl->nModifications)
                {
                    TRACE("dwModNumber %u >= nModifications %u\n", dwModNumber, pVkToVchTbl->nModifications);
                    break;
                }

                /* Read character */
                wch = pVkToVch->wch[dwModNumber];
                if (wch == WCH_NONE)
                    break;

                *pbDead = (wch == WCH_DEAD);
                *pbLigature = (wch == WCH_LGTR);
                *pwcTranslatedChar = wch;

                TRACE("%lu %04x: dwModNumber %08x Char %04x\n",
                      i, wVirtKey, dwModNumber, wch);

                if (*pbDead)
                {
                    /* After WCH_DEAD, real character is located */
                    pVkToVch = (PVK_TO_WCHARS10)(((BYTE *)pVkToVch) + pVkToVchTbl->cbSize);
                    if (pVkToVch->VirtualKey != 0xFF)
                    {
                        WARN("Found dead key with no trailer in the table.\n");
                        WARN("VK: %04x, ADDR: %p\n", wVirtKey, pVkToVch);
                        break;
                    }
                    *pwcTranslatedChar = pVkToVch->wch[dwModNumber];
                }
                return TRUE;
            }
            pVkToVch = (PVK_TO_WCHARS10)(((BYTE *)pVkToVch) + pVkToVchTbl->cbSize);
        }
    }

    /* If nothing has been found in layout, check if this is ASCII control character.
       Note: we could add it to layout table, but windows does not have it there */
    if (wVirtKey >= 'A' && wVirtKey <= 'Z' &&
        pKeyState && IS_KEY_DOWN(pKeyState, VK_CONTROL) &&
        !IS_KEY_DOWN(pKeyState, VK_MENU))
    {
        *pwcTranslatedChar = (wVirtKey - 'A') + 1; /* ASCII control character */
        *pbDead = FALSE;
        *pbLigature = FALSE;
        return TRUE;
    }

    return FALSE;
}

/*
 * IntToUnicodeEx
 *
 * Translates virtual key to characters
 */
static
int APIENTRY
IntToUnicodeEx(UINT wVirtKey,
               UINT wScanCode,
               PBYTE pKeyState,
               LPWSTR pwszBuff,
               int cchBuff,
               UINT wFlags,
               PKBDTABLES pKbdTbl)
{
    WCHAR wchTranslatedChar;
    BOOL bDead, bLigature;
    static WCHAR wchDead = 0;
    int iRet = 0;

    ASSERT(pKbdTbl);

    if (!IntTranslateChar(wVirtKey,
                          pKeyState,
                          &bDead,
                          &bLigature,
                          &wchTranslatedChar,
                          pKbdTbl))
    {
        return 0;
    }

    if (bLigature)
    {
        WARN("Not handling ligature (yet)\n" );
        return 0;
    }

    /* If we got dead char in previous call check dead keys in keyboard layout */
    if (wchDead)
    {
        UINT i;
        WCHAR wchFirst, wchSecond;
        TRACE("Previous dead char: %lc (%x)\n", wchDead, wchDead);

        if (pKbdTbl->pDeadKey)
        {
            for (i = 0; pKbdTbl->pDeadKey[i].dwBoth; i++)
            {
                wchFirst = pKbdTbl->pDeadKey[i].dwBoth >> 16;
                wchSecond = pKbdTbl->pDeadKey[i].dwBoth & 0xFFFF;
                if (wchFirst == wchDead && wchSecond == wchTranslatedChar)
                {
                    wchTranslatedChar = pKbdTbl->pDeadKey[i].wchComposed;
                    wchDead = 0;
                    bDead = FALSE;
                    break;
                }
            }
        }
        else
        {
#if defined(__GNUC__)
            if (wchDead == 0x8000)
            {
                ERR("GCC is inventing bits, ignoring fake dead key\n");
                wchDead = 0;
            }
#endif
        }

        TRACE("Final char: %lc (%x)\n", wchTranslatedChar, wchTranslatedChar);
    }

    /* Dead char has not been not found */
    if (wchDead)
    {
        /* Treat both characters normally */
        if (cchBuff > iRet)
            pwszBuff[iRet++] = wchDead;
        bDead = FALSE;
    }

    /* Add character to the buffer */
    if (cchBuff > iRet)
        pwszBuff[iRet++] = wchTranslatedChar;

    /* Save dead character */
    wchDead = bDead ? wchTranslatedChar : 0;

    return bDead ? -iRet : iRet;
}

/*
 * IntVkToVsc
 *
 * Translates virtual key to scan code
 */
static
WORD FASTCALL
IntVkToVsc(WORD wVk, PKBDTABLES pKbdTbl)
{
    unsigned i;

    ASSERT(pKbdTbl);

    /* Check standard keys first */
    for (i = 0; i < pKbdTbl->bMaxVSCtoVK; i++)
    {
        if ((pKbdTbl->pusVSCtoVK[i] & 0xFF) == wVk)
            return i;
    }

    /* Check extended keys now */
    for (i = 0; pKbdTbl->pVSCtoVK_E0[i].Vsc; i++)
    {
        if ((pKbdTbl->pVSCtoVK_E0[i].Vk & 0xFF) == wVk)
            return 0xE000 | pKbdTbl->pVSCtoVK_E0[i].Vsc;
    }

    for (i = 0; pKbdTbl->pVSCtoVK_E1[i].Vsc; i++)
    {
        if ((pKbdTbl->pVSCtoVK_E1[i].Vk & 0xFF) == wVk)
            return 0xE100 | pKbdTbl->pVSCtoVK_E1[i].Vsc;
    }

    /* Virtual key has not been found */
    return 0;
}

/*
 * IntVscToVk
 *
 * Translates prefixed scancode to virtual key
 */
static
WORD FASTCALL
IntVscToVk(WORD wScanCode, PKBDTABLES pKbdTbl)
{
    unsigned i;
    WORD wVk = 0;

    ASSERT(pKbdTbl);

    if ((wScanCode & 0xFF00) == 0xE000)
    {
        for (i = 0; pKbdTbl->pVSCtoVK_E0[i].Vsc; i++)
        {
            if (pKbdTbl->pVSCtoVK_E0[i].Vsc == (wScanCode & 0xFF))
            {
                wVk = pKbdTbl->pVSCtoVK_E0[i].Vk;
            }
        }
    }
    else if ((wScanCode & 0xFF00) == 0xE100)
    {
        for (i = 0; pKbdTbl->pVSCtoVK_E1[i].Vsc; i++)
        {
            if (pKbdTbl->pVSCtoVK_E1[i].Vsc == (wScanCode & 0xFF))
            {
                wVk = pKbdTbl->pVSCtoVK_E1[i].Vk;
            }
        }
    }
    else if (wScanCode < pKbdTbl->bMaxVSCtoVK)
    {
        wVk = pKbdTbl->pusVSCtoVK[wScanCode];
    }

    /* 0xFF nad 0x00 are invalid VKs */
    return wVk != 0xFF ? wVk : 0;
}

/*
 * IntVkToChar
 *
 * Translates virtual key to character, ignoring shift state
 */
static
WCHAR FASTCALL
IntVkToChar(WORD wVk, PKBDTABLES pKbdTbl)
{
    WCHAR wch;
    BOOL bDead, bLigature;

    ASSERT(pKbdTbl);

    if (IntTranslateChar(wVk,
                         NULL,
                         &bDead,
                         &bLigature,
                         &wch,
                         pKbdTbl))
    {
        return wch;
    }

    return 0;
}

/*
 * NtUserGetAsyncKeyState
 *
 * Gets key state from global bitmap
 */
SHORT
APIENTRY
NtUserGetAsyncKeyState(INT Key)
{
    WORD wRet = 0;

    TRACE("Enter NtUserGetAsyncKeyState\n");

    if (Key >= 0x100 || Key < 0)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        ERR("Invalid parameter Key\n");
        return 0;
    }

    UserEnterExclusive();

    if (IS_KEY_DOWN(gafAsyncKeyState, Key))
        wRet |= 0x8000; // If down, windows returns 0x8000.
    if (gafAsyncKeyStateRecentDown[Key / 8] & (1 << (Key % 8)))
        wRet |= 0x1;
    gafAsyncKeyStateRecentDown[Key / 8] &= ~(1 << (Key % 8));

    UserLeave();

    TRACE("Leave NtUserGetAsyncKeyState, ret=%u\n", wRet);
    return wRet;
}

/*
 * UpdateAsyncKeyState
 *
 * Updates gafAsyncKeyState array
 */
static
VOID NTAPI
UpdateAsyncKeyState(WORD wVk, BOOL bIsDown)
{
    if (bIsDown)
    {
        /* If it's first key down event, xor lock bit */
        if (!IS_KEY_DOWN(gafAsyncKeyState, wVk))
            SET_KEY_LOCKED(gafAsyncKeyState, wVk, !IS_KEY_LOCKED(gafAsyncKeyState, wVk));

        SET_KEY_DOWN(gafAsyncKeyState, wVk, TRUE);
        gafAsyncKeyStateRecentDown[wVk / 8] |= (1 << (wVk % 8));
    }
    else
        SET_KEY_DOWN(gafAsyncKeyState, wVk, FALSE);
}

/*
 * co_CallLowLevelKeyboardHook
 *
 * Calls WH_KEYBOARD_LL hook
 */
static LRESULT
co_CallLowLevelKeyboardHook(WORD wVk, WORD wScanCode, DWORD dwFlags, BOOL bInjected, DWORD dwTime, DWORD dwExtraInfo)
{
    KBDLLHOOKSTRUCT KbdHookData;
    UINT uMsg;

    KbdHookData.vkCode = wVk;
    KbdHookData.scanCode = wScanCode;
    KbdHookData.flags = 0;
    if (dwFlags & KEYEVENTF_EXTENDEDKEY)
        KbdHookData.flags |= LLKHF_EXTENDED;
    if (IS_KEY_DOWN(gafAsyncKeyState, VK_MENU))
        KbdHookData.flags |= LLKHF_ALTDOWN;
    if (dwFlags & KEYEVENTF_KEYUP)
        KbdHookData.flags |= LLKHF_UP;
    if (bInjected)
        KbdHookData.flags |= LLKHF_INJECTED;
    KbdHookData.time = dwTime;
    KbdHookData.dwExtraInfo = dwExtraInfo;

    /* Note: it doesnt support WM_SYSKEYUP */
    if (dwFlags & KEYEVENTF_KEYUP)
        uMsg = WM_KEYUP;
    else if (IS_KEY_DOWN(gafAsyncKeyState, VK_MENU) && !IS_KEY_DOWN(gafAsyncKeyState, VK_CONTROL))
        uMsg = WM_SYSKEYDOWN;
    else
        uMsg = WM_KEYDOWN;

    return co_HOOK_CallHooks(WH_KEYBOARD_LL, HC_ACTION, uMsg, (LPARAM)&KbdHookData);
}

/*
 * SnapWindow
 *
 * Saves snapshot of specified window or whole screen in the clipboard
 */
static VOID
SnapWindow(HWND hWnd)
{
    HBITMAP hbm = NULL, hbmOld;
    HDC hdc = NULL, hdcMem;
    SETCLIPBDATA scd;
    INT cx, cy;
    PWND pWnd = NULL;

    TRACE("SnapWindow(%p)\n", hWnd);

    /* If no windows is given, make snapshot of desktop window */
    if (!hWnd)
        hWnd = IntGetDesktopWindow();

    pWnd = UserGetWindowObject(hWnd);
    if (!pWnd)
    {
        ERR("Invalid window\n");
        goto cleanup;
    }

    hdc = UserGetDCEx(pWnd, NULL, DCX_USESTYLE | DCX_WINDOW);
    if (!hdc)
    {
        ERR("UserGetDCEx failed!\n");
        goto cleanup;
    }

    cx = pWnd->rcWindow.right - pWnd->rcWindow.left;
    cy = pWnd->rcWindow.bottom - pWnd->rcWindow.top;

    hbm = NtGdiCreateCompatibleBitmap(hdc, cx, cy);
    if (!hbm)
    {
        ERR("NtGdiCreateCompatibleBitmap failed!\n");
        goto cleanup;
    }

    hdcMem = NtGdiCreateCompatibleDC(hdc);
    if (!hdcMem)
    {
        ERR("NtGdiCreateCompatibleDC failed!\n");
        goto cleanup;
    }

    hbmOld = NtGdiSelectBitmap(hdcMem, hbm);
    NtGdiBitBlt(hdcMem, 0, 0, cx, cy, hdc, 0, 0, SRCCOPY, CLR_INVALID, 0);
    NtGdiSelectBitmap(hdcMem, hbmOld);
    IntGdiDeleteDC(hdcMem, FALSE);

    /* Save snapshot in clipboard */
    if (UserOpenClipboard(NULL))
    {
        UserEmptyClipboard();
        scd.fIncSerialNumber = TRUE;
        scd.fGlobalHandle = FALSE;
        if (UserSetClipboardData(CF_BITMAP, hbm, &scd))
        {
            /* Bitmap is managed by system now */
            hbm = NULL;
        }
        UserCloseClipboard();
    }

cleanup:
    if (hbm)
        GreDeleteObject(hbm);
    if (hdc)
        UserReleaseDC(pWnd, hdc, FALSE);
}

/* Find the next/previous keyboard layout of the same/different language */
static PKL FASTCALL
IntGetNextKL(
    _In_ PKL pKL,
    _In_ BOOL bNext,
    _In_ BOOL bSameLang)
{
    PKL pFirstKL = pKL;
    LANGID LangID = LOWORD(pKL->hkl);

    do
    {
        pKL = (bNext ? pKL->pklNext : pKL->pklPrev);

        if (!(pKL->dwKL_Flags & KL_UNLOAD) && bSameLang == (LangID == LOWORD(pKL->hkl)))
            return pKL;
    } while (pKL != pFirstKL);

    return pFirstKL;
}

/* Perform layout toggle by [Left Alt]+Shift or Ctrl+Shift */
static VOID
IntLanguageToggle(
    _In_ PUSER_MESSAGE_QUEUE pFocusQueue,
    _In_ BOOL bSameLang,
    _In_ INT nKeyState)
{
    PWND pWnd;
    PTHREADINFO pti;
    PKL pkl;
    WPARAM wParam = 0;

    if (!pFocusQueue)
    {
        ERR("IntLanguageToggle(): NULL pFocusQueue\n");
        return;
    }
    pWnd = pFocusQueue->spwndFocus;
    if (!pWnd)
        pWnd = pFocusQueue->spwndActive;
    if (!pWnd)
        return;

    pti = pWnd->head.pti;
    pkl = pti->KeyboardLayout;

    if (nKeyState == INPUTLANGCHANGE_FORWARD)
        pkl = IntGetNextKL(pkl, TRUE, bSameLang);
    else if (nKeyState == INPUTLANGCHANGE_BACKWARD)
        pkl = IntGetNextKL(pkl, FALSE, bSameLang);

    if (gSystemFS & pkl->dwFontSigs)
        wParam |= INPUTLANGCHANGE_SYSCHARSET;

    UserPostMessage(UserHMGetHandle(pWnd), WM_INPUTLANGCHANGEREQUEST, wParam, (LPARAM)pkl->hkl);
}

/* Check Language Toggle by [Left Alt]+Shift or Ctrl+Shift */
static BOOL
IntCheckLanguageToggle(
    _In_ PUSER_MESSAGE_QUEUE pFocusQueue,
    _In_ BOOL bIsDown,
    _In_ WORD wVk,
    _Inout_ PINT pKeyState)
{
    if (bIsDown) /* Toggle key combination is pressed? */
    {
        if (wVk == VK_LSHIFT)
            *pKeyState = INPUTLANGCHANGE_FORWARD;
        else if (wVk == VK_RSHIFT)
            *pKeyState = INPUTLANGCHANGE_BACKWARD;
        else if (!wVk && IS_KEY_DOWN(gafAsyncKeyState, VK_LSHIFT))
            *pKeyState = INPUTLANGCHANGE_FORWARD;
        else if (!wVk && IS_KEY_DOWN(gafAsyncKeyState, VK_RSHIFT))
            *pKeyState = INPUTLANGCHANGE_BACKWARD;
        else
            return FALSE;
    }
    else
    {
        if (*pKeyState == 0)
            return FALSE;

        IntLanguageToggle(pFocusQueue, (pKeyState == &gLayoutToggleKeyState), *pKeyState);
        *pKeyState = 0;
    }
    return TRUE;
}

/*
 * UserSendKeyboardInput
 *
 * Process keyboard input from input devices and SendInput API
 */
BOOL NTAPI
ProcessKeyEvent(WORD wVk, WORD wScanCode, DWORD dwFlags, BOOL bInjected, DWORD dwTime, DWORD dwExtraInfo)
{
    WORD wSimpleVk = 0, wFixedVk, wVk2;
    PUSER_MESSAGE_QUEUE pFocusQueue;
    PTHREADINFO pti;
    BOOL bExt = (dwFlags & KEYEVENTF_EXTENDEDKEY) ? TRUE : FALSE;
    BOOL bIsDown = (dwFlags & KEYEVENTF_KEYUP) ? FALSE : TRUE;
    BOOL bPacket = (dwFlags & KEYEVENTF_UNICODE) ? TRUE : FALSE;
    BOOL bWasSimpleDown = FALSE, bPostMsg = TRUE, bIsSimpleDown;
    MSG Msg;
    static BOOL bMenuDownRecently = FALSE;
    BOOL bLangToggled = FALSE;

    /* Get virtual key without shifts (VK_(L|R)* -> VK_*) */
    wSimpleVk = IntSimplifyVk(wVk);

    if (PRIMARYLANGID(gusLanguageID) == LANG_JAPANESE)
    {
        /* Japanese special! */
        if (IS_KEY_DOWN(gafAsyncKeyState, VK_SHIFT))
        {
            if (wSimpleVk == VK_OEM_ATTN)
                wSimpleVk = VK_CAPITAL;
            else if (wSimpleVk == VK_OEM_COPY)
                wSimpleVk = VK_OEM_FINISH;
        }
    }

    bWasSimpleDown = IS_KEY_DOWN(gafAsyncKeyState, wSimpleVk);

    /* Update key without shifts */
    wVk2 = IntFixVk(wSimpleVk, !bExt);
    bIsSimpleDown = bIsDown || IS_KEY_DOWN(gafAsyncKeyState, wVk2);
    UpdateAsyncKeyState(wSimpleVk, bIsSimpleDown);

    if (bIsDown)
    {
        /* Update keyboard LEDs */
        IntKeyboardUpdateLeds(ghKeyboardDevice,
                              wSimpleVk,
                              wScanCode);
    }

    /* Call WH_KEYBOARD_LL hook */
    if (co_CallLowLevelKeyboardHook(wVk, wScanCode, dwFlags, bInjected, dwTime, dwExtraInfo))
    {
        ERR("Kbd msg dropped by WH_KEYBOARD_LL hook\n");
        bPostMsg = FALSE;
    }

    /* Check if this is a hotkey */
    if (co_UserProcessHotKeys(wSimpleVk, bIsDown)) //// Check if this is correct, refer to hotkey sequence message tests.
    {
        TRACE("HotKey Processed\n");
        bPostMsg = FALSE;
    }

    wFixedVk = IntFixVk(wSimpleVk, bExt); /* LSHIFT + EXT = RSHIFT */
    if (wSimpleVk == VK_SHIFT) /* shift can't be extended */
        bExt = FALSE;

    /* If we have a focus queue, post a keyboard message */
    pFocusQueue = IntGetFocusMessageQueue();
    TRACE("ProcessKeyEvent Q 0x%p Active pWnd 0x%p Focus pWnd 0x%p\n",
           pFocusQueue,
           (pFocusQueue ?  pFocusQueue->spwndActive : 0),
           (pFocusQueue ?  pFocusQueue->spwndFocus : 0));

    /* If it is F10 or ALT is down and CTRL is up, it's a system key */
    if ( wVk == VK_F10 ||
        (wSimpleVk == VK_MENU && bMenuDownRecently) ||
        (IS_KEY_DOWN(gafAsyncKeyState, VK_MENU) &&
        !IS_KEY_DOWN(gafAsyncKeyState, VK_CONTROL)) ||
         // See MSDN WM_SYSKEYDOWN/UP fixes last wine Win test_keyboard_input.
        (pFocusQueue && !pFocusQueue->spwndFocus) )
    {
        bMenuDownRecently = FALSE; // reset
        if (bIsDown)
        {
            Msg.message = WM_SYSKEYDOWN;
            if (wSimpleVk == VK_MENU)
            {
                // Note: If only LALT is pressed WM_SYSKEYUP is generated instead of WM_KEYUP
                bMenuDownRecently = TRUE;
            }
        }
        else
            Msg.message = WM_SYSKEYUP;
    }
    else
    {
        if (bIsDown)
            Msg.message = WM_KEYDOWN;
        else
            Msg.message = WM_KEYUP;
    }

    /* Update async state of not simplified vk here.
       See user32_apitest:GetKeyState */
    UpdateAsyncKeyState(wFixedVk, bIsDown);

    /* Alt-Tab/Esc Check. Use FocusQueue or RIT Queue */
    if (bIsSimpleDown && !bWasSimpleDown &&
        IS_KEY_DOWN(gafAsyncKeyState, VK_MENU) &&
        !IS_KEY_DOWN(gafAsyncKeyState, VK_CONTROL) &&
        (wVk == VK_ESCAPE || wVk == VK_TAB))
    {
       TRACE("Alt-Tab/Esc Pressed wParam %x\n",wVk);
    }

    /*
     * Check Language/Layout Toggle by [Left Alt]+Shift or Ctrl+Shift.
     * @see https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-2000-server/cc976564%28v=technet.10%29
     */
    if (gdwLanguageToggleKey == 1 || gdwLanguageToggleKey == 2)
    {
        if (wSimpleVk == VK_SHIFT) /* Shift key is pressed or released */
        {
            UINT targetKey = ((gdwLanguageToggleKey == 1) ? VK_LMENU : VK_CONTROL);
            if (IS_KEY_DOWN(gafAsyncKeyState, targetKey))
                bLangToggled = IntCheckLanguageToggle(pFocusQueue, bIsDown, wVk, &gLanguageToggleKeyState);
        }
        else if ((wSimpleVk == VK_MENU && gdwLanguageToggleKey == 1) ||
                 (wSimpleVk == VK_CONTROL && gdwLanguageToggleKey == 2))
        {
            if (IS_KEY_DOWN(gafAsyncKeyState, VK_SHIFT))
                bLangToggled = IntCheckLanguageToggle(pFocusQueue, bIsDown, 0, &gLanguageToggleKeyState);
        }
    }
    if (!bLangToggled && (gdwLayoutToggleKey == 1 || gdwLayoutToggleKey == 2))
    {
        if (wSimpleVk == VK_SHIFT) /* Shift key is pressed or released */
        {
            UINT targetKey = ((gdwLayoutToggleKey == 1) ? VK_LMENU : VK_CONTROL);
            if (IS_KEY_DOWN(gafAsyncKeyState, targetKey))
                IntCheckLanguageToggle(pFocusQueue, bIsDown, wVk, &gLayoutToggleKeyState);
        }
        else if ((wSimpleVk == VK_MENU && gdwLayoutToggleKey == 1) ||
                 (wSimpleVk == VK_CONTROL && gdwLayoutToggleKey == 2))
        {
            if (IS_KEY_DOWN(gafAsyncKeyState, VK_SHIFT))
                IntCheckLanguageToggle(pFocusQueue, bIsDown, 0, &gLayoutToggleKeyState);
        }
    }

    if (bIsDown && wVk == VK_SNAPSHOT)
    {
        if (pFocusQueue &&
            IS_KEY_DOWN(gafAsyncKeyState, VK_MENU) &&
            !IS_KEY_DOWN(gafAsyncKeyState, VK_CONTROL))
        {
            // Snap from Active Window, Focus can be null.
            SnapWindow(pFocusQueue->spwndActive ? UserHMGetHandle(pFocusQueue->spwndActive) : 0);
        }
        else
            SnapWindow(NULL); // Snap Desktop.
    }
    else if (pFocusQueue && bPostMsg)
    {
        PWND Wnd = pFocusQueue->spwndFocus; // SysInit.....

        pti = pFocusQueue->ptiKeyboard;

        if (!Wnd && pFocusQueue->spwndActive) // SysInit.....
        {
           // Going with Active. WM_SYSKEYXXX last wine Win test_keyboard_input.
           Wnd = pFocusQueue->spwndActive;
        }
        if (Wnd) pti = Wnd->head.pti;

        /* Init message */
        Msg.hwnd = Wnd ? UserHMGetHandle(Wnd) : NULL;
        Msg.wParam = wFixedVk & 0xFF; /* Note: It's simplified by msg queue */
        Msg.lParam = MAKELPARAM(1, wScanCode);
        Msg.time = dwTime;
        Msg.pt = gpsi->ptCursor;

        if ( Msg.message == WM_KEYDOWN || Msg.message == WM_SYSKEYDOWN )
        {
           if ( (Msg.wParam == VK_SHIFT ||
                 Msg.wParam == VK_CONTROL ||
                 Msg.wParam == VK_MENU ) &&
               !IS_KEY_DOWN(gafAsyncKeyState, Msg.wParam))
           {
              ERR("Set last input\n");
              //ptiLastInput = pti;
           }
        }

        /* If it is VK_PACKET, high word of wParam is used for wchar */
        if (!bPacket)
        {
            if (bExt)
                Msg.lParam |= KF_EXTENDED << 16;
            if (IS_KEY_DOWN(gafAsyncKeyState, VK_MENU))
                Msg.lParam |= KF_ALTDOWN << 16;
            if (bWasSimpleDown)
                Msg.lParam |= KF_REPEAT << 16;
            if (!bIsDown)
                Msg.lParam |= KF_UP << 16;
            /* FIXME: Set KF_DLGMODE and KF_MENUMODE when needed */
            if (pFocusQueue->QF_flags & QF_DIALOGACTIVE)
                Msg.lParam |= KF_DLGMODE << 16;
            if (pFocusQueue->MenuOwner) // pti->pMenuState->fMenuStarted
                Msg.lParam |= KF_MENUMODE << 16;
        }

        // Post mouse move before posting key buttons, to keep it syned.
        if (pFocusQueue->QF_flags & QF_MOUSEMOVED)
        {
           IntCoalesceMouseMove(pti);
        }

        /* Post a keyboard message */
        MsqPostMessage(pti, &Msg, TRUE, QS_KEY, 0, dwExtraInfo);
    }
    return TRUE;
}
/* --- pretranslation split: the remaining function groups live in keyboard.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "keyboard.split2.c"
