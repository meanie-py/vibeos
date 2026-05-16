/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS kernel
 * PURPOSE:          Window hooks
 * PROGRAMER:        Casper S. Hornstrup <chorns@users.sourceforge.net>
 *                   James Tabor <james.tabor@reactos.org>
 *                   Rafal Harabien <rafalh@reactos.org>
  * NOTE:            Most of this code was adapted from Wine,
 *                   Copyright (C) 2002 Alexandre Julliard
 */

#include <win32k.h>
#include <safe_user_io.h>
DBG_DEFAULT_CHANNEL(UserHook);

typedef struct _HOOKPACK
{
  PHOOK pHk;
  LPARAM lParam;
  PVOID pHookStructs;
} HOOKPACK, *PHOOKPACK;

UNICODE_STRING strUahModule;
UNICODE_STRING strUahInitFunc;
PPROCESSINFO ppiUahServer;

/* PRIVATE FUNCTIONS *********************************************************/

/* Calls ClientLoadLibrary in user32 in order to load or unload a module */
BOOL
IntLoadHookModule(int iHookID, HHOOK hHook, BOOL Unload)
{
   PPROCESSINFO ppi;
   BOOL bResult;

   ppi = PsGetCurrentProcessWin32Process();

   TRACE("IntLoadHookModule. Client PID: %p\n", PsGetProcessId(ppi->peProcess));

    /* Check if this is the api hook */
    if(iHookID == WH_APIHOOK)
    {
        if(!Unload && !(ppi->W32PF_flags & W32PF_APIHOOKLOADED))
        {
            /* A callback in user mode can trigger UserLoadApiHook to be called and
               as a result IntLoadHookModule will be called recursively.
               To solve this we set the flag that means that the application has
               loaded the api hook before the callback and in case of error we remove it */
            ppi->W32PF_flags |= W32PF_APIHOOKLOADED;

            /* Call ClientLoadLibrary in user32 */
            bResult = co_IntClientLoadLibrary(&strUahModule, &strUahInitFunc, Unload, TRUE);
            TRACE("co_IntClientLoadLibrary returned %d\n", bResult );
            if (!bResult)
            {
                /* Remove the flag we set before */
                ppi->W32PF_flags &= ~W32PF_APIHOOKLOADED;
            }
            return bResult;
        }
        else if(Unload && (ppi->W32PF_flags & W32PF_APIHOOKLOADED))
        {
            /* Call ClientLoadLibrary in user32 */
            bResult = co_IntClientLoadLibrary(NULL, NULL, Unload, TRUE);
            if (bResult)
            {
                ppi->W32PF_flags &= ~W32PF_APIHOOKLOADED;
            }
            return bResult;
        }

        return TRUE;
    }

    STUB;

    return FALSE;
}

/*
IntHookModuleUnloaded:
Sends a internal message to all threads of the requested desktop
and notifies them that a global hook was destroyed
and an injected module must be unloaded.
As a result, IntLoadHookModule will be called for all the threads that
will receive the special purpose internal message.
*/
BOOL
IntHookModuleUnloaded(PDESKTOP pdesk, int iHookID, HHOOK hHook)
{
    PTHREADINFO ptiCurrent;
    PLIST_ENTRY ListEntry;
    PPROCESSINFO ppiCsr;

    TRACE("IntHookModuleUnloaded: iHookID=%d\n", iHookID);

    ppiCsr = PsGetProcessWin32Process(gpepCSRSS);

    ListEntry = pdesk->PtiList.Flink;
    while(ListEntry != &pdesk->PtiList)
    {
        ptiCurrent = CONTAINING_RECORD(ListEntry, THREADINFO, PtiLink);

        /* FIXME: Do some more security checks here */

        /* FIXME: HACK: The first check is a reactos specific hack for system threads */
        if(!PsIsSystemProcess(ptiCurrent->ppi->peProcess) &&
           ptiCurrent->ppi != ppiCsr)
        {
            if(ptiCurrent->ppi->W32PF_flags & W32PF_APIHOOKLOADED)
            {
                TRACE("IntHookModuleUnloaded: sending message to PID %p, ppi=%p\n", PsGetProcessId(ptiCurrent->ppi->peProcess), ptiCurrent->ppi);
                co_MsqSendMessageAsync( ptiCurrent,
                                        0,
                                        iHookID,
                                        TRUE,
                                        (LPARAM)hHook,
                                        NULL,
                                        0,
                                        FALSE,
                                        MSQ_INJECTMODULE);
            }
        }
        ListEntry = ListEntry->Flink;
    }

    return TRUE;
}

BOOL
FASTCALL
UserLoadApiHook(VOID)
{
    return IntLoadHookModule(WH_APIHOOK, 0, FALSE);
}

BOOL
FASTCALL
UserRegisterUserApiHook(
    PUNICODE_STRING pstrDllName,
    PUNICODE_STRING pstrFuncName)
{
    PTHREADINFO pti, ptiCurrent;
    HWND *List;
    PWND DesktopWindow, pwndCurrent;
    ULONG i;
    PPROCESSINFO ppiCsr;

    pti = PsGetCurrentThreadWin32Thread();
    ppiCsr = PsGetProcessWin32Process(gpepCSRSS);

    /* Fail if the api hook is already registered */
    if(gpsi->dwSRVIFlags & SRVINFO_APIHOOK)
    {
        return FALSE;
    }

    TRACE("UserRegisterUserApiHook. Server PID: %p\n", PsGetProcessId(pti->ppi->peProcess));

    /* Register the api hook */
    gpsi->dwSRVIFlags |= SRVINFO_APIHOOK;

    strUahModule = *pstrDllName;
    strUahInitFunc = *pstrFuncName;
    ppiUahServer = pti->ppi;

    /* Broadcast an internal message to every top level window */
    DesktopWindow = UserGetWindowObject(IntGetDesktopWindow());
    List = IntWinListChildren(DesktopWindow);

    if (List != NULL)
    {
        for (i = 0; List[i]; i++)
        {
            pwndCurrent = UserGetWindowObject(List[i]);
            if(pwndCurrent == NULL)
            {
                continue;
            }
            ptiCurrent = pwndCurrent->head.pti;

           /* FIXME: The first check is a reactos specific hack for system threads */
            if(PsIsSystemProcess(ptiCurrent->ppi->peProcess) ||
                ptiCurrent->ppi == ppiCsr)
            {
                continue;
            }

            co_MsqSendMessageAsync( ptiCurrent,
                                    0,
                                    WH_APIHOOK,
                                    FALSE,   /* Load the module */
                                    0,
                                    NULL,
                                    0,
                                    FALSE,
                                    MSQ_INJECTMODULE);
        }
        ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
    }

    return TRUE;
}

BOOL
FASTCALL
UserUnregisterUserApiHook(VOID)
{
    PTHREADINFO pti;

    pti = PsGetCurrentThreadWin32Thread();

    /* Fail if the api hook is not registered */
    if(!(gpsi->dwSRVIFlags & SRVINFO_APIHOOK))
    {
        return FALSE;
    }

    /* Only the process that registered the api hook can uregister it */
    if(ppiUahServer != PsGetCurrentProcessWin32Process())
    {
        return FALSE;
    }

    TRACE("UserUnregisterUserApiHook. Server PID: %p\n", PsGetProcessId(pti->ppi->peProcess));

    /* Unregister the api hook */
    gpsi->dwSRVIFlags &= ~SRVINFO_APIHOOK;
    ppiUahServer = NULL;
    ReleaseCapturedUnicodeString(&strUahModule, UserMode);
    ReleaseCapturedUnicodeString(&strUahInitFunc, UserMode);

    /* Notify all applications that the api hook module must be unloaded */
    return IntHookModuleUnloaded(pti->rpdesk, WH_APIHOOK, 0);
}

static
LRESULT
FASTCALL
co_IntCallLowLevelHook(PHOOK Hook,
                     INT Code,
                     WPARAM wParam,
                     LPARAM lParam)
{
    NTSTATUS Status;
    PTHREADINFO pti;
    PHOOKPACK pHP;
    INT Size = 0;
    UINT uTimeout = 300;
    BOOL Block = FALSE;
    ULONG_PTR uResult = 0;

    if (Hook->ptiHooked)
       pti = Hook->ptiHooked;
    else
       pti = Hook->head.pti;

    pHP = ExAllocatePoolWithTag(NonPagedPool, sizeof(HOOKPACK), TAG_HOOK);
    if (!pHP) return 0;

    pHP->pHk = Hook;
    pHP->lParam = lParam;
    pHP->pHookStructs = NULL;

// This prevents stack corruption from the caller.
    switch(Hook->HookId)
    {
       case WH_JOURNALPLAYBACK:
       case WH_JOURNALRECORD:
          uTimeout = 0;
          Size = sizeof(EVENTMSG);
          break;
       case WH_KEYBOARD_LL:
          Size = sizeof(KBDLLHOOKSTRUCT);
          break;
       case WH_MOUSE_LL:
          Size = sizeof(MSLLHOOKSTRUCT);
          break;
       case WH_MOUSE:
          uTimeout = 200;
          Block = TRUE;
          Size = sizeof(MOUSEHOOKSTRUCT);
          break;
       case WH_KEYBOARD:
          uTimeout = 200;
          Block = TRUE;
          break;
    }

    if (Size)
    {
       pHP->pHookStructs = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_HOOK);
       if (pHP->pHookStructs) RtlCopyMemory(pHP->pHookStructs, (PVOID)lParam, Size);
    }

    /* FIXME: Should get timeout from
     * HKEY_CURRENT_USER\Control Panel\Desktop\LowLevelHooksTimeout */
    Status = co_MsqSendMessage( pti,
                                IntToPtr(Code), // hWnd
                                Hook->HookId,   // Msg
                                wParam,
                               (LPARAM)pHP,
                                uTimeout,
                                Block,
                                MSQ_ISHOOK,
                               &uResult);
    if (!NT_SUCCESS(Status))
    {
       ERR("Error Hook Call SendMsg. %d Status: 0x%x\n", Hook->HookId, Status);
       if (pHP->pHookStructs) ExFreePoolWithTag(pHP->pHookStructs, TAG_HOOK);
       ExFreePoolWithTag(pHP, TAG_HOOK);
    }
    return NT_SUCCESS(Status) ? uResult : 0;
}


//
// Dispatch MsgQueue Hook Call processor!
//
LRESULT
APIENTRY
co_CallHook( INT HookId,
             INT Code,
             WPARAM wParam,
             LPARAM lParam)
{
    LRESULT Result = 0;
    PHOOK phk;
    PHOOKPACK pHP = (PHOOKPACK)lParam;

    phk = pHP->pHk;
    lParam = pHP->lParam;

    switch(HookId)
    {
       case WH_JOURNALPLAYBACK:
       case WH_JOURNALRECORD:
       case WH_KEYBOARD_LL:
       case WH_MOUSE_LL:
       case WH_MOUSE:
          lParam = (LPARAM)pHP->pHookStructs;
       case WH_KEYBOARD:
          break;
    }

    if (!UserObjectInDestroy(UserHMGetHandle(phk))) //// Fix CORE-10549.
    {
    /* The odds are high for this to be a Global call. */
    Result = co_IntCallHookProc( HookId,
                                 Code,
                                 wParam,
                                 lParam,
                                 phk->Proc,
                                 phk->ihmod,
                                 phk->offPfn,
                                 phk->Ansi,
                                &phk->ModuleName);
    }
    /* The odds so high, no one is waiting for the results. */
    if (pHP->pHookStructs) ExFreePoolWithTag(pHP->pHookStructs, TAG_HOOK);
    ExFreePoolWithTag(pHP, TAG_HOOK);
    return Result;
}

static
LRESULT
APIENTRY
co_HOOK_CallHookNext( PHOOK Hook,
                      INT Code,
                      WPARAM wParam,
                      LPARAM lParam)
{
    TRACE("Calling Next HOOK %d\n", Hook->HookId);

    return co_IntCallHookProc( Hook->HookId,
                               Code,
                               wParam,
                               lParam,
                               Hook->Proc,
                               Hook->ihmod,
                               Hook->offPfn,
                               Hook->Ansi,
                              &Hook->ModuleName);
}

static
LRESULT
FASTCALL
co_IntCallDebugHook(PHOOK Hook,
                  int Code,
                  WPARAM wParam,
                  LPARAM lParam,
                  BOOL Ansi)
{
    NTSTATUS Status;
    LRESULT lResult = 0;
    ULONG Size;
    DEBUGHOOKINFO Debug;
    PVOID HooklParam = NULL;
    BOOL BadChk = FALSE;

    if (lParam)
    {
        Status = SafeProbeForRead((PVOID)lParam,
                     sizeof(DEBUGHOOKINFO),
                     1);
        if (!NT_SUCCESS(Status))
        {
            BadChk = TRUE;
        }
        else
        {
            RtlCopyMemory(&Debug,
                          (PVOID)lParam,
                          sizeof(DEBUGHOOKINFO));
        }

        if (BadChk)
        {
            ERR("HOOK WH_DEBUG read from lParam ERROR!\n");
            return lResult;
        }
    }
    else
        return lResult; /* Need lParam! */

    switch (wParam)
    {
        case WH_CBT:
        {
            switch (Debug.code)
            {
                case HCBT_CLICKSKIPPED:
                    Size = sizeof(MOUSEHOOKSTRUCTEX);
                    break;

                case HCBT_MOVESIZE:
                    Size = sizeof(RECT);
                    break;

                case HCBT_ACTIVATE:
                    Size = sizeof(CBTACTIVATESTRUCT);
                    break;

                case HCBT_CREATEWND: /* Handle ANSI? */
                    Size = sizeof(CBT_CREATEWND);
                    /* What shall we do? Size += sizeof(HOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS); same as CREATESTRUCTEX */
                    break;

                default:
                    Size = sizeof(LPARAM);
            }
        }
        break;

        case WH_MOUSE_LL:
            Size = sizeof(MSLLHOOKSTRUCT);
            break;

        case WH_KEYBOARD_LL:
            Size = sizeof(KBDLLHOOKSTRUCT);
            break;

        case WH_MSGFILTER:
        case WH_SYSMSGFILTER:
        case WH_GETMESSAGE:
            Size = sizeof(MSG);
            break;

        case WH_JOURNALPLAYBACK:
        case WH_JOURNALRECORD:
            Size = sizeof(EVENTMSG);
            break;

        case WH_FOREGROUNDIDLE:
        case WH_KEYBOARD:
        case WH_SHELL:
        default:
            Size = sizeof(LPARAM);
    }

    if (Size > sizeof(LPARAM))
        HooklParam = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_HOOK);

    if (HooklParam)
    {
        Status = SafeProbeForRead((PVOID)Debug.lParam,
                     Size,
                     1);
        if (!NT_SUCCESS(Status))
        {
            BadChk = TRUE;
        }
        else
        {
            RtlCopyMemory(HooklParam,
                          (PVOID)Debug.lParam,
                          Size);
        }

        if (BadChk)
        {
            ERR("HOOK WH_DEBUG read from Debug.lParam ERROR!\n");
            ExFreePoolWithTag(HooklParam, TAG_HOOK);
            return lResult;
        }
    }

    if (HooklParam) Debug.lParam = (LPARAM)HooklParam;
    lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)&Debug);
    if (HooklParam) ExFreePoolWithTag(HooklParam, TAG_HOOK);

    return lResult;
}


static
LRESULT
APIENTRY
co_UserCallNextHookEx(PHOOK Hook,
                    int Code,
                    WPARAM wParam,
                    LPARAM lParam,
                    BOOL Ansi)
{
    LRESULT lResult = 0;
    BOOL BadChk = FALSE;
    NTSTATUS Status;

    /* Handle this one first. */
    if ((Hook->HookId == WH_MOUSE) ||
        (Hook->HookId == WH_CBT && Code == HCBT_CLICKSKIPPED))
    {
        MOUSEHOOKSTRUCTEX Mouse;
        if (lParam)
        {
            Status = SafeProbeForRead((PVOID)lParam,
                                      sizeof(MOUSEHOOKSTRUCTEX),
                                      1);
            if (!NT_SUCCESS(Status))
            {
                BadChk = TRUE;
            }
            else
            {
                RtlCopyMemory(&Mouse,
                              (PVOID)lParam,
                              sizeof(MOUSEHOOKSTRUCTEX));
            }

            if (BadChk)
            {
                ERR("HOOK WH_MOUSE read from lParam ERROR!\n");
            }
        }

        if (!BadChk)
        {
            lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)&Mouse);
        }

        return lResult;
    }

    switch(Hook->HookId)
    {
        case WH_MOUSE_LL:
        {
            MSLLHOOKSTRUCT Mouse;

            if (lParam)
            {
                Status = SafeProbeForRead((PVOID)lParam,
                                          sizeof(MSLLHOOKSTRUCT),
                                          1);
                if (!NT_SUCCESS(Status))
                {
                    BadChk = TRUE;
                }
                else
                {
                    RtlCopyMemory(&Mouse,
                                  (PVOID)lParam,
                                  sizeof(MSLLHOOKSTRUCT));
                }

                if (BadChk)
                {
                    ERR("HOOK WH_MOUSE_LL read from lParam ERROR!\n");
                }
            }

            if (!BadChk)
            {
                lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)&Mouse);
            }
            break;
        }

        case WH_KEYBOARD_LL:
        {
            KBDLLHOOKSTRUCT Keyboard;

            if (lParam)
            {
                Status = SafeProbeForRead((PVOID)lParam,
                                          sizeof(KBDLLHOOKSTRUCT),
                                          1);
                if (!NT_SUCCESS(Status))
                {
                    BadChk = TRUE;
                }
                else
                {
                    RtlCopyMemory(&Keyboard,
                                  (PVOID)lParam,
                                  sizeof(KBDLLHOOKSTRUCT));
                }

                if (BadChk)
                {
                    ERR("HOOK WH_KEYBORD_LL read from lParam ERROR!\n");
                }
            }

            if (!BadChk)
            {
                lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)&Keyboard);
            }
            break;
        }

        case WH_MSGFILTER:
        case WH_SYSMSGFILTER:
        case WH_GETMESSAGE:
        {
            MSG Msg;

            if (lParam)
            {
                Status = SafeProbeForRead((PVOID)lParam,
                                          sizeof(MSG),
                                          1);
                if (!NT_SUCCESS(Status))
                {
                    BadChk = TRUE;
                }
                else
                {
                    RtlCopyMemory(&Msg,
                                  (PVOID)lParam,
                                  sizeof(MSG));
                }

                if (BadChk)
                {
                    ERR("HOOK WH_XMESSAGEX read from lParam ERROR!\n");
                }
            }

            if (!BadChk)
            {
                lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)&Msg);

                if (lParam && (Hook->HookId == WH_GETMESSAGE))
                {
                    Status = SafeProbeForWrite((PVOID)lParam,
                                               sizeof(MSG),
                                               1);
                    if (!NT_SUCCESS(Status))
                    {
                        BadChk = TRUE;
                    }
                    else
                    {
                        RtlCopyMemory((PVOID)lParam,
                                      &Msg,
                                      sizeof(MSG));
                    }

                    if (BadChk)
                    {
                        ERR("HOOK WH_GETMESSAGE write to lParam ERROR!\n");
                    }
                }
            }
            break;
        }

        case WH_CBT:
            TRACE("HOOK WH_CBT!\n");
            switch (Code)
            {
                case HCBT_CREATEWND:
                {
                    LPCBT_CREATEWNDW pcbtcww = (LPCBT_CREATEWNDW)lParam;

                    TRACE("HOOK HCBT_CREATEWND\n");
                    if (Ansi)
                    {
                        Status = SafeProbeForRead(pcbtcww,
                                                  sizeof(CBT_CREATEWNDA),
                                                  1);
                        if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pcbtcww->lpcs,
                                                                           sizeof(CREATESTRUCTA),
                                                                           1);
                        if (NT_SUCCESS(Status)) Status = SafeProbeForRead(pcbtcww->lpcs->lpszName,
                                                                          sizeof(CHAR),
                                                                          1);

                        if (NT_SUCCESS(Status) && !IS_ATOM(pcbtcww->lpcs->lpszClass))
                        {
                            _Analysis_assume_(pcbtcww->lpcs->lpszClass != NULL);
                            Status = SafeProbeForRead(pcbtcww->lpcs->lpszClass,
                                                      sizeof(CHAR),
                                                      1);
                        }
                    }
                    else
                    {
                        Status = SafeProbeForRead(pcbtcww,
                                                  sizeof(CBT_CREATEWNDW),
                                                  1);
                        if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(pcbtcww->lpcs,
                                                                           sizeof(CREATESTRUCTW),
                                                                           1);
                        if (NT_SUCCESS(Status)) Status = SafeProbeForRead(pcbtcww->lpcs->lpszName,
                                                                          sizeof(WCHAR),
                                                                          1);

                        if (NT_SUCCESS(Status) && !IS_ATOM(pcbtcww->lpcs->lpszClass))
                        {
                            _Analysis_assume_(pcbtcww->lpcs->lpszClass != NULL);
                            Status = SafeProbeForRead(pcbtcww->lpcs->lpszClass,
                                                      sizeof(WCHAR),
                                                      1);
                        }
                    }

                    if (!NT_SUCCESS(Status))
                    {
                        BadChk = TRUE;
                    }

                    if (BadChk)
                    {
                        ERR("HOOK HCBT_CREATEWND write ERROR!\n");
                    }
                    /* The next call handles the structures. */
                    if (!BadChk && Hook->Proc)
                    {
                       lResult = co_HOOK_CallHookNext(Hook, Code, wParam, lParam);
                    }
                    break;
                }

                case HCBT_MOVESIZE:
                {
                    RECTL rt;

                    TRACE("HOOK HCBT_MOVESIZE\n");

                    if (lParam)
                    {
                        Status = SafeProbeForRead((PVOID)lParam,
                                                  sizeof(RECT),
                                                  1);
                        if (!NT_SUCCESS(Status))
                        {
                            BadChk = TRUE;
                        }
                        else
                        {
                            RtlCopyMemory(&rt,
                                          (PVOID)lParam,
                                          sizeof(RECT));
                        }

                        if (BadChk)
                        {
                            ERR("HOOK HCBT_MOVESIZE read from lParam ERROR!\n");
                        }
                    }

                    if (!BadChk)
                    {
                        lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)&rt);
                    }
                    break;
                }

                case HCBT_ACTIVATE:
                {
                    CBTACTIVATESTRUCT CbAs;

                    TRACE("HOOK HCBT_ACTIVATE\n");
                    if (lParam)
                    {
                        Status = SafeProbeForRead((PVOID)lParam,
                                                  sizeof(CBTACTIVATESTRUCT),
                                                  1);
                        if (!NT_SUCCESS(Status))
                        {
                            BadChk = TRUE;
                        }
                        else
                        {
                            RtlCopyMemory(&CbAs,
                                          (PVOID)lParam,
                                          sizeof(CBTACTIVATESTRUCT));
                        }

                        if (BadChk)
                        {
                            ERR("HOOK HCBT_ACTIVATE read from lParam ERROR!\n");
                        }
                    }

                    if (!BadChk)
                    {
                        lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)&CbAs);
                    }
                    break;
                }

                /* The rest just use default. */
                default:
                    TRACE("HOOK HCBT_ %d\n",Code);
                    lResult = co_HOOK_CallHookNext(Hook, Code, wParam, lParam);
                    break;
            }
            break;
/*
 Note WH_JOURNALPLAYBACK,
    "To have the system wait before processing the message, the return value
     must be the amount of time, in clock ticks, that the system should wait."
 */
        case WH_JOURNALPLAYBACK:
        case WH_JOURNALRECORD:
        {
            EVENTMSG EventMsg;

            if (lParam)
            {
                Status = SafeProbeForRead((PVOID)lParam,
                                          sizeof(EVENTMSG),
                                          1);
                if (!NT_SUCCESS(Status))
                {
                    BadChk = TRUE;
                }
                else
                {
                    RtlCopyMemory(&EventMsg,
                                  (PVOID)lParam,
                                  sizeof(EVENTMSG));
                }

                if (BadChk)
                {
                    ERR("HOOK WH_JOURNAL read from lParam ERROR!\n");
                }
            }

            if (!BadChk)
            {
                lResult = co_HOOK_CallHookNext(Hook, Code, wParam, (LPARAM)(lParam ? &EventMsg : NULL));

                if (lParam)
                {
                    Status = SafeProbeForWrite((PVOID)lParam,
                                               sizeof(EVENTMSG),
                                               1);
                    if (!NT_SUCCESS(Status))
                    {
                        BadChk = TRUE;
                    }
                    else
                    {
                        RtlCopyMemory((PVOID)lParam,
                                      &EventMsg,
                                      sizeof(EVENTMSG));
                    }

                    if (BadChk)
                    {
                        ERR("HOOK WH_JOURNAL write to lParam ERROR!\n");
                    }
                }
            }
            break;
        }

        case WH_DEBUG:
            lResult = co_IntCallDebugHook(Hook, Code, wParam, lParam, Ansi);
            break;

        /*
         * Default the rest like, WH_FOREGROUNDIDLE, WH_KEYBOARD and WH_SHELL.
         */
        case WH_FOREGROUNDIDLE:
        case WH_KEYBOARD:
        case WH_SHELL:
            lResult = co_HOOK_CallHookNext(Hook, Code, wParam, lParam);
            break;

        default:
            ERR("Unsupported HOOK Id -> %d\n",Hook->HookId);
            break;
    }
    return lResult;
}


PHOOK
FASTCALL
IntGetHookObject(HHOOK hHook)
{
    PHOOK Hook;

    if (!hHook)
    {
       EngSetLastError(ERROR_INVALID_HOOK_HANDLE);
       return NULL;
    }

    Hook = (PHOOK)UserGetObject(gHandleTable, hHook, TYPE_HOOK);
    if (!Hook)
    {
       EngSetLastError(ERROR_INVALID_HOOK_HANDLE);
       return NULL;
    }

    UserReferenceObject(Hook);

    return Hook;
}

static
HHOOK*
FASTCALL
IntGetGlobalHookHandles(PDESKTOP pdo, int HookId)
{
    PLIST_ENTRY pLastHead, pElem;
    unsigned i = 0;
    unsigned cHooks = 0;
    HHOOK *pList;
    PHOOK pHook;

    pLastHead = &pdo->pDeskInfo->aphkStart[HOOKID_TO_INDEX(HookId)];
    for (pElem = pLastHead->Flink; pElem != pLastHead; pElem = pElem->Flink)
      ++cHooks;

    pList = ExAllocatePoolWithTag(PagedPool, (cHooks + 1) * sizeof(HHOOK), TAG_HOOK);
    if (!pList)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }

    for (pElem = pLastHead->Flink; pElem != pLastHead; pElem = pElem->Flink)
    {
        pHook = CONTAINING_RECORD(pElem, HOOK, Chain);
        NT_ASSERT(i < cHooks);
        pList[i++] = UserHMGetHandle(pHook);
    }
    pList[i] = NULL;

    return pList;
}
/* --- pretranslation split: the remaining function groups live in hook.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "hook.split2.c"
