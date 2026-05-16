/* message.split3.c — #include fragment of message.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original message.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */



DWORD APIENTRY
IntGetQueueStatus(DWORD Changes)
{
    PTHREADINFO pti;
    DWORD Result;

    pti = PsGetCurrentThreadWin32Thread();
// wine:
    Changes &= (QS_ALLINPUT|QS_ALLPOSTMESSAGE|QS_SMRESULT);

    /* High word, types of messages currently in the queue.
       Low  word, types of messages that have been added to the queue and that
                  are still in the queue
     */
    Result = MAKELONG(pti->pcti->fsChangeBits & Changes, pti->pcti->fsWakeBits & Changes);

    pti->pcti->fsChangeBits &= ~Changes;

    return Result;
}

BOOL APIENTRY
IntInitMessagePumpHook(VOID)
{
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

    if (pti->pcti)
    {
        pti->pcti->dwcPumpHook++;
        return TRUE;
    }
    return FALSE;
}

BOOL APIENTRY
IntUninitMessagePumpHook(VOID)
{
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

    if (pti->pcti)
    {
        if (pti->pcti->dwcPumpHook <= 0)
        {
            return FALSE;
        }
        pti->pcti->dwcPumpHook--;
        return TRUE;
    }
    return FALSE;
}

BOOL FASTCALL
IntCallMsgFilter(LPMSG lpmsg, INT code)
{
    BOOL Ret;

    Ret = co_HOOK_CallHooks(WH_SYSMSGFILTER, code, 0, (LPARAM)lpmsg);
    if (!Ret)
        Ret = co_HOOK_CallHooks(WH_MSGFILTER, code, 0, (LPARAM)lpmsg);

    return Ret;
}

/** Functions ******************************************************************/

BOOL
APIENTRY
NtUserDragDetect(
   HWND hWnd,
   POINT pt) // Just like the User call.
{
    MSG msg;
    RECT rect;
    ULONG wDragWidth, wDragHeight;
    BOOL Ret = FALSE;

    TRACE("Enter NtUserDragDetect(%p)\n", hWnd);
    UserEnterExclusive();

    wDragWidth = UserGetSystemMetrics(SM_CXDRAG);
    wDragHeight= UserGetSystemMetrics(SM_CYDRAG);

    rect.left = pt.x - wDragWidth;
    rect.right = pt.x + wDragWidth;

    rect.top = pt.y - wDragHeight;
    rect.bottom = pt.y + wDragHeight;

    co_UserSetCapture(hWnd);

    for (;;)
    {
        while (co_IntGetPeekMessage( &msg, 0, WM_MOUSEFIRST, WM_MOUSELAST, PM_REMOVE, FALSE ) ||
               co_IntGetPeekMessage( &msg, 0, WM_QUEUESYNC,  WM_QUEUESYNC, PM_REMOVE, FALSE ) ||
               co_IntGetPeekMessage( &msg, 0, WM_KEYFIRST,   WM_KEYLAST,   PM_REMOVE, FALSE ) )
        {
            if ( msg.message == WM_LBUTTONUP )
            {
                co_UserSetCapture(NULL);
                goto Exit; // Return FALSE
            }
            if ( msg.message == WM_MOUSEMOVE )
            {
                POINT tmp;
                tmp.x = (short)LOWORD(msg.lParam);
                tmp.y = (short)HIWORD(msg.lParam);
                if( !RECTL_bPointInRect( &rect, tmp.x, tmp.y ) )
                {
                    co_UserSetCapture(NULL);
                    Ret = TRUE;
                    goto Exit;
                }
            }
            if ( msg.message == WM_KEYDOWN )
            {
                if ( msg.wParam == VK_ESCAPE )
                {
                   co_UserSetCapture(NULL);
                   Ret = TRUE;
                   goto Exit;
                }
            }
            if ( msg.message == WM_QUEUESYNC )
            {
                co_HOOK_CallHooks( WH_CBT, HCBT_QS, 0, 0 );
            }
        }
        co_IntWaitMessage(NULL, 0, 0);
    }

Exit:
   TRACE("Leave NtUserDragDetect, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

BOOL APIENTRY
NtUserPostMessage(HWND hWnd,
                  UINT Msg,
                  WPARAM wParam,
                  LPARAM lParam)
{
    BOOL ret;

    UserEnterExclusive();

    ret = UserPostMessage(hWnd, Msg, wParam, lParam);

    UserLeave();

    return ret;
}

BOOL APIENTRY
NtUserPostThreadMessage(DWORD idThread,
                        UINT Msg,
                        WPARAM wParam,
                        LPARAM lParam)
{
    BOOL ret = FALSE;
    PETHREAD peThread;
    PTHREADINFO pThread;
    NTSTATUS Status;

    UserEnterExclusive();

    Status = PsLookupThreadByThreadId(UlongToHandle(idThread), &peThread);

    if ( Status == STATUS_SUCCESS )
    {
        pThread = (PTHREADINFO)peThread->Tcb.Win32Thread;
        if( !pThread ||
            !pThread->MessageQueue ||
            (pThread->TIF_flags & TIF_INCLEANUP))
        {
            ObDereferenceObject( peThread );
            goto exit;
        }
        ret = UserPostThreadMessage( pThread, Msg, wParam, lParam);
        ObDereferenceObject( peThread );
    }
    else
    {
        SetLastNtError( Status );
    }
exit:
    UserLeave();
    return ret;
}

BOOL APIENTRY
NtUserWaitMessage(VOID)
{
    BOOL ret;

    UserEnterExclusive();
    TRACE("NtUserWaitMessage Enter\n");
    ret = co_IntWaitMessage(NULL, 0, 0);
    TRACE("NtUserWaitMessage Leave\n");
    UserLeave();

    return ret;
}

BOOL APIENTRY
NtUserGetMessage(PMSG pMsg,
                  HWND hWnd,
                  UINT MsgFilterMin,
                  UINT MsgFilterMax )
{
    MSG Msg;
    BOOL Ret;
    NTSTATUS Status;

    if ( (MsgFilterMin|MsgFilterMax) & ~WM_MAXIMUM )
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    UserEnterExclusive();

    RtlZeroMemory(&Msg, sizeof(MSG));

    Ret = co_IntGetPeekMessage(&Msg, hWnd, MsgFilterMin, MsgFilterMax, PM_REMOVE, TRUE);

    UserLeave();

    if (Ret)
    {
        Status = SafeProbeForWrite(pMsg, sizeof(MSG), 1);
        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
            Ret = FALSE;
        }
        else
        {
            RtlCopyMemory(pMsg, &Msg, sizeof(MSG));
        }
    }

    if ((INT)Ret != -1)
       Ret = Ret ? (WM_QUIT != pMsg->message) : FALSE;

    return Ret;
}


BOOL APIENTRY
NtUserPeekMessage( PMSG pMsg,
                  HWND hWnd,
                  UINT MsgFilterMin,
                  UINT MsgFilterMax,
                  UINT RemoveMsg)
{
    MSG Msg;
    BOOL Ret;
    NTSTATUS Status;

    if ( RemoveMsg & PM_BADMSGFLAGS )
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return FALSE;
    }

    UserEnterExclusive();

    RtlZeroMemory(&Msg, sizeof(MSG));

    Ret = co_IntGetPeekMessage(&Msg, hWnd, MsgFilterMin, MsgFilterMax, RemoveMsg, FALSE);

    UserLeave();

    if (Ret)
    {
        Status = SafeProbeForWrite(pMsg, sizeof(MSG), 1);
        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
            Ret = FALSE;
        }
        else
        {
            RtlCopyMemory(pMsg, &Msg, sizeof(MSG));
        }
    }

    return Ret;
}


BOOL APIENTRY
NtUserCallMsgFilter( LPMSG lpmsg, INT code)
{
    BOOL Ret = FALSE;
    MSG Msg;
    NTSTATUS Status;

    {
        NTSTATUS __SehStatus = SafeProbeForRead(lpmsg, sizeof(MSG), 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            return FALSE;
        }
    }
    RtlCopyMemory( &Msg, lpmsg, sizeof(MSG));

    UserEnterExclusive();
    Ret = IntCallMsgFilter(&Msg, code);
    UserLeave();

    Status = SafeProbeForWrite(lpmsg, sizeof(MSG), 1);
    if (!NT_SUCCESS(Status))
    {
        Ret = FALSE;
    }
    else
    {
        RtlCopyMemory(lpmsg, &Msg, sizeof(MSG));
    }

    return Ret;
}


LRESULT APIENTRY
NtUserDispatchMessage(PMSG UnsafeMsgInfo)
{
    LRESULT Res = 0;
    MSG SafeMsg;

    {
        NTSTATUS __SehStatus = SafeProbeForRead(UnsafeMsgInfo, sizeof(MSG), 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            SetLastNtError(__SehStatus);
            return FALSE;
        }
    }
    RtlCopyMemory(&SafeMsg, UnsafeMsgInfo, sizeof(MSG));

    UserEnterExclusive();

    Res = IntDispatchMessage(&SafeMsg);

    UserLeave();
    return Res;
}

BOOL APIENTRY
NtUserTranslateMessage(LPMSG lpMsg, UINT flags)
{
    MSG SafeMsg;
    BOOL Ret;
    PWND pWnd;

    {
        NTSTATUS __SehStatus = SafeProbeForRead(lpMsg, sizeof(MSG), 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            SetLastNtError(__SehStatus);
            return FALSE;
        }
    }
    RtlCopyMemory(&SafeMsg, lpMsg, sizeof(MSG));

    UserEnterExclusive();
    pWnd = UserGetWindowObject(SafeMsg.hwnd);
    if (pWnd) // Must have a window!
    {
       Ret = IntTranslateKbdMessage(&SafeMsg, flags);
    }
    else
    {
        TRACE("No Window for Translate. hwnd 0x%p Msg %u\n", SafeMsg.hwnd, SafeMsg.message);
        Ret = FALSE;
    }
    UserLeave();

    return Ret;
}

LRESULT APIENTRY ScrollBarWndProc(HWND Wnd, UINT Msg, WPARAM wParam, LPARAM lParam);

BOOL APIENTRY
NtUserMessageCall( HWND hWnd,
                   UINT Msg,
                   WPARAM wParam,
                   LPARAM lParam,
                   ULONG_PTR ResultInfo,
                   DWORD dwType, // fnID?
                   BOOL Ansi)
{
    LRESULT lResult = 0;
    BOOL Ret = FALSE;
    PWND Window = NULL;
    USER_REFERENCE_ENTRY Ref;
    NTSTATUS Status;

    UserEnterExclusive();

    switch(dwType)
    {
    case FNID_SCROLLBAR:
        {
           lResult = ScrollBarWndProc(hWnd, Msg, wParam, lParam);
           break;
        }
    case FNID_DESKTOP:
        {
           Window = UserGetWindowObject(hWnd);
           if (Window)
           {
              //ERR("FNID_DESKTOP IN\n");
              Ret = DesktopWindowProc(Window, Msg, wParam, lParam, &lResult);
              //ERR("FNID_DESKTOP OUT\n");
           }
           break;
        }
   case FNID_MENU:
       {
          Window = UserGetWindowObject(hWnd);
          if (Window)
          {
              Ret = PopupMenuWndProc( Window, Msg, wParam, lParam, &lResult);
          }
          break;
       }
   case FNID_MESSAGEWND:
       {
           Window = UserGetWindowObject(hWnd);
           if (Window)
           {
                Ret = !UserMessageWindowProc(Window, Msg, wParam, lParam, &lResult);
           }
           break;
       }
    case FNID_DEFWINDOWPROC:
        /* Validate input */
        if (hWnd)
        {
           Window = UserGetWindowObject(hWnd);
           if (!Window)
           {
               UserLeave();
               return FALSE;
           }
           UserRefObjectCo(Window, &Ref);
        }
        lResult = IntDefWindowProc(Window, Msg, wParam, lParam, Ansi);
        Ret = TRUE;
        if (hWnd)
            UserDerefObjectCo(Window);
        break;
    case FNID_SENDNOTIFYMESSAGE:
        Ret = UserSendNotifyMessage(hWnd, Msg, wParam, lParam);
        break;
    case FNID_BROADCASTSYSTEMMESSAGE:
        {
            BROADCASTPARM parm, *retparam;
            DWORD_PTR RetVal = 0;

            if (ResultInfo)
            {
                Status = SafeProbeForWrite((PVOID)ResultInfo, sizeof(BROADCASTPARM), 1);
                if (!NT_SUCCESS(Status))
                {
                    break;
                }
                else
                {
                    RtlCopyMemory(&parm, (PVOID)ResultInfo, sizeof(BROADCASTPARM));
                }
            }
            else
                break;

            if ( parm.recipients & BSM_ALLDESKTOPS ||
                 parm.recipients == BSM_ALLCOMPONENTS )
            {
               PLIST_ENTRY DesktopEntry;
               PDESKTOP rpdesk;
               HWND *List, hwndDenied = NULL;
               HDESK hDesk = NULL;
               PWND pwnd, pwndDesk;
               ULONG i;
               UINT fuFlags;

               for (DesktopEntry = InputWindowStation->DesktopListHead.Flink;
                    DesktopEntry != &InputWindowStation->DesktopListHead;
                    DesktopEntry = DesktopEntry->Flink)
               {
                  rpdesk = CONTAINING_RECORD(DesktopEntry, DESKTOP, ListEntry);
                  pwndDesk = rpdesk->pDeskInfo->spwnd;
                  List = IntWinListChildren(pwndDesk);

                  if (parm.flags & BSF_QUERY)
                  {
                     if (List != NULL)
                     {
                        if (parm.flags & BSF_FORCEIFHUNG || parm.flags & BSF_NOHANG)
                        {
                           fuFlags = SMTO_ABORTIFHUNG;
                        }
                        else if (parm.flags & BSF_NOTIMEOUTIFNOTHUNG)
                        {
                           fuFlags = SMTO_NOTIMEOUTIFNOTHUNG;
                        }
                        else
                        {
                           fuFlags = SMTO_NORMAL;
                        }
                        co_IntSendMessageTimeout( UserHMGetHandle(pwndDesk),
                                                  Msg,
                                                  wParam,
                                                  lParam,
                                                  fuFlags,
                                                  2000,
                                                 &RetVal);
                        Ret = TRUE;
                        for (i = 0; List[i]; i++)
                        {
                           pwnd = UserGetWindowObject(List[i]);
                           if (!pwnd) continue;

                           if ( pwnd->fnid == FNID_MENU ||
                                pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                              continue;

                           if ( parm.flags & BSF_IGNORECURRENTTASK )
                           {
                              if ( pwnd->head.pti == gptiCurrent )
                                 continue;
                           }
                           co_IntSendMessageTimeout( List[i],
                                                     Msg,
                                                     wParam,
                                                     lParam,
                                                     fuFlags,
                                                     2000,
                                                    &RetVal);

                           if (!RetVal && EngGetLastError() == ERROR_TIMEOUT)
                           {
                              if (!(parm.flags & BSF_FORCEIFHUNG))
                                 Ret = FALSE;
                           }
                           if (RetVal == BROADCAST_QUERY_DENY)
                           {
                              hwndDenied = List[i];
                              hDesk = UserHMGetHandle(pwndDesk);
                              Ret = FALSE;
                           }
                        }
                        ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                        Status = SafeProbeForWrite((PVOID)ResultInfo, sizeof(BROADCASTPARM), 1);
                        if (!NT_SUCCESS(Status))
                        {
                            break;
                        }
                        else
                        {
                            retparam = (PBROADCASTPARM) ResultInfo;
                            retparam->hDesk = hDesk;
                            retparam->hWnd = hwndDenied;
                        }
                        if (!Ret) break; // Have a hit! Let everyone know!
                     }
                  }
                  else if (parm.flags & BSF_POSTMESSAGE)
                  {
                     if (List != NULL)
                     {
                        UserPostMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);

                        for (i = 0; List[i]; i++)
                        {
                           pwnd = UserGetWindowObject(List[i]);
                           if (!pwnd) continue;

                           if ( pwnd->fnid == FNID_MENU ||
                                pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                              continue;

                           if ( parm.flags & BSF_IGNORECURRENTTASK )
                           {
                              if ( pwnd->head.pti == gptiCurrent )
                                 continue;
                           }
                           UserPostMessage(List[i], Msg, wParam, lParam);
                        }
                        ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                     }
                     Ret = TRUE;
                  }
                  else
                  {
                     if (List != NULL)
                     {
                        UserSendNotifyMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);

                        for (i = 0; List[i]; i++)
                        {
                           pwnd = UserGetWindowObject(List[i]);
                           if (!pwnd) continue;

                           if ( pwnd->fnid == FNID_MENU ||
                                pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                              continue;

                           if ( parm.flags & BSF_IGNORECURRENTTASK )
                           {
                              if ( pwnd->head.pti == gptiCurrent )
                                 continue;
                           }
                           UserSendNotifyMessage(List[i], Msg, wParam, lParam);
                        }
                        ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                     }
                     if (lParam && !wParam && _wcsicmp((WCHAR*)lParam, L"Environment") == 0)
                     {
                         /* Handle Broadcast of WM_SETTINGCHAGE for Environment */
                         co_IntDoSendMessage(HWND_BROADCAST, WM_SETTINGCHANGE,
                                             0, (LPARAM)L"Environment", 0);
                     }
                     Ret = TRUE;
                  }
               }
            }
            else if (parm.recipients & BSM_APPLICATIONS)
            {
               HWND *List, hwndDenied = NULL;
               HDESK hDesk = NULL;
               PWND pwnd, pwndDesk;
               ULONG i;
               UINT fuFlags;

               pwndDesk = UserGetDesktopWindow();
               List = IntWinListChildren(pwndDesk);

               if (parm.flags & BSF_QUERY)
               {
                  if (List != NULL)
                  {
                     if (parm.flags & BSF_FORCEIFHUNG || parm.flags & BSF_NOHANG)
                     {
                        fuFlags = SMTO_ABORTIFHUNG;
                     }
                     else if (parm.flags & BSF_NOTIMEOUTIFNOTHUNG)
                     {
                        fuFlags = SMTO_NOTIMEOUTIFNOTHUNG;
                     }
                     else
                     {
                        fuFlags = SMTO_NORMAL;
                     }
                     co_IntSendMessageTimeout( UserHMGetHandle(pwndDesk),
                                               Msg,
                                               wParam,
                                               lParam,
                                               fuFlags,
                                               2000,
                                              &RetVal);
                     Ret = TRUE;
                     for (i = 0; List[i]; i++)
                     {
                        pwnd = UserGetWindowObject(List[i]);
                        if (!pwnd) continue;

                        if ( pwnd->fnid == FNID_MENU ||
                             pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                           continue;

                        if ( parm.flags & BSF_IGNORECURRENTTASK )
                        {
                           if ( pwnd->head.pti == gptiCurrent )
                              continue;
                        }
                        co_IntSendMessageTimeout( List[i],
                                                  Msg,
                                                  wParam,
                                                  lParam,
                                                  fuFlags,
                                                  2000,
                                                 &RetVal);

                        if (!RetVal && EngGetLastError() == ERROR_TIMEOUT)
                        {
                           if (!(parm.flags & BSF_FORCEIFHUNG))
                              Ret = FALSE;
                        }
                        if (RetVal == BROADCAST_QUERY_DENY)
                        {
                           hwndDenied = List[i];
                           hDesk = UserHMGetHandle(pwndDesk);
                           Ret = FALSE;
                        }
                     }
                     ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                     Status = SafeProbeForWrite((PVOID)ResultInfo, sizeof(BROADCASTPARM), 1);
                     if (!NT_SUCCESS(Status))
                     {
                         break;
                     }
                     else
                     {
                         retparam = (PBROADCASTPARM) ResultInfo;
                         retparam->hDesk = hDesk;
                         retparam->hWnd = hwndDenied;
                     }
                  }
               }
               else if (parm.flags & BSF_POSTMESSAGE)
               {
                  if (List != NULL)
                  {
                     UserPostMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);

                     for (i = 0; List[i]; i++)
                     {
                        pwnd = UserGetWindowObject(List[i]);
                        if (!pwnd) continue;

                        if ( pwnd->fnid == FNID_MENU ||
                             pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                           continue;

                        if ( parm.flags & BSF_IGNORECURRENTTASK )
                        {
                           if ( pwnd->head.pti == gptiCurrent )
                              continue;
                        }
                        UserPostMessage(List[i], Msg, wParam, lParam);
                     }
                     ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                  }
                  Ret = TRUE;
               }
               else
               {
                  if (List != NULL)
                  {
                     UserSendNotifyMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);

                     for (i = 0; List[i]; i++)
                     {
                        pwnd = UserGetWindowObject(List[i]);
                        if (!pwnd) continue;

                        if ( pwnd->fnid == FNID_MENU ||
                             pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                           continue;

                        if ( parm.flags & BSF_IGNORECURRENTTASK )
                        {
                           if ( pwnd->head.pti == gptiCurrent )
                              continue;
                        }
                        UserSendNotifyMessage(List[i], Msg, wParam, lParam);
                     }
                     ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                  }
                  Ret = TRUE;
               }
            }
        }
        break;
    case FNID_SENDMESSAGECALLBACK:
        {
            CALL_BACK_INFO CallBackInfo;
            ULONG_PTR uResult;

            Status = SafeProbeForRead((PVOID)ResultInfo, sizeof(CALL_BACK_INFO), 1);
            if (!NT_SUCCESS(Status))
            {
                break;
            }
            else
            {
                RtlCopyMemory(&CallBackInfo, (PVOID)ResultInfo, sizeof(CALL_BACK_INFO));
            }

            if (is_pointer_message(Msg, wParam))
            {
               EngSetLastError(ERROR_MESSAGE_SYNC_ONLY );
               break;
            }

            if (!(Ret = co_IntSendMessageWithCallBack(hWnd, Msg, wParam, lParam,
                        CallBackInfo.CallBack, CallBackInfo.Context, &uResult)))
            {
                ERR("Callback failure!\n");
            }
        }
        break;
    case FNID_SENDMESSAGE:
        {
            lResult = co_IntDoSendMessage(hWnd, Msg, wParam, lParam, 0);
            Ret = TRUE;

            if (ResultInfo)
            {
                Status = SafeProbeForWrite((PVOID)ResultInfo, sizeof(ULONG_PTR), 1);
                if (!NT_SUCCESS(Status))
                {
                    Ret = FALSE;
                    break;
                }
                else
                {
                    RtlCopyMemory((PVOID)ResultInfo, &lResult, sizeof(ULONG_PTR));
                }
            }
            break;
        }
    case FNID_SENDMESSAGEFF:
    case FNID_SENDMESSAGEWTOOPTION:
        {
            DOSENDMESSAGE dsm, *pdsm = (PDOSENDMESSAGE)ResultInfo;
            if (ResultInfo)
            {
                Status = SafeProbeForRead(pdsm, sizeof(DOSENDMESSAGE), 1);
                if (!NT_SUCCESS(Status))
                {
                    break;
                }
                else
                {
                    RtlCopyMemory(&dsm, pdsm, sizeof(DOSENDMESSAGE));
                }
            }

            Ret = co_IntDoSendMessage( hWnd, Msg, wParam, lParam, pdsm ? &dsm : NULL );

            if (pdsm)
            {
                Status = SafeProbeForWrite(pdsm, sizeof(DOSENDMESSAGE), 1);
                if (!NT_SUCCESS(Status))
                {
                    Ret = FALSE;
                    break;
                }
                else
                {
                    RtlCopyMemory(pdsm, &dsm, sizeof(DOSENDMESSAGE));
                }
            }
            break;
        }
        // CallNextHook bypass.
    case FNID_CALLWNDPROC:
    case FNID_CALLWNDPROCRET:
        {
            PTHREADINFO pti;
            PCLIENTINFO ClientInfo;
            PHOOK NextObj, Hook;

            pti = GetW32ThreadInfo();

            Hook = pti->sphkCurrent;

            if (!Hook) break;

            NextObj = Hook->phkNext;
            ClientInfo = pti->pClientInfo;
            Status = SafeProbeForWrite(ClientInfo, sizeof(*ClientInfo), __alignof(CLIENTINFO));
            if (!NT_SUCCESS(Status))
            {
                ClientInfo = NULL;
            }
            else
            {
                ClientInfo->phkCurrent = NextObj;
            }

            if (!ClientInfo || !NextObj) break;

            NextObj->phkNext = IntGetNextHook(NextObj);

            if ( Hook->HookId == WH_CALLWNDPROC)
            {
                CWPSTRUCT CWP;
                CWP.hwnd    = hWnd;
                CWP.message = Msg;
                CWP.wParam  = wParam;
                CWP.lParam  = lParam;
                TRACE("WH_CALLWNDPROC: Hook %p NextHook %p\n", Hook, NextObj);

                lResult = co_IntCallHookProc( Hook->HookId,
                                              HC_ACTION,
                                              ((ClientInfo->CI_flags & CI_CURTHPRHOOK) ? 1 : 0),
                                              (LPARAM)&CWP,
                                              Hook->Proc,
                                              Hook->ihmod,
                                              Hook->offPfn,
                                              Hook->Ansi,
                                              &Hook->ModuleName);
            }
            else
            {
                CWPRETSTRUCT CWPR;
                CWPR.hwnd    = hWnd;
                CWPR.message = Msg;
                CWPR.wParam  = wParam;
                CWPR.lParam  = lParam;
                CWPR.lResult = ClientInfo->dwHookData;

                lResult = co_IntCallHookProc( Hook->HookId,
                                              HC_ACTION,
                                              ((ClientInfo->CI_flags & CI_CURTHPRHOOK) ? 1 : 0),
                                              (LPARAM)&CWPR,
                                              Hook->Proc,
                                              Hook->ihmod,
                                              Hook->offPfn,
                                              Hook->Ansi,
                                              &Hook->ModuleName);
            }
        }
        break;
    }

    switch(dwType)
    {
    case FNID_DEFWINDOWPROC:
    case FNID_CALLWNDPROC:
    case FNID_CALLWNDPROCRET:
    case FNID_SCROLLBAR:
    case FNID_DESKTOP:
    case FNID_MENU:
        if (ResultInfo)
        {
            Status = SafeProbeForWrite((PVOID)ResultInfo, sizeof(LRESULT), 1);
            if (!NT_SUCCESS(Status))
            {
                Ret = FALSE;
            }
            else
            {
                RtlCopyMemory((PVOID)ResultInfo, &lResult, sizeof(LRESULT));
            }
        }
        break;
    default:
        break;
    }

    UserLeave();

    return Ret;
}


#define INFINITE 0xFFFFFFFF
#define WAIT_FAILED ((DWORD)0xFFFFFFFF)

DWORD
APIENTRY
NtUserWaitForInputIdle( IN HANDLE hProcess,
                        IN DWORD dwMilliseconds,
                        IN BOOL bSharedWow)
{
    PEPROCESS Process;
    PPROCESSINFO W32Process;
    PTHREADINFO pti;
    NTSTATUS Status;
    HANDLE Handles[3];
    LARGE_INTEGER Timeout;
    KAPC_STATE ApcState;

    UserEnterExclusive();

    Status = ObReferenceObjectByHandle(hProcess,
                                       PROCESS_QUERY_INFORMATION,
                                       *PsProcessType,
                                       UserMode,
                                       (PVOID*)&Process,
                                       NULL);

    if (!NT_SUCCESS(Status))
    {
        UserLeave();
        SetLastNtError(Status);
        return WAIT_FAILED;
    }

    pti = PsGetCurrentThreadWin32Thread();

    W32Process = (PPROCESSINFO)Process->Win32Process;

    if ( PsGetProcessExitProcessCalled(Process) ||
         !W32Process ||
         pti->ppi == W32Process)
    {
        ObDereferenceObject(Process);
        UserLeave();
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return WAIT_FAILED;
    }

    Handles[0] = Process;
    Handles[1] = W32Process->InputIdleEvent;
    Handles[2] = pti->pEventQueueServer; // IntMsqSetWakeMask returns hEventQueueClient

    if (!Handles[1])
    {
        ObDereferenceObject(Process);
        UserLeave();
        return STATUS_SUCCESS;  /* no event to wait on */
    }

    if (dwMilliseconds != INFINITE)
       Timeout.QuadPart = (LONGLONG) dwMilliseconds * (LONGLONG) -10000;

    KeStackAttachProcess(&Process->Pcb, &ApcState);
    W32Process->W32PF_flags |= W32PF_WAITFORINPUTIDLE;
    for (pti = W32Process->ptiList; pti; pti = pti->ptiSibling)
    {
       pti->TIF_flags |= TIF_WAITFORINPUTIDLE;
       pti->pClientInfo->dwTIFlags = pti->TIF_flags;
    }
    KeUnstackDetachProcess(&ApcState);

    TRACE("WFII: ppi %p\n", W32Process);
    TRACE("WFII: waiting for %p\n", Handles[1] );

    /*
     * We must add a refcount to our current PROCESSINFO,
     * because anything could happen (including process death) we're leaving win32k
     */
    IntReferenceProcessInfo(W32Process);

    do
    {
        UserLeave();
        Status = KeWaitForMultipleObjects( 3,
                                           Handles,
                                           WaitAny,
                                           UserRequest,
                                           UserMode,
                                           FALSE,
                                           dwMilliseconds == INFINITE ? NULL : &Timeout,
                                           NULL);
        UserEnterExclusive();

        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
            Status = WAIT_FAILED;
            goto WaitExit;
        }

        switch (Status)
        {
        case STATUS_WAIT_0:
            goto WaitExit;

        case STATUS_WAIT_2:
            {
               MSG Msg;
               co_IntGetPeekMessage( &Msg, 0, 0, 0, PM_REMOVE | PM_QS_SENDMESSAGE, FALSE);
               ERR("WFII: WAIT 2\n");
            }
            break;

        case STATUS_TIMEOUT:
            ERR("WFII: timeout\n");
        case WAIT_FAILED:
            goto WaitExit;

        default:
            ERR("WFII: finished\n");
            Status = STATUS_SUCCESS;
            goto WaitExit;
        }
    }
    while (TRUE);

WaitExit:
    KeStackAttachProcess(&Process->Pcb, &ApcState);
    for (pti = W32Process->ptiList; pti; pti = pti->ptiSibling)
    {
       pti->TIF_flags &= ~TIF_WAITFORINPUTIDLE;
       pti->pClientInfo->dwTIFlags = pti->TIF_flags;
    }
    W32Process->W32PF_flags &= ~W32PF_WAITFORINPUTIDLE;
    KeUnstackDetachProcess(&ApcState);

    IntDereferenceProcessInfo(W32Process);
    ObDereferenceObject(Process);
    UserLeave();
    return Status;
}

/* EOF */
