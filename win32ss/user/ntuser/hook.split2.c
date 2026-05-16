/* hook.split2.c — #include fragment of hook.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original hook.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


/* Find the next hook in the chain  */
PHOOK
FASTCALL
IntGetNextHook(PHOOK Hook)
{
    int HookId = Hook->HookId;
    PLIST_ENTRY pLastHead, pElem;
    PTHREADINFO pti;

    if (Hook->ptiHooked)
    {
       pti = Hook->ptiHooked;
       pLastHead = &pti->aphkStart[HOOKID_TO_INDEX(HookId)];
    }
    else
    {
       pti = PsGetCurrentThreadWin32Thread();
       pLastHead = &pti->rpdesk->pDeskInfo->aphkStart[HOOKID_TO_INDEX(HookId)];
    }

    pElem = Hook->Chain.Flink;
    if (pElem != pLastHead)
       return CONTAINING_RECORD(pElem, HOOK, Chain);
    return NULL;
}

/* Free a hook, removing it from its chain */
static
VOID
FASTCALL
IntFreeHook(PHOOK Hook)
{
    RemoveEntryList(&Hook->Chain);
    if (Hook->ModuleName.Buffer)
    {
       ExFreePoolWithTag(Hook->ModuleName.Buffer, TAG_HOOK);
       Hook->ModuleName.Buffer = NULL;
    }
    /* Close handle */
    UserDeleteObject(UserHMGetHandle(Hook), TYPE_HOOK);
}

/* Remove a hook, freeing it from the chain */
BOOLEAN
IntRemoveHook(PVOID Object)
{
    INT HookId;
    PTHREADINFO ptiHook, pti;
    PDESKTOP pdo;
    PHOOK Hook = Object;

    NT_ASSERT(UserIsEnteredExclusive());

    HookId = Hook->HookId;
    pti = PsGetCurrentThreadWin32Thread();

    if (Hook->ptiHooked) // Local
    {
        ptiHook = Hook->ptiHooked;

        IntFreeHook(Hook);

        if (IsListEmpty(&ptiHook->aphkStart[HOOKID_TO_INDEX(HookId)]))
        {
            BOOL bOtherProcess;
            KAPC_STATE ApcState;

            ptiHook->fsHooks &= ~HOOKID_TO_FLAG(HookId);
            bOtherProcess = (ptiHook->ppi != pti->ppi);

            if (bOtherProcess)
                KeStackAttachProcess(&ptiHook->ppi->peProcess->Pcb, &ApcState);

            _SEH2_TRY
            {
                ptiHook->pClientInfo->fsHooks = ptiHook->fsHooks;
            }
            _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            {
                /* Do nothing */
                (void)0;
            }
            _SEH2_END;

            if (bOtherProcess)
                KeUnstackDetachProcess(&ApcState);
       }
    }
    else // Global
    {
        IntFreeHook(Hook);

        pdo = IntGetActiveDesktop();

        if (pdo &&
             pdo->pDeskInfo &&
             IsListEmpty(&pdo->pDeskInfo->aphkStart[HOOKID_TO_INDEX(HookId)]))
        {
            pdo->pDeskInfo->fsHooks &= ~HOOKID_TO_FLAG(HookId);
        }
    }

    return TRUE;
}

/*
  Win32k Kernel Space Hook Caller.
 */
LRESULT
APIENTRY
co_HOOK_CallHooks( INT HookId,
                   INT Code,
                   WPARAM wParam,
                   LPARAM lParam)
{
    PHOOK Hook, SaveHook;
    PTHREADINFO pti;
    PCLIENTINFO ClientInfo;
    PLIST_ENTRY pLastHead;
    PDESKTOP pdo;
    BOOL Local = FALSE, Global = FALSE;
    LRESULT Result = 0;
    USER_REFERENCE_ENTRY Ref;

    ASSERT(WH_MINHOOK <= HookId && HookId <= WH_MAXHOOK);

    pti = PsGetCurrentThreadWin32Thread();
    if (!pti || !pti->rpdesk || !pti->rpdesk->pDeskInfo)
    {
       pdo = IntGetActiveDesktop();
    /* If KeyboardThread|MouseThread|(RawInputThread or RIT) aka system threads,
       pti->fsHooks most likely, is zero. So process KbT & MsT to "send" the message.
     */
       if ( !pti || !pdo || (!(HookId == WH_KEYBOARD_LL) && !(HookId == WH_MOUSE_LL)) )
       {
          TRACE("No PDO %d\n", HookId);
          goto Exit;
       }
    }
    else
    {
       pdo = pti->rpdesk;
    }

    if ( pti->TIF_flags & (TIF_INCLEANUP|TIF_DISABLEHOOKS))
    {
       TRACE("Hook Thread dead %d\n", HookId);
       goto Exit;
    }

    if ( ISITHOOKED(HookId) )
    {
       TRACE("Local Hooker %d\n", HookId);
       Local = TRUE;
    }

    if ( pdo->pDeskInfo->fsHooks & HOOKID_TO_FLAG(HookId) )
    {
       TRACE("Global Hooker %d\n", HookId);
       Global = TRUE;
    }

    if ( !Local && !Global ) goto Exit; // No work!

    Hook = NULL;

    /* SetWindowHookEx sorts out the Thread issue by placing the Hook to
       the correct Thread if not NULL.
     */
    if ( Local )
    {
       pLastHead = &pti->aphkStart[HOOKID_TO_INDEX(HookId)];
       if (IsListEmpty(pLastHead))
       {
          ERR("No Local Hook Found!\n");
          goto Exit;
       }

       Hook = CONTAINING_RECORD(pLastHead->Flink, HOOK, Chain);
       ObReferenceObject(pti->pEThread);
       IntReferenceThreadInfo(pti);
       UserRefObjectCo(Hook, &Ref);

       ClientInfo = pti->pClientInfo;
       SaveHook = pti->sphkCurrent;
       /* Note: Setting pti->sphkCurrent will also lock the next hook to this
        *       hook ID. So, the CallNextHookEx will only call to that hook ID
        *       chain anyway. For Thread Hooks....
        */

       /* Load it for the next call. */
       pti->sphkCurrent = Hook;
       Hook->phkNext = IntGetNextHook(Hook);
       if (ClientInfo)
       {
          _SEH2_TRY
          {
             ClientInfo->phkCurrent = Hook;
          }
          _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
          {
             ClientInfo = NULL; // Don't bother next run.
          }
          _SEH2_END;
       }
       Result = co_IntCallHookProc( HookId,
                                    Code,
                                    wParam,
                                    lParam,
                                    Hook->Proc,
                                    Hook->ihmod,
                                    Hook->offPfn,
                                    Hook->Ansi,
                                   &Hook->ModuleName);
       if (ClientInfo)
       {
          _SEH2_TRY
          {
             ClientInfo->phkCurrent = SaveHook;
          }
          _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
          {
              /* Do nothing */
              (void)0;
          }
          _SEH2_END;
       }
       pti->sphkCurrent = SaveHook;
       Hook->phkNext = NULL;
       UserDerefObjectCo(Hook);
       IntDereferenceThreadInfo(pti);
       ObDereferenceObject(pti->pEThread);
    }

    if ( Global )
    {
       PTHREADINFO ptiHook;
       HHOOK *pHookHandles;
       unsigned i;

       /* Keep hooks in array because hooks can be destroyed in user world */
       pHookHandles = IntGetGlobalHookHandles(pdo, HookId);
       if(!pHookHandles)
          goto Exit;

      /* Performance goes down the drain. If more hooks are associated to this
       * hook ID, this will have to post to each of the thread message queues
       * or make a direct call.
       */
       for(i = 0; pHookHandles[i]; ++i)
       {
          Hook = (PHOOK)UserGetObject(gHandleTable, pHookHandles[i], TYPE_HOOK);
          if(!Hook)
          {
              ERR("Invalid hook!\n");
              continue;
          }

         /* Hook->Thread is null, we hax around this with Hook->head.pti. */
          ptiHook = Hook->head.pti;

          if ( (pti->TIF_flags & TIF_DISABLEHOOKS) || (ptiHook->TIF_flags & TIF_INCLEANUP))
          {
             TRACE("Next Hook %p, %p\n", ptiHook->rpdesk, pdo);
             continue;
          }
          UserRefObjectCo(Hook, &Ref);

          if (ptiHook != pti )
          {
                                                  // Block | TimeOut
             if ( HookId == WH_JOURNALPLAYBACK || //   1   |    0
                  HookId == WH_JOURNALRECORD   || //   1   |    0
                  HookId == WH_KEYBOARD        || //   1   |   200
                  HookId == WH_MOUSE           || //   1   |   200
                  HookId == WH_KEYBOARD_LL     || //   0   |   300
                  HookId == WH_MOUSE_LL )         //   0   |   300
             {
                TRACE("\nGlobal Hook posting to another Thread! %d\n",HookId );
                Result = co_IntCallLowLevelHook(Hook, Code, wParam, lParam);
             }
             else if (ptiHook->ppi == pti->ppi)
             {
                TRACE("\nGlobal Hook calling to another Thread! %d\n",HookId );
                ObReferenceObject(ptiHook->pEThread);
                IntReferenceThreadInfo(ptiHook);
                Result = co_IntCallHookProc( HookId,
                                             Code,
                                             wParam,
                                             lParam,
                                             Hook->Proc,
                                             Hook->ihmod,
                                             Hook->offPfn,
                                             Hook->Ansi,
                                            &Hook->ModuleName);
                IntDereferenceThreadInfo(ptiHook);
                ObDereferenceObject(ptiHook->pEThread);
             }
          }
          else
          { /* Make the direct call. */
             TRACE("Global going Local Hook calling to Thread! %d\n",HookId );
             ObReferenceObject(pti->pEThread);
             IntReferenceThreadInfo(pti);
             Result = co_IntCallHookProc( HookId,
                                          Code,
                                          wParam,
                                          lParam,
                                          Hook->Proc,
                                          Hook->ihmod,
                                          Hook->offPfn,
                                          Hook->Ansi,
                                         &Hook->ModuleName);
             IntDereferenceThreadInfo(pti);
             ObDereferenceObject(pti->pEThread);
          }
          UserDerefObjectCo(Hook);
       }
       ExFreePoolWithTag(pHookHandles, TAG_HOOK);
       TRACE("Ret: Global HookId %d Result 0x%x\n", HookId,Result);
    }
Exit:
    return Result;
}

BOOL
FASTCALL
IntUnhookWindowsHook(int HookId, HOOKPROC pfnFilterProc)
{
    PHOOK Hook;
    PLIST_ENTRY pLastHead, pElement;
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

    if (HookId < WH_MINHOOK || WH_MAXHOOK < HookId )
    {
       EngSetLastError(ERROR_INVALID_HOOK_FILTER);
       return FALSE;
    }

    if (pti->fsHooks)
    {
       pLastHead = &pti->aphkStart[HOOKID_TO_INDEX(HookId)];

       pElement = pLastHead->Flink;
       while (pElement != pLastHead)
       {
          Hook = CONTAINING_RECORD(pElement, HOOK, Chain);

          /* Get the next element now, we might free the hook in what follows */
          pElement = Hook->Chain.Flink;

          if (Hook->Proc == pfnFilterProc)
          {
             if (Hook->head.pti == pti)
             {
                IntRemoveHook(Hook);
                return TRUE;
             }
             else
             {
                EngSetLastError(ERROR_ACCESS_DENIED);
                return FALSE;
             }
          }
       }
    }
    return FALSE;
}

/*
 *  Support for compatibility only? Global hooks are processed in kernel space.
 *  This is very thread specific! Never seeing applications with more than one
 *  hook per thread installed. Most of the applications are Global hookers and
 *  associated with just one hook Id. Maybe it's for diagnostic testing or a
 *  throw back to 3.11?
 */
LRESULT
APIENTRY
NtUserCallNextHookEx( int Code,
                      WPARAM wParam,
                      LPARAM lParam,
                      BOOL Ansi)
{
    PTHREADINFO pti;
    PHOOK HookObj, NextObj;
    PCLIENTINFO ClientInfo;
    LRESULT lResult = 0;

    TRACE("Enter NtUserCallNextHookEx\n");
    UserEnterExclusive();

    pti = GetW32ThreadInfo();

    HookObj = pti->sphkCurrent;

    if (!HookObj)
       goto Exit; // Return 0

    NextObj = HookObj->phkNext;

    pti->sphkCurrent = NextObj;
    ClientInfo = pti->pClientInfo;
    _SEH2_TRY
    {
       ClientInfo->phkCurrent = NextObj;
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
       ClientInfo = NULL;
    }
    _SEH2_END;

    /* Now in List run down. */
    if (ClientInfo && NextObj)
    {
       NextObj->phkNext = IntGetNextHook(NextObj);
       lResult = co_UserCallNextHookEx( NextObj, Code, wParam, lParam, NextObj->Ansi);
    }

Exit:
    TRACE("Leave NtUserCallNextHookEx, ret=%i\n", lResult);
    UserLeave();
    return lResult;
}

HHOOK
APIENTRY
NtUserSetWindowsHookAW( int idHook,
                        HOOKPROC lpfn,
                        BOOL Ansi)
{
    DWORD ThreadId;
    UNICODE_STRING USModuleName;

    RtlInitUnicodeString(&USModuleName, NULL);
    ThreadId = PtrToUint(NtCurrentTeb()->ClientId.UniqueThread);

    return NtUserSetWindowsHookEx( NULL,
                                  &USModuleName,
                                   ThreadId,
                                   idHook,
                                   lpfn,
                                   Ansi);
}

HHOOK
APIENTRY
NtUserSetWindowsHookEx( HINSTANCE Mod,
                        PUNICODE_STRING UnsafeModuleName,
                        DWORD ThreadId,
                        int HookId,
                        HOOKPROC HookProc,
                        BOOL Ansi)
{
    PWINSTATION_OBJECT WinStaObj;
    PHOOK Hook = NULL;
    UNICODE_STRING ModuleName;
    NTSTATUS Status;
    HHOOK Handle;
    PTHREADINFO pti, ptiHook = NULL;
    HHOOK Ret = NULL;

    TRACE("Enter NtUserSetWindowsHookEx\n");
    UserEnterExclusive();

    pti = PsGetCurrentThreadWin32Thread();

    if (HookId < WH_MINHOOK || WH_MAXHOOK < HookId )
    {
        EngSetLastError(ERROR_INVALID_HOOK_FILTER);
        goto Cleanup; // Return NULL
    }

    if (!HookProc)
    {
        EngSetLastError(ERROR_INVALID_FILTER_PROC);
        goto Cleanup; // Return NULL
    }

    if (ThreadId)  /* thread-local hook */
    {
       if ( HookId == WH_JOURNALRECORD ||
            HookId == WH_JOURNALPLAYBACK ||
            HookId == WH_KEYBOARD_LL ||
            HookId == WH_MOUSE_LL ||
            HookId == WH_SYSMSGFILTER)
       {
           TRACE("Local hook installing Global HookId: %d\n",HookId);
           /* these can only be global */
           EngSetLastError(ERROR_GLOBAL_ONLY_HOOK);
           goto Cleanup; // Return NULL
       }

       if ( !(ptiHook = IntTID2PTI( UlongToHandle(ThreadId) )))
       {
          ERR("Invalid thread id 0x%x\n", ThreadId);
          EngSetLastError(ERROR_INVALID_PARAMETER);
          goto Cleanup; // Return NULL
       }

       if ( ptiHook->rpdesk != pti->rpdesk) // gptiCurrent->rpdesk)
       {
          ERR("Local hook wrong desktop HookId: %d\n",HookId);
          EngSetLastError(ERROR_ACCESS_DENIED);
          goto Cleanup; // Return NULL
       }

       if (ptiHook->ppi != pti->ppi)
       {
          if ( !Mod &&
              (HookId == WH_GETMESSAGE ||
               HookId == WH_CALLWNDPROC ||
               HookId == WH_CBT ||
               HookId == WH_HARDWARE ||
               HookId == WH_DEBUG ||
               HookId == WH_SHELL ||
               HookId == WH_FOREGROUNDIDLE ||
               HookId == WH_CALLWNDPROCRET) )
          {
             ERR("Local hook needs hMod HookId: %d\n",HookId);
             EngSetLastError(ERROR_HOOK_NEEDS_HMOD);
             goto Cleanup; // Return NULL
          }

          if ( (ptiHook->TIF_flags & (TIF_CSRSSTHREAD|TIF_SYSTEMTHREAD)) &&
               (HookId == WH_GETMESSAGE ||
                HookId == WH_CALLWNDPROC ||
                HookId == WH_CBT ||
                HookId == WH_HARDWARE ||
                HookId == WH_DEBUG ||
                HookId == WH_SHELL ||
                HookId == WH_FOREGROUNDIDLE ||
                HookId == WH_CALLWNDPROCRET) )
          {
             EngSetLastError(ERROR_HOOK_TYPE_NOT_ALLOWED);
             goto Cleanup; // Return NULL
          }
       }
    }
    else  /* System-global hook */
    {
       ptiHook = pti; // gptiCurrent;
       if ( !Mod &&
            (HookId == WH_GETMESSAGE ||
             HookId == WH_CALLWNDPROC ||
             HookId == WH_CBT ||
             HookId == WH_SYSMSGFILTER ||
             HookId == WH_HARDWARE ||
             HookId == WH_DEBUG ||
             HookId == WH_SHELL ||
             HookId == WH_FOREGROUNDIDLE ||
             HookId == WH_CALLWNDPROCRET) )
       {
          ERR("Global hook needs hMod HookId: %d\n",HookId);
          EngSetLastError(ERROR_HOOK_NEEDS_HMOD);
          goto Cleanup; // Return NULL
       }
    }

    Status = IntValidateWindowStationHandle( PsGetCurrentProcess()->Win32WindowStation,
                                             UserMode,
                                             0,
                                            &WinStaObj,
                                             0);

    if (!NT_SUCCESS(Status))
    {
       SetLastNtError(Status);
       goto Cleanup; // Return NULL
    }
    ObDereferenceObject(WinStaObj);

    Hook = UserCreateObject(gHandleTable, NULL, ptiHook, (PHANDLE)&Handle, TYPE_HOOK, sizeof(HOOK));

    if (!Hook)
    {
       goto Cleanup; // Return NULL
    }

    Hook->ihmod   = (INT_PTR)Mod; // Module Index from atom table, Do this for now.
    Hook->HookId  = HookId;
    Hook->rpdesk  = ptiHook->rpdesk;
    Hook->phkNext = NULL; /* Dont use as a chain! Use link lists for chaining. */
    Hook->Proc    = HookProc;
    Hook->Ansi    = Ansi;

    TRACE("Set Hook Desk %p DeskInfo %p Handle Desk %p\n", pti->rpdesk, pti->pDeskInfo, Hook->head.rpdesk);

    if (ThreadId)  /* Thread-local hook */
    {
       InsertHeadList(&ptiHook->aphkStart[HOOKID_TO_INDEX(HookId)], &Hook->Chain);
       ptiHook->sphkCurrent = NULL;
       Hook->ptiHooked = ptiHook;
       ptiHook->fsHooks |= HOOKID_TO_FLAG(HookId);

       if (ptiHook->pClientInfo)
       {
          if ( ptiHook->ppi == pti->ppi) /* gptiCurrent->ppi) */
          {
             _SEH2_TRY
             {
                ptiHook->pClientInfo->fsHooks = ptiHook->fsHooks;
                ptiHook->pClientInfo->phkCurrent = NULL;
             }
             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
             {
                ERR("Problem writing to Local ClientInfo!\n");
             }
             _SEH2_END;
          }
          else
          {
             KAPC_STATE ApcState;

             KeStackAttachProcess(&ptiHook->ppi->peProcess->Pcb, &ApcState);
             _SEH2_TRY
             {
                ptiHook->pClientInfo->fsHooks = ptiHook->fsHooks;
                ptiHook->pClientInfo->phkCurrent = NULL;
             }
             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
             {
                ERR("Problem writing to Remote ClientInfo!\n");
             }
             _SEH2_END;
             KeUnstackDetachProcess(&ApcState);
          }
       }
    }
    else
    {
       InsertHeadList(&ptiHook->rpdesk->pDeskInfo->aphkStart[HOOKID_TO_INDEX(HookId)], &Hook->Chain);
       Hook->ptiHooked = NULL;
       //gptiCurrent->pDeskInfo->fsHooks |= HOOKID_TO_FLAG(HookId);
       ptiHook->rpdesk->pDeskInfo->fsHooks |= HOOKID_TO_FLAG(HookId);
       ptiHook->sphkCurrent = NULL;
       ptiHook->pClientInfo->phkCurrent = NULL;
    }

    RtlInitUnicodeString(&Hook->ModuleName, NULL);

    if (Mod)
    {
       Status = MmCopyFromCaller(&ModuleName,
                                  UnsafeModuleName,
                                  sizeof(UNICODE_STRING));
       if (!NT_SUCCESS(Status))
       {
          IntRemoveHook(Hook);
          SetLastNtError(Status);
          goto Cleanup; // Return NULL
       }

       Hook->ModuleName.Buffer = ExAllocatePoolWithTag( PagedPool,
                                                        ModuleName.MaximumLength,
                                                        TAG_HOOK);
       if (NULL == Hook->ModuleName.Buffer)
       {
          IntRemoveHook(Hook);
          EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
          goto Cleanup; // Return NULL
       }

       Hook->ModuleName.MaximumLength = ModuleName.MaximumLength;
       Status = MmCopyFromCaller( Hook->ModuleName.Buffer,
                                  ModuleName.Buffer,
                                  ModuleName.MaximumLength);
       if (!NT_SUCCESS(Status))
       {
          ExFreePoolWithTag(Hook->ModuleName.Buffer, TAG_HOOK);
          Hook->ModuleName.Buffer = NULL;
          IntRemoveHook(Hook);
          SetLastNtError(Status);
          goto Cleanup; // Return NULL
       }

       Hook->ModuleName.Length = ModuleName.Length;
       //// FIXME: Need to load from user32 to verify hMod before calling hook with hMod set!!!!
       //// Mod + offPfn == new HookProc Justin Case module is from another process.
       FIXME("NtUserSetWindowsHookEx Setting process hMod instance addressing.\n");
       /* Make proc relative to the module base */
       Hook->offPfn = (ULONG_PTR)((char *)HookProc - (char *)Mod);
    }
    else
       Hook->offPfn = 0;

    TRACE("Installing: HookId %d Global %s\n", HookId, !ThreadId ? "TRUE" : "FALSE");
    Ret = Handle;

Cleanup:
    if (Hook)
        UserDereferenceObject(Hook);
    TRACE("Leave NtUserSetWindowsHookEx, ret=%p\n", Ret);
    UserLeave();
    return Ret;
}

BOOL
APIENTRY
NtUserUnhookWindowsHookEx(HHOOK Hook)
{
    PHOOK HookObj;
    BOOL Ret = FALSE;

    TRACE("Enter NtUserUnhookWindowsHookEx\n");
    UserEnterExclusive();

    if (!(HookObj = IntGetHookObject(Hook)))
    {
        ERR("Invalid handle passed to NtUserUnhookWindowsHookEx\n");
        /* SetLastNtError(Status); */
        goto Exit; // Return FALSE
    }

    ASSERT(Hook == UserHMGetHandle(HookObj));

    IntRemoveHook(HookObj);

    UserDereferenceObject(HookObj);

    Ret = TRUE;

Exit:
    TRACE("Leave NtUserUnhookWindowsHookEx, ret=%i\n", Ret);
    UserLeave();
    return Ret;
}

BOOL
APIENTRY
NtUserRegisterUserApiHook(
    PUNICODE_STRING m_dllname1,
    PUNICODE_STRING m_funname1,
    DWORD dwUnknown3,
    DWORD dwUnknown4)
{
    BOOL ret;
    UNICODE_STRING strDllNameSafe;
    UNICODE_STRING strFuncNameSafe;
    NTSTATUS Status;

    /* Probe and capture parameters */
    Status = ProbeAndCaptureUnicodeString(&strDllNameSafe, UserMode, m_dllname1);
    if(!NT_SUCCESS(Status))
    {
        EngSetLastError(RtlNtStatusToDosError(Status));
        return FALSE;
    }

    Status = ProbeAndCaptureUnicodeString(&strFuncNameSafe, UserMode, m_funname1);
    if(!NT_SUCCESS(Status))
    {
        ReleaseCapturedUnicodeString(&strDllNameSafe, UserMode);
        EngSetLastError(RtlNtStatusToDosError(Status));
        return FALSE;
    }

    UserEnterExclusive();

    /* Call internal function */
    ret = UserRegisterUserApiHook(&strDllNameSafe, &strFuncNameSafe);

    UserLeave();

    /* Cleanup only in case of failure */
    if(ret == FALSE)
    {
        ReleaseCapturedUnicodeString(&strDllNameSafe, UserMode);
        ReleaseCapturedUnicodeString(&strFuncNameSafe, UserMode);
    }

    return ret;
}

BOOL
APIENTRY
NtUserUnregisterUserApiHook(VOID)
{
    BOOL ret;

    UserEnterExclusive();
    ret = UserUnregisterUserApiHook();
    UserLeave();

    return ret;
}

/* EOF */
