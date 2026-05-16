/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS Win32k subsystem
 * PURPOSE:          Input Method Editor and Input Method Manager support
 * FILE:             win32ss/user/ntuser/ime.c
 * PROGRAMERS:       Casper S. Hornstrup (chorns@users.sourceforge.net)
 *                   Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
 */

#include <win32k.h>
#include <safe_user_io.h>
#include <jpnvkeys.h>

DBG_DEFAULT_CHANNEL(UserMisc);

#define INVALID_THREAD_ID  ((ULONG)-1)
#define INVALID_HOTKEY     ((UINT)-1)
#define MOD_KEYS           (MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_WIN)
#define MOD_LEFT_RIGHT     (MOD_LEFT | MOD_RIGHT)

#define LANGID_CHINESE_SIMPLIFIED   MAKELANGID(LANG_CHINESE,  SUBLANG_CHINESE_SIMPLIFIED)
#define LANGID_JAPANESE             MAKELANGID(LANG_JAPANESE, SUBLANG_DEFAULT)
#define LANGID_KOREAN               MAKELANGID(LANG_KOREAN,   SUBLANG_KOREAN)
#define LANGID_CHINESE_TRADITIONAL  MAKELANGID(LANG_CHINESE,  SUBLANG_CHINESE_TRADITIONAL)
#define LANGID_NEUTRAL              MAKELANGID(LANG_NEUTRAL,  SUBLANG_NEUTRAL)

HIMC ghIMC = NULL;
BOOL gfImeOpen = (BOOL)-1;
DWORD gdwImeConversion = (DWORD)-1;
BOOL gfIMEShowStatus = (BOOL)-1;

typedef struct tagIMEHOTKEY
{
    struct tagIMEHOTKEY *pNext;
    DWORD  dwHotKeyId;
    UINT   uVirtualKey;
    UINT   uModifiers;
    HKL    hKL;
} IMEHOTKEY, *PIMEHOTKEY;

PIMEHOTKEY gpImeHotKeyList = NULL;
LCID glcidSystem = 0;

static inline PIMEUI FASTCALL IntGetImeUIFromWnd(_In_ PWND pWnd)
{
    ASSERT(pWnd->cbwndExtra >= sizeof(PIMEUI));
    PIMEWND pImeWnd = (PIMEWND)pWnd;
    return pImeWnd->pimeui;
}

static DWORD FASTCALL
IntGetImeCompatFlags(_In_opt_ PTHREADINFO pti)
{
    if (!pti)
        pti = PsGetCurrentThreadWin32Thread();

    return pti->ppi->dwImeCompatFlags;
}

UINT FASTCALL
IntGetImeHotKeyLanguageScore(
    _In_ HKL hKL,
    _In_ LANGID HotKeyLangId)
{
    LCID lcid;

    if (HotKeyLangId == LANGID_NEUTRAL || HotKeyLangId == LOWORD(hKL))
        return 3;

    _SEH2_TRY
    {
        lcid = NtCurrentTeb()->CurrentLocale;
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception in IntGetImeHotKeyLanguageScore: TEB=%p, hKL=%p, HotKeyLangId=0x%04x\n", NtCurrentTeb(), hKL, HotKeyLangId);
        lcid = MAKELCID(LANGID_NEUTRAL, SORT_DEFAULT);
    }
    _SEH2_END;

    if (HotKeyLangId == LANGIDFROMLCID(lcid))
        return 2;

    if (glcidSystem == 0)
        ZwQueryDefaultLocale(FALSE, &glcidSystem);

    if (HotKeyLangId == LANGIDFROMLCID(glcidSystem))
        return 1;

    return 0;
}

HKL FASTCALL
IntGetActiveKeyboardLayout(VOID)
{
    PTHREADINFO pti;

    if (gpqForeground && gpqForeground->spwndActive)
    {
        pti = gpqForeground->spwndActive->head.pti;
        if (pti && pti->KeyboardLayout)
            return pti->KeyboardLayout->hkl;
    }

    return UserGetKeyboardLayout(0);
}

static LANGID FASTCALL
IntGetImeHotKeyLangId(_In_ DWORD dwHotKeyId)
{
#define IME_CHOTKEY 0x10
#define IME_JHOTKEY 0x30
#define IME_KHOTKEY 0x50
#define IME_THOTKEY 0x70
#define IME_XHOTKEY 0x90
    static const LANGID s_array[] =
    {
        /* 0x00 */ (WORD)-1,
        /* 0x10 */ LANGID_CHINESE_SIMPLIFIED,
        /* 0x20 */ LANGID_CHINESE_SIMPLIFIED,
        /* 0x30 */ LANGID_JAPANESE,
        /* 0x40 */ LANGID_JAPANESE,
        /* 0x50 */ LANGID_KOREAN,
        /* 0x60 */ LANGID_KOREAN,
        /* 0x70 */ LANGID_CHINESE_TRADITIONAL,
        /* 0x80 */ LANGID_CHINESE_TRADITIONAL
    };

    if (IME_CHOTKEY <= dwHotKeyId && dwHotKeyId < IME_XHOTKEY)
        return s_array[(dwHotKeyId & 0xF0) >> 4];
    return LANGID_NEUTRAL;
}

static VOID FASTCALL
IntAddImeHotKey(
    _In_ PIMEHOTKEY *ppList,
    _In_ PIMEHOTKEY pHotKey)
{
    PIMEHOTKEY pNode;

    if (!*ppList)
    {
        *ppList = pHotKey;
        return;
    }

    for (pNode = *ppList; pNode; pNode = pNode->pNext)
    {
        if (!pNode->pNext)
        {
            pNode->pNext = pHotKey;
            return;
        }
    }
}

static PIMEHOTKEY FASTCALL
IntGetImeHotKeyById(
    _In_ PIMEHOTKEY pList,
    _In_ DWORD dwHotKeyId)
{
    PIMEHOTKEY pNode;
    for (pNode = pList; pNode; pNode = pNode->pNext)
    {
        if (pNode->dwHotKeyId == dwHotKeyId)
            return pNode;
    }
    return NULL;
}

static PIMEHOTKEY APIENTRY
IntGetImeHotKeyByKeyAndLang(
    _In_ PIMEHOTKEY pList,
    _In_ UINT uModKeys,
    _In_ UINT uLeftRight,
    _In_ UINT uVirtualKey,
    _In_ LANGID TargetLangId)
{
    PIMEHOTKEY pNode;
    LANGID LangID;
    UINT uModifiers;

    for (pNode = pList; pNode; pNode = pNode->pNext)
    {
        if (pNode->uVirtualKey != uVirtualKey)
            continue;

        LangID = IntGetImeHotKeyLangId(pNode->dwHotKeyId);
        if (LangID != TargetLangId && LangID != 0)
            continue;

        uModifiers = pNode->uModifiers;
        if (uModifiers & MOD_IGNORE_ALL_MODIFIER)
            return pNode;

        if ((uModifiers & MOD_KEYS) != uModKeys)
            continue;

        if ((uModifiers & uLeftRight) || (uModifiers & MOD_LEFT_RIGHT) == uLeftRight)
            return pNode;
    }

    return NULL;
}

static VOID FASTCALL
IntDeleteImeHotKey(
    _Inout_ PIMEHOTKEY *ppList,
    _In_ PIMEHOTKEY pHotKey)
{
    PIMEHOTKEY pNode;

    if (*ppList == pHotKey)
    {
        *ppList = pHotKey->pNext;
        ExFreePoolWithTag(pHotKey, USERTAG_IMEHOTKEY);
        return;
    }

    for (pNode = *ppList; pNode; pNode = pNode->pNext)
    {
        if (pNode->pNext == pHotKey)
        {
            pNode->pNext = pHotKey->pNext;
            ExFreePoolWithTag(pHotKey, USERTAG_IMEHOTKEY);
            return;
        }
    }
}

static PIMEHOTKEY
IntGetImeHotKeyByKey(
    _In_ PIMEHOTKEY pList,
    _In_ UINT uModKeys,
    _In_ UINT uLeftRight,
    _In_ UINT uVirtualKey)
{
    PIMEHOTKEY pNode, ret = NULL;
    PTHREADINFO pti = GetW32ThreadInfo();
    LANGID LangId;
    HKL hKL = IntGetActiveKeyboardLayout();
    BOOL fKorean = (PRIMARYLANGID(LOWORD(hKL)) == LANG_KOREAN);
    UINT nScore, nMaxScore = 0;

    for (pNode = pList; pNode; pNode = pNode->pNext)
    {
        if (pNode->uVirtualKey != uVirtualKey)
            continue;

        if ((pNode->uModifiers & MOD_IGNORE_ALL_MODIFIER))
        {
            ;
        }
        else if ((pNode->uModifiers & MOD_KEYS) != uModKeys)
        {
            continue;
        }
        else if ((pNode->uModifiers & uLeftRight) ||
                 (pNode->uModifiers & MOD_LEFT_RIGHT) == uLeftRight)
        {
            ;
        }
        else
        {
            continue;
        }

        LangId = IntGetImeHotKeyLangId(pNode->dwHotKeyId);
        nScore = IntGetImeHotKeyLanguageScore(hKL, LangId);
        if (nScore >= 3)
            return pNode;

        if (fKorean)
            continue;

        if (nScore == 0)
        {
            if (pNode->dwHotKeyId == IME_CHOTKEY_IME_NONIME_TOGGLE ||
                pNode->dwHotKeyId == IME_THOTKEY_IME_NONIME_TOGGLE)
            {
                if (LOWORD(pti->hklPrev) == LangId)
                    return pNode;
            }
        }

        if (nMaxScore < nScore)
        {
            nMaxScore = nScore;
            ret = pNode;
        }
    }

    return ret;
}

static PIMEHOTKEY
IntCheckImeHotKey(
    _In_ const USER_MESSAGE_QUEUE *MessageQueue,
    _In_ UINT uVirtualKey,
    _In_ LPARAM lParam)
{
    PIMEHOTKEY pHotKey;
    UINT uModifiers;
    BOOL bKeyUp = (lParam & 0x80000000);
    const BYTE *KeyState = MessageQueue->afKeyState;
    static UINT s_uKeyUpVKey = 0;

    if (bKeyUp)
    {
        if (s_uKeyUpVKey != uVirtualKey)
        {
            s_uKeyUpVKey = 0;
            return NULL;
        }

        s_uKeyUpVKey = 0;
    }

    uModifiers = 0;
    if (IS_KEY_DOWN(KeyState, VK_LSHIFT))   uModifiers |= (MOD_SHIFT | MOD_LEFT);
    if (IS_KEY_DOWN(KeyState, VK_RSHIFT))   uModifiers |= (MOD_SHIFT | MOD_RIGHT);
    if (IS_KEY_DOWN(KeyState, VK_LCONTROL)) uModifiers |= (MOD_CONTROL | MOD_LEFT);
    if (IS_KEY_DOWN(KeyState, VK_RCONTROL)) uModifiers |= (MOD_CONTROL | MOD_RIGHT);
    if (IS_KEY_DOWN(KeyState, VK_LMENU))    uModifiers |= (MOD_ALT | MOD_LEFT);
    if (IS_KEY_DOWN(KeyState, VK_RMENU))    uModifiers |= (MOD_ALT | MOD_RIGHT);

    pHotKey = IntGetImeHotKeyByKey(gpImeHotKeyList,
                                   (uModifiers & MOD_KEYS),
                                   (uModifiers & MOD_LEFT_RIGHT),
                                   uVirtualKey);
    if (pHotKey)
    {
        if (bKeyUp)
        {
            if (pHotKey->uModifiers & MOD_ON_KEYUP)
                return pHotKey;
        }
        else
        {
            if (pHotKey->uModifiers & MOD_ON_KEYUP)
                s_uKeyUpVKey = uVirtualKey;
            else
                return pHotKey;
        }
    }

    return NULL;
}

VOID FASTCALL
IntFreeImeHotKeys(VOID)
{
    PIMEHOTKEY pNode, pNext;
    for (pNode = gpImeHotKeyList; pNode; pNode = pNext)
    {
        pNext = pNode->pNext;
        ExFreePoolWithTag(pNode, USERTAG_IMEHOTKEY);
    }
    gpImeHotKeyList = NULL;
}

static BOOL APIENTRY
IntSetImeHotKey(
    _In_ DWORD dwHotKeyId,
    _In_ UINT uModifiers,
    _In_ UINT uVirtualKey,
    _In_opt_ HKL hKL,
    _In_ DWORD dwAction)
{
    PIMEHOTKEY pNode;
    LANGID LangId;

    switch (dwAction)
    {
        case SETIMEHOTKEY_DELETE:
            pNode = IntGetImeHotKeyById(gpImeHotKeyList, dwHotKeyId); /* Find hotkey by ID */
            if (!pNode)
            {
                ERR("dwHotKeyId: 0x%lX\n", dwHotKeyId);
                return FALSE;
            }

            IntDeleteImeHotKey(&gpImeHotKeyList, pNode); /* Delete it */
            return TRUE;

        case SETIMEHOTKEY_ADD:
            if (LOWORD(uVirtualKey) == VK_PACKET) /* In case of VK_PACKET */
                return FALSE;

            LangId = IntGetImeHotKeyLangId(dwHotKeyId);
            if (LangId == LANGID_KOREAN)
                return FALSE; /* Korean can't add IME hotkeys */

            /* Find hotkey by key and language */
            pNode = IntGetImeHotKeyByKeyAndLang(gpImeHotKeyList,
                                                (uModifiers & MOD_KEYS),
                                                (uModifiers & MOD_LEFT_RIGHT),
                                                uVirtualKey, LangId);
            if (pNode == NULL) /* If not found */
                pNode = IntGetImeHotKeyById(gpImeHotKeyList, dwHotKeyId); /* Find by ID */

            if (pNode) /* Already exists */
            {
                pNode->uModifiers = uModifiers;
                pNode->uVirtualKey = uVirtualKey;
                pNode->hKL = hKL;
                return TRUE;
            }

            /* Allocate new hotkey */
            pNode = ExAllocatePoolWithTag(PagedPool, sizeof(IMEHOTKEY), USERTAG_IMEHOTKEY);
            if (!pNode)
                return FALSE;

            /* Populate */
            pNode->pNext = NULL;
            pNode->dwHotKeyId = dwHotKeyId;
            pNode->uModifiers = uModifiers;
            pNode->uVirtualKey = uVirtualKey;
            pNode->hKL = hKL;
            IntAddImeHotKey(&gpImeHotKeyList, pNode); /* Add it */
            return TRUE;

        case SETIMEHOTKEY_INITIALIZE:
            IntFreeImeHotKeys(); /* Delete all the IME hotkeys */
            return TRUE;

        default:
            ERR("0x%lX\n", dwAction);
            return FALSE;
    }
}

BOOL NTAPI
NtUserGetImeHotKey(
    _In_ DWORD dwHotKeyId,
    _Out_ PUINT lpuModifiers,
    _Out_ PUINT lpuVirtualKey,
    _Out_opt_ LPHKL lphKL)
{
    PIMEHOTKEY pNode = NULL;
    NTSTATUS Status;

    UserEnterExclusive();

    Status = SafeProbeForWrite(lpuModifiers, sizeof(UINT), 1);
    if (NT_SUCCESS(Status))
        Status = SafeProbeForWrite(lpuVirtualKey, sizeof(UINT), 1);
    if (NT_SUCCESS(Status) && lphKL)
        Status = SafeProbeForWrite(lphKL, sizeof(HKL), 1);
    if (!NT_SUCCESS(Status))
    {
        ERR("Exception in NtUserGetImeHotKey: %p, %p, %p\n", lpuModifiers, lpuVirtualKey, lphKL);
        goto Quit;
    }

    pNode = IntGetImeHotKeyById(gpImeHotKeyList, dwHotKeyId);
    if (!pNode)
        goto Quit;

    _SEH2_TRY
    {
        *lpuModifiers = pNode->uModifiers;
        *lpuVirtualKey = pNode->uVirtualKey;
        if (lphKL)
            *lphKL = pNode->hKL;
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception in NtUserGetImeHotKey: %p, %p, %p, %p\n", pNode, lpuModifiers, lpuVirtualKey, lphKL);
        pNode = NULL;
    }
    _SEH2_END;

Quit:
    UserLeave();
    return !!pNode;
}


BOOL
NTAPI
NtUserSetImeHotKey(
    _In_ DWORD dwHotKeyId,
    _In_ UINT uModifiers,
    _In_ UINT uVirtualKey,
    _In_opt_ HKL hKL,
    _In_ DWORD dwAction)
{
    BOOL ret;
    UserEnterExclusive();
    ret = IntSetImeHotKey(dwHotKeyId, uModifiers, uVirtualKey, hKL, dwAction);
    UserLeave();
    return ret;
}

DWORD
NTAPI
NtUserCheckImeHotKey(
    _In_ UINT uVirtualKey,
    _In_ LPARAM lParam)
{
    PIMEHOTKEY pNode;
    DWORD ret = INVALID_HOTKEY;

    UserEnterExclusive();

    if (!gpqForeground || !IS_IMM_MODE())
        goto Quit;

    pNode = IntCheckImeHotKey(gpqForeground, uVirtualKey, lParam);
    if (pNode)
        ret = pNode->dwHotKeyId;

Quit:
    UserLeave();
    return ret;
}

PWND FASTCALL
IntGetTopLevelWindow(_In_ PWND pwnd)
{
    if (!pwnd)
        return NULL;

    while (pwnd->style & WS_CHILD)
        pwnd = pwnd->spwndParent;

    return pwnd;
}

static HIMC FASTCALL
IntAssociateInputContext(_Inout_ PWND pWnd, _In_ PIMC pImc)
{
    HIMC hOldImc = pWnd->hImc;
    pWnd->hImc = (pImc ? UserHMGetHandle(pImc) : NULL);
    return hOldImc;
}

DWORD
NTAPI
NtUserSetThreadLayoutHandles(
    _In_ HKL hNewKL,
    _In_ HKL hOldKL)
{
    PTHREADINFO pti;
    PKL pOldKL, pNewKL;

    UserEnterExclusive();

    pti = GetW32ThreadInfo();
    pOldKL = pti->KeyboardLayout;
    if (pOldKL && pOldKL->hkl != hOldKL)
        goto Quit;

    pNewKL = UserHklToKbl(hNewKL);
    if (!pNewKL)
        goto Quit;

    if (IS_IME_HKL(hNewKL) != IS_IME_HKL(hOldKL))
        pti->hklPrev = hOldKL;

    UserAssignmentLock((PVOID*)&pti->KeyboardLayout, pNewKL);
    pti->pClientInfo->hKL = pNewKL->hkl;

Quit:
    UserLeave();
    return 0;
}

DWORD FASTCALL
UserBuildHimcList(
    _Inout_ PTHREADINFO pti,
    _In_ DWORD dwCount,
    _Inout_ HIMC *phList)
{
    PIMC pIMC;
    DWORD dwRealCount = 0;

    if (pti)
    {
        for (pIMC = pti->spDefaultImc; pIMC; pIMC = pIMC->pImcNext)
        {
            if (dwRealCount < dwCount)
                phList[dwRealCount] = UserHMGetHandle(pIMC);

            ++dwRealCount;
        }
    }
    else
    {
        for (pti = gptiCurrent->ppi->ptiList; pti; pti = pti->ptiSibling)
        {
            for (pIMC = pti->spDefaultImc; pIMC; pIMC = pIMC->pImcNext)
            {
                if (dwRealCount < dwCount)
                    phList[dwRealCount] = UserHMGetHandle(pIMC);

                ++dwRealCount;
            }
        }
    }

    return dwRealCount;
}

UINT FASTCALL
IntImmProcessKey(
    _In_ PUSER_MESSAGE_QUEUE MessageQueue,
    _In_ PWND pWnd,
    _In_ UINT uMsg,
    _In_ WPARAM wParam,
    _In_ LPARAM lParam)
{
    UINT uVirtualKey, ret;
    DWORD dwHotKeyId;
    PKL pKL;
    PIMC pIMC;
    PIMEHOTKEY pImeHotKey;
    HKL hKL;
    HWND hWnd;

    ASSERT_REFS_CO(pWnd);

    switch (uMsg)
    {
        case WM_KEYDOWN:
        case WM_KEYUP:
        case WM_SYSKEYDOWN:
        case WM_SYSKEYUP:
            break;

        default:
            return 0;
    }

    pIMC = NULL;
    hWnd = UserHMGetHandle(pWnd);
    pKL = pWnd->head.pti->KeyboardLayout;
    if (!pKL)
        return 0;

    uVirtualKey = LOBYTE(wParam);
    pImeHotKey = IntCheckImeHotKey(MessageQueue, uVirtualKey, lParam);
    if (pImeHotKey)
    {
        dwHotKeyId = pImeHotKey->dwHotKeyId;
        hKL = pImeHotKey->hKL;
    }
    else
    {
        dwHotKeyId = INVALID_HOTKEY;
        hKL = NULL;
    }

    if (IME_HOTKEY_DSWITCH_FIRST <= dwHotKeyId && dwHotKeyId <= IME_HOTKEY_DSWITCH_LAST)
    {
        if (pKL->hkl != hKL)
        {
            UserPostMessage(hWnd, WM_INPUTLANGCHANGEREQUEST,
                            ((pKL->dwFontSigs & gSystemFS) ? INPUTLANGCHANGE_SYSCHARSET : 0),
                            (LPARAM)hKL);
        }

        if (IntGetImeCompatFlags(pWnd->head.pti) & 0x800000)
            return 0;

        return IPHK_HOTKEY;
    }

    if (!IS_IMM_MODE())
    {
        if (dwHotKeyId != INVALID_HOTKEY)
            WARN("!IS_IMM_MODE(): dwHotKeyId 0x%X\n", dwHotKeyId);

        return 0;
    }

    if (dwHotKeyId == INVALID_HOTKEY)
    {
        if (!pKL->piiex)
            return 0;

        if (pWnd->hImc)
            pIMC = UserGetObject(gHandleTable, pWnd->hImc, TYPE_INPUTCONTEXT);
        if (!pIMC)
            return 0;

        if ((lParam & (KF_UP << 16)) &&
            (pKL->piiex->ImeInfo.fdwProperty & IME_PROP_IGNORE_UPKEYS))
        {
            return 0;
        }

        switch (uVirtualKey)
        {
            case VK_DBE_CODEINPUT:
            case VK_DBE_ENTERCONFIGMODE:
            case VK_DBE_ENTERWORDREGISTERMODE:
            case VK_DBE_HIRAGANA:
            case VK_DBE_KATAKANA:
            case VK_DBE_NOCODEINPUT:
            case VK_DBE_NOROMAN:
            case VK_DBE_ROMAN:
                break;

            default:
            {
                if (uMsg == WM_SYSKEYDOWN || uMsg == WM_SYSKEYUP)
                {
                    if (uVirtualKey != VK_MENU && uVirtualKey != VK_F10)
                        return 0;
                }

                if (!(pKL->piiex->ImeInfo.fdwProperty & IME_PROP_NEED_ALTKEY))
                {
                    if (uVirtualKey == VK_MENU || (lParam & 0x20000000))
                        return 0;
                }
                break;
            }
        }
    }

    if (LOBYTE(uVirtualKey) == VK_PACKET)
        uVirtualKey = MAKELONG(wParam, GetW32ThreadInfo()->wchInjected);

    ret = co_IntImmProcessKey(hWnd, pKL->hkl, uVirtualKey, lParam, dwHotKeyId);

    if (IntGetImeCompatFlags(pWnd->head.pti) & 0x800000)
        ret &= ~IPHK_HOTKEY;

    return ret;
}

NTSTATUS
NTAPI
NtUserBuildHimcList(
    _In_ DWORD dwThreadId,
    _In_ DWORD dwCount,
    _Out_ HIMC *phList,
    _Out_ PDWORD pdwCount)
{
    NTSTATUS ret = STATUS_UNSUCCESSFUL;
    DWORD dwRealCount;
    PTHREADINFO pti;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        goto Quit;
    }

    if (dwThreadId == 0)
    {
        pti = gptiCurrent;
    }
    else if (dwThreadId == INVALID_THREAD_ID)
    {
        pti = NULL;
    }
    else
    {
        pti = IntTID2PTI(UlongToHandle(dwThreadId));
        if (!pti || !pti->rpdesk)
            goto Quit;
    }

    _SEH2_TRY
    {
        ProbeForWrite(phList, dwCount * sizeof(HIMC), 1);
        ProbeForWrite(pdwCount, sizeof(DWORD), 1);
        *pdwCount = dwRealCount = UserBuildHimcList(pti, dwCount, phList);
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception in NtUserBuildHimcList: %p, %p\n", phList, pdwCount);
        _SEH2_YIELD(goto Quit);
    }
    _SEH2_END;

    if (dwCount < dwRealCount)
        ret = STATUS_BUFFER_TOO_SMALL;
    else
        ret = STATUS_SUCCESS;

Quit:
    UserLeave();
    return ret;
}

static VOID FASTCALL
UserSetImeConversionKeyState(PTHREADINFO pti, DWORD dwConversion)
{
    HKL hKL;
    LANGID LangID;
    LPBYTE KeyState;
    BOOL bAlphaNumeric, bKatakana, bHiragana, bFullShape, bRoman, bCharCode;

    if (!pti->KeyboardLayout)
        return;

    hKL = pti->KeyboardLayout->hkl;
    LangID = LOWORD(hKL);
    KeyState = pti->MessageQueue->afKeyState;

    switch (PRIMARYLANGID(LangID))
    {
        case LANG_JAPANESE:
            bAlphaNumeric = !(dwConversion & IME_CMODE_NATIVE);
            bKatakana = !bAlphaNumeric && (dwConversion & IME_CMODE_KATAKANA);
            bHiragana = !bAlphaNumeric && !(dwConversion & IME_CMODE_KATAKANA);
            SET_KEY_DOWN(KeyState, VK_DBE_ALPHANUMERIC, bAlphaNumeric);
            SET_KEY_LOCKED(KeyState, VK_DBE_ALPHANUMERIC, bAlphaNumeric);
            SET_KEY_DOWN(KeyState, VK_DBE_HIRAGANA, bHiragana);
            SET_KEY_LOCKED(KeyState, VK_DBE_HIRAGANA, bHiragana);
            SET_KEY_DOWN(KeyState, VK_DBE_KATAKANA, bKatakana);
            SET_KEY_LOCKED(KeyState, VK_DBE_KATAKANA, bKatakana);

            bFullShape = (dwConversion & IME_CMODE_FULLSHAPE);
            SET_KEY_DOWN(KeyState, VK_DBE_DBCSCHAR, bFullShape);
            SET_KEY_LOCKED(KeyState, VK_DBE_DBCSCHAR, bFullShape);
            SET_KEY_DOWN(KeyState, VK_DBE_SBCSCHAR, !bFullShape);
            SET_KEY_LOCKED(KeyState, VK_DBE_SBCSCHAR, !bFullShape);

            bRoman = (dwConversion & IME_CMODE_ROMAN);
            SET_KEY_DOWN(KeyState, VK_DBE_ROMAN, bRoman);
            SET_KEY_LOCKED(KeyState, VK_DBE_ROMAN, bRoman);
            SET_KEY_DOWN(KeyState, VK_DBE_NOROMAN, !bRoman);
            SET_KEY_LOCKED(KeyState, VK_DBE_NOROMAN, !bRoman);

            bCharCode = (dwConversion & IME_CMODE_CHARCODE);
            SET_KEY_DOWN(KeyState, VK_DBE_CODEINPUT, bCharCode);
            SET_KEY_LOCKED(KeyState, VK_DBE_CODEINPUT, bCharCode);
            SET_KEY_DOWN(KeyState, VK_DBE_NOCODEINPUT, !bCharCode);
            SET_KEY_LOCKED(KeyState, VK_DBE_NOCODEINPUT, !bCharCode);
            break;

        case LANG_KOREAN:
            SET_KEY_LOCKED(KeyState, VK_HANGUL, (dwConversion & IME_CMODE_NATIVE));
            SET_KEY_LOCKED(KeyState, VK_JUNJA, (dwConversion & IME_CMODE_FULLSHAPE));
            SET_KEY_LOCKED(KeyState, VK_HANJA, (dwConversion & IME_CMODE_HANJACONVERT));
            break;

        default:
            break;
    }
}

DWORD
NTAPI
NtUserNotifyIMEStatus(
    _In_ HWND hwnd,
    _In_ BOOL fOpen,
    _In_ DWORD dwConversion)
{
    PWND pwnd;
    PTHREADINFO pti;
    HKL hKL;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        goto Quit;
    }

    pwnd = ValidateHwndNoErr(hwnd);
    if (!pwnd)
    {
        ERR("Invalid HWND %p\n", hwnd);
        goto Quit;
    }

    pti = pwnd->head.pti;
    if (!pti || !gptiForeground)
        goto Quit;
    if (pti != gptiForeground && pti->MessageQueue != gptiForeground->MessageQueue)
        goto Quit;
    if (ghIMC == pwnd->hImc && gfImeOpen == !!fOpen && gdwImeConversion == dwConversion)
        goto Quit;

    ghIMC = pwnd->hImc;
    if (ghIMC)
    {
        gfImeOpen = !!fOpen;
        gdwImeConversion = dwConversion;
        UserSetImeConversionKeyState(pti, (fOpen ? dwConversion : IME_CMODE_ALPHANUMERIC));
    }

    if (ISITHOOKED(WH_SHELL))
    {
        hKL = (pti->KeyboardLayout ? pti->KeyboardLayout->hkl : NULL);
        co_HOOK_CallHooks(WH_SHELL, HSHELL_LANGUAGE, (WPARAM)hwnd, (LPARAM)hKL);
    }

    // TODO:

Quit:
    UserLeave();
    return 0;
}

BOOL
NTAPI
NtUserDisableThreadIme(_In_ DWORD dwThreadID)
{
    PTHREADINFO pti, ptiCurrent;
    PPROCESSINFO ppi;
    BOOL ret = FALSE;

    UserEnterExclusive();

    if (!IS_IMM_MODE())
    {
        ERR("!IS_IMM_MODE()\n");
        EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        goto Quit;
    }

    ptiCurrent = GetW32ThreadInfo();

    if (dwThreadID == INVALID_THREAD_ID)
    {
        ppi = ptiCurrent->ppi;
        ppi->W32PF_flags |= W32PF_DISABLEIME;

Retry:
        for (pti = ppi->ptiList; pti; pti = pti->ptiSibling)
        {
            pti->TIF_flags |= TIF_DISABLEIME;

            if (pti->spwndDefaultIme)
            {
                co_UserDestroyWindow(pti->spwndDefaultIme);
                pti->spwndDefaultIme = NULL;
                goto Retry; /* The contents of ppi->ptiList may be changed. */
            }
        }
    }
    else
    {
        if (dwThreadID == 0)
        {
            pti = ptiCurrent;
        }
        else
        {
            pti = IntTID2PTI(UlongToHandle(dwThreadID));

            /* The thread needs to reside in the current process. */
            if (!pti || pti->ppi != ptiCurrent->ppi)
                goto Quit;
        }

        pti->TIF_flags |= TIF_DISABLEIME;

        if (pti->spwndDefaultIme)
        {
            co_UserDestroyWindow(pti->spwndDefaultIme);
            pti->spwndDefaultIme = NULL;
        }
    }

    ret = TRUE;

Quit:
    UserLeave();
    return ret;
}
/* --- pretranslation split: the remaining function groups live in ime.split2..3.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "ime.split2.c"
#include "ime.split3.c"
