/* message.split2.c — #include fragment of message.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original message.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL FASTCALL
co_IntWaitMessage( PWND Window,
                   UINT MsgFilterMin,
                   UINT MsgFilterMax )
{
    PTHREADINFO pti;
    NTSTATUS Status = STATUS_SUCCESS;
    MSG Msg;
    LONG_PTR ExtraInfo = 0;

    pti = PsGetCurrentThreadWin32Thread();

    do
    {
        if ( co_IntPeekMessage( &Msg,       // Dont reenter!
                                 Window,
                                 MsgFilterMin,
                                 MsgFilterMax,
                                 MAKELONG( PM_NOREMOVE, GetWakeMask( MsgFilterMin, MsgFilterMax)),
                                 &ExtraInfo,
                                 TRUE ) )   // act like GetMessage.
        {
            return TRUE;
        }

        /* Nothing found. Wait for new messages. */
        Status = co_MsqWaitForNewMessages( pti,
                                           Window,
                                           MsgFilterMin,
                                           MsgFilterMax);
        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
            ERR("Exit co_IntWaitMessage on error!\n");
            return FALSE;
        }
        if (Status == STATUS_USER_APC || Status == STATUS_TIMEOUT)
        {
           return FALSE;
        }
    }
    while ( TRUE );

    return FALSE;
}

BOOL APIENTRY
co_IntGetPeekMessage( PMSG pMsg,
                      HWND hWnd,
                      UINT MsgFilterMin,
                      UINT MsgFilterMax,
                      UINT RemoveMsg,
                      BOOL bGMSG )
{
    PWND Window;
    PTHREADINFO pti;
    BOOL Present = FALSE;
    NTSTATUS Status;
    LONG_PTR ExtraInfo = 0;

    if ( hWnd == HWND_TOPMOST || hWnd == HWND_BROADCAST )
        hWnd = HWND_BOTTOM;

    /* Validate input */
    if (hWnd && hWnd != HWND_BOTTOM)
    {
        if (!(Window = UserGetWindowObject(hWnd)))
        {
            if (bGMSG)
                return -1;
            else
                return FALSE;
        }
    }
    else
    {
        Window = (PWND)hWnd;
    }

    if (MsgFilterMax < MsgFilterMin)
    {
        MsgFilterMin = 0;
        MsgFilterMax = 0;
    }

    if (bGMSG)
    {
       RemoveMsg |= ((GetWakeMask( MsgFilterMin, MsgFilterMax ))<< 16);
    }

    pti = PsGetCurrentThreadWin32Thread();
    pti->pClientInfo->cSpins++; // Bump up the spin count.

    do
    {
        Present = co_IntPeekMessage( pMsg,
                                     Window,
                                     MsgFilterMin,
                                     MsgFilterMax,
                                     RemoveMsg,
                                     &ExtraInfo,
                                     bGMSG );
        if (Present)
        {
           if ( pMsg->message != WM_DEVICECHANGE || (pMsg->wParam & 0x8000) )
           {
               /* GetMessage or PostMessage must never get messages that contain pointers */
               ASSERT(FindMsgMemory(pMsg->message) == NULL);
           }

           if ( pMsg->message >= WM_DDE_FIRST && pMsg->message <= WM_DDE_LAST )
           {
              if (!IntDdeGetMessageHook(pMsg, ExtraInfo))
              {
                 TRACE("DDE Get return ERROR\n");
                 continue;
              }
           }

           if (pMsg->message != WM_PAINT && pMsg->message != WM_QUIT)
           {
              if (!RtlEqualMemory(&pti->ptLast, &pMsg->pt, sizeof(POINT)))
              {
                 pti->TIF_flags |= TIF_MSGPOSCHANGED;
              }
              pti->timeLast = pMsg->time;
              pti->ptLast   = pMsg->pt;
           }

           // The WH_GETMESSAGE hook enables an application to monitor messages about to
           // be returned by the GetMessage or PeekMessage function.

           co_HOOK_CallHooks( WH_GETMESSAGE, HC_ACTION, RemoveMsg & PM_REMOVE, (LPARAM)pMsg);

           if ( bGMSG || pMsg->message == WM_PAINT) break;
        }

        if ( bGMSG )
        {
            Status = co_MsqWaitForNewMessages( pti,
                                               Window,
                                               MsgFilterMin,
                                               MsgFilterMax);
           if ( !NT_SUCCESS(Status) ||
                Status == STATUS_USER_APC ||
                Status == STATUS_TIMEOUT )
           {
              Present = -1;
              break;
           }
        }
        else
        {
           if (!(RemoveMsg & PM_NOYIELD))
           {
              IdlePing();
              // Yield this thread!
              UserLeave();
              ZwYieldExecution();
              UserEnterExclusive();
              // Fall through to exit.
              IdlePong();
           }
           break;
        }
    }
    while( bGMSG && !Present );

    // Been spinning, time to swap vinyl...
    if (pti->pClientInfo->cSpins >= 100)
    {
       // Clear the spin cycle to fix the mix.
       pti->pClientInfo->cSpins = 0;
       //if (!(pti->TIF_flags & TIF_SPINNING)) // FIXME: Need to swap vinyl...
    }
    return Present;
}

BOOL FASTCALL
UserPostThreadMessage( PTHREADINFO pti,
                       UINT Msg,
                       WPARAM wParam,
                       LPARAM lParam )
{
    MSG Message;

    if (is_pointer_message(Msg, wParam))
    {
        EngSetLastError(ERROR_MESSAGE_SYNC_ONLY );
        return FALSE;
    }
    Message.hwnd = NULL;
    Message.message = Msg;
    Message.wParam = wParam;
    Message.lParam = lParam;
    Message.pt = gpsi->ptCursor;
    Message.time = EngGetTickCount32();
    MsqPostMessage(pti, &Message, FALSE, QS_POSTMESSAGE, 0, 0);
    return TRUE;
}

PTHREADINFO FASTCALL
IntSendTo(PWND Window, PTHREADINFO ptiCur, UINT Msg)
{
   if ( ptiCur )
   {
      if (!Window ||
           Window->head.pti == ptiCur )
      {
         return NULL;
      }
   }
   return Window ? Window->head.pti : NULL;
}

BOOL FASTCALL
UserPostMessage( HWND Wnd,
                 UINT Msg,
                 WPARAM wParam,
                 LPARAM lParam )
{
    PTHREADINFO pti;
    MSG Message;
    LONG_PTR ExtraInfo = 0;

    Message.hwnd = Wnd;
    Message.message = Msg;
    Message.wParam = wParam;
    Message.lParam = lParam;
    Message.pt = gpsi->ptCursor;
    Message.time = EngGetTickCount32();

    if (is_pointer_message(Message.message, Message.wParam))
    {
        EngSetLastError(ERROR_MESSAGE_SYNC_ONLY );
        return FALSE;
    }

    if (Wnd == HWND_BROADCAST || Wnd == HWND_TOPMOST)
    {
        HWND *List;
        PWND DesktopWindow;
        ULONG i;

        if (!is_message_broadcastable(Msg)) return TRUE;

        DesktopWindow = UserGetDesktopWindow();
        List = IntWinListChildren(DesktopWindow);

        if (List != NULL)
        {
            UserPostMessage(UserHMGetHandle(DesktopWindow), Msg, wParam, lParam);
            for (i = 0; List[i]; i++)
            {
                PWND pwnd = UserGetWindowObject(List[i]);
                if (!pwnd) continue;

                if ( pwnd->fnid == FNID_MENU || // Also need pwnd->pcls->atomClassName == gaOleMainThreadWndClass
                     pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                   continue;

                UserPostMessage(List[i], Msg, wParam, lParam);
            }
            ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
        }
    }
    else
    {
        PWND Window;

        if (!Wnd)
        {
           return UserPostThreadMessage( gptiCurrent,
                                         Msg,
                                         wParam,
                                         lParam);
        }

        Window = UserGetWindowObject(Wnd);
        if ( !Window )
        {
            ERR("UserPostMessage: Invalid handle 0x%p Msg 0x%x!\n", Wnd, Msg);
            return FALSE;
        }

        pti = Window->head.pti;

        if ( pti->TIF_flags & TIF_INCLEANUP )
        {
            ERR("Attempted to post message to window %p when the thread is in cleanup!\n", Wnd);
            return FALSE;
        }

        if ( Window->state & WNDS_DESTROYED )
        {
            ERR("Attempted to post message to window %p that is being destroyed!\n", Wnd);
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            return FALSE;
        }

        if ( Msg >= WM_DDE_FIRST && Msg <= WM_DDE_LAST )
        {
           if (!IntDdePostMessageHook(Window, Msg, wParam, &lParam, &ExtraInfo))
           {
              TRACE("Posting Exit DDE 0x%x\n",Msg);
              return FALSE;
           }
           Message.lParam = lParam;
        }

        MsqPostMessage(pti, &Message, FALSE, QS_POSTMESSAGE, 0, ExtraInfo);
    }
    return TRUE;
}

LRESULT FASTCALL
co_IntSendMessage( HWND hWnd,
                   UINT Msg,
                   WPARAM wParam,
                   LPARAM lParam )
{
    ULONG_PTR Result = 0;

    if (co_IntSendMessageTimeout(hWnd, Msg, wParam, lParam, SMTO_NORMAL, 0, &Result))
    {
        return (LRESULT)Result;
    }
    return 0;
}

static LRESULT FASTCALL
co_IntSendMessageTimeoutSingle( HWND hWnd,
                                UINT Msg,
                                WPARAM wParam,
                                LPARAM lParam,
                                UINT uFlags,
                                UINT uTimeout,
                                ULONG_PTR *uResult )
{
    NTSTATUS Status = STATUS_SUCCESS;
    PWND Window;
    PMSGMEMORY MsgMemoryEntry;
    INT lParamBufferSize;
    LPARAM lParamPacked;
    PTHREADINFO Win32Thread, ptiSendTo = NULL;
    ULONG_PTR Result = 0;
    LRESULT Ret = FALSE;
    USER_REFERENCE_ENTRY Ref;
    BOOL DoCallBack = TRUE;

    if (!(Window = UserGetWindowObject(hWnd)))
    {
        TRACE("SendMessageTimeoutSingle: Invalid handle 0x%p!\n",hWnd);
        return FALSE;
    }

    UserRefObjectCo(Window, &Ref);

    Win32Thread = PsGetCurrentThreadWin32Thread();

    ptiSendTo = IntSendTo(Window, Win32Thread, Msg);

    if ( Msg >= WM_DDE_FIRST && Msg <= WM_DDE_LAST )
    {
       if (!IntDdeSendMessageHook(Window, Msg, wParam, lParam))
       {
          ERR("Sending Exit DDE 0x%x\n",Msg);
          goto Cleanup; // Return FALSE
       }
    }

    if ( !ptiSendTo )
    {
        if (Win32Thread->TIF_flags & TIF_INCLEANUP)
        {
            /* Never send messages to exiting threads */
            goto Cleanup; // Return FALSE
        }

        if (Msg & 0x80000000)
        {
           TRACE("SMTS: Internal Message!\n");
           Result = (ULONG_PTR)handle_internal_message( Window, Msg, wParam, lParam );
           if (uResult) *uResult = Result;
           Ret = TRUE;
           goto Cleanup;
        }

        // Only happens when calling the client!
        IntCallWndProc( Window, hWnd, Msg, wParam, lParam);

        if ( Window->state & WNDS_SERVERSIDEWINDOWPROC )
        {
           TRACE("SMT: Server Side Window Procedure\n");
           // Handle it here. Safeguard against excessive recursions.
           if (IoGetRemainingStackSize() < PAGE_SIZE)
           {
              ERR("Server Callback Exceeded Stack!\n");
              goto Cleanup; // Return FALSE
           }
           /* Return after server side call, IntCallWndProcRet will not be called. */
           switch(Window->fnid)
           {
              case FNID_DESKTOP:
                DoCallBack = !DesktopWindowProc(Window, Msg, wParam, lParam,(LRESULT*)&Result);
                break;
              case FNID_MESSAGEWND:
                DoCallBack = !UserMessageWindowProc(Window, Msg, wParam, lParam,(LRESULT*)&Result);
                break;
              case FNID_MENU:
                DoCallBack = !PopupMenuWndProc( Window, Msg, wParam, lParam,(LRESULT*)&Result);
                break;
           }
           if (!DoCallBack)
           {
              if (uResult) *uResult = Result;
              Ret = TRUE;
              goto Cleanup;
           }
        }
        /* See if this message type is present in the table */
        MsgMemoryEntry = FindMsgMemory(Msg);
        if (NULL == MsgMemoryEntry)
        {
           lParamBufferSize = -1;
        }
        else
        {
           lParamBufferSize = MsgMemorySize(MsgMemoryEntry, wParam, lParam);
           // If zero, do not allow callback on client side to allocate a buffer!!!!! See CORE-7695.
           if (!lParamBufferSize) lParamBufferSize = -1;
        }

        if (! NT_SUCCESS(PackParam(&lParamPacked, Msg, wParam, lParam, FALSE)))
        {
           ERR("Failed to pack message parameters\n");
           goto Cleanup; // Return FALSE
        }

        Result = (ULONG_PTR)co_IntCallWindowProc( Window->lpfnWndProc,
                                                  !Window->Unicode,
                                                  hWnd,
                                                  Msg,
                                                  wParam,
                                                  lParamPacked,
                                                  lParamBufferSize );
        if (uResult)
        {
            *uResult = Result;
        }

        if (! NT_SUCCESS(UnpackParam(lParamPacked, Msg, wParam, lParam, FALSE)))
        {
            ERR("Failed to unpack message parameters\n");
            Ret = TRUE;
            goto Cleanup;
        }

        // Only happens when calling the client!
        IntCallWndProcRet( Window, hWnd, Msg, wParam, lParam, (LRESULT *)uResult);

        Ret = TRUE;
        goto Cleanup;
    }

    if (Window->state & WNDS_DESTROYED)
    {
        /* FIXME: Last error? */
        ERR("Attempted to send message to window %p that is being destroyed!\n", hWnd);
        goto Cleanup; // Return FALSE
    }

    if ((uFlags & SMTO_ABORTIFHUNG) && MsqIsHung(ptiSendTo, 4 * MSQ_HUNG))
    {
        // FIXME: Set window hung and add to a list.
        /* FIXME: Set a LastError? */
        ERR("Window %p (%p) (pti %p) is hung!\n", hWnd, Window, ptiSendTo);
        goto Cleanup; // Return FALSE
    }

    do
    {
        Status = co_MsqSendMessage( ptiSendTo,
                                    hWnd,
                                    Msg,
                                    wParam,
                                    lParam,
                                    uTimeout,
                                    (uFlags & SMTO_BLOCK),
                                    MSQ_NORMAL,
                                    uResult );
    }
    while ((Status == STATUS_TIMEOUT) &&
           (uFlags & SMTO_NOTIMEOUTIFNOTHUNG) &&
           !MsqIsHung(ptiSendTo, MSQ_HUNG)); // FIXME: Set window hung and add to a list.

    if (Status == STATUS_TIMEOUT)
    {
        if (0 && MsqIsHung(ptiSendTo, MSQ_HUNG))
        {
            TRACE("Let's go Ghost!\n");
            IntMakeHungWindowGhosted(hWnd);
        }
/*
 *  MSDN says:
 *  Microsoft Windows 2000: If GetLastError returns zero, then the function
 *  timed out.
 *  XP+ : If the function fails or times out, the return value is zero.
 *  To get extended error information, call GetLastError. If GetLastError
 *  returns ERROR_TIMEOUT, then the function timed out.
 */
        EngSetLastError(ERROR_TIMEOUT);
        goto Cleanup; // Return FALSE
    }
    else if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        goto Cleanup; // Return FALSE
    }

    Ret = TRUE;

Cleanup:
    UserDerefObjectCo(Window);
    return Ret;
}

LRESULT FASTCALL
co_IntSendMessageTimeout( HWND hWnd,
                          UINT Msg,
                          WPARAM wParam,
                          LPARAM lParam,
                          UINT uFlags,
                          UINT uTimeout,
                          ULONG_PTR *uResult )
{
    PWND DesktopWindow;
    HWND *Children;
    HWND *Child;

    if (hWnd != HWND_BROADCAST && hWnd != HWND_TOPMOST)
    {
        return co_IntSendMessageTimeoutSingle(hWnd, Msg, wParam, lParam, uFlags, uTimeout, uResult);
    }

    if (!is_message_broadcastable(Msg)) return TRUE;

    DesktopWindow = UserGetDesktopWindow();
    if (NULL == DesktopWindow)
    {
       EngSetLastError(ERROR_INTERNAL_ERROR);
       return 0;
    }

    if (hWnd != HWND_TOPMOST)
    {
       /* Send message to the desktop window too! */
       co_IntSendMessageTimeoutSingle(UserHMGetHandle(DesktopWindow), Msg, wParam, lParam, uFlags, uTimeout, uResult);
    }

    Children = IntWinListChildren(DesktopWindow);
    if (NULL == Children)
    {
        return 0;
    }

    for (Child = Children; NULL != *Child; Child++)
    {
        PWND pwnd = UserGetWindowObject(*Child);
        if (!pwnd) continue;

        if ( pwnd->fnid == FNID_MENU ||
             pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
           continue;

        co_IntSendMessageTimeoutSingle(*Child, Msg, wParam, lParam, uFlags, uTimeout, uResult);
    }

    ExFreePoolWithTag(Children, USERTAG_WINDOWLIST);

    return (LRESULT) TRUE;
}

LRESULT FASTCALL
co_IntSendMessageNoWait(HWND hWnd,
                        UINT Msg,
                        WPARAM wParam,
                        LPARAM lParam)
{
    ULONG_PTR Result = 0;
    return co_IntSendMessageWithCallBack(hWnd,
                                         Msg,
                                         wParam,
                                         lParam,
                                         NULL,
                                         0,
                                         &Result);
}
/* MSDN:
   If you send a message in the range below WM_USER to the asynchronous message
   functions (PostMessage, SendNotifyMessage, and SendMessageCallback), its
   message parameters cannot include pointers. Otherwise, the operation will fail.
   The functions will return before the receiving thread has had a chance to
   process the message and the sender will free the memory before it is used.
*/
LRESULT FASTCALL
co_IntSendMessageWithCallBack(HWND hWnd,
                              UINT Msg,
                              WPARAM wParam,
                              LPARAM lParam,
                              SENDASYNCPROC CompletionCallback,
                              ULONG_PTR CompletionCallbackContext,
                              ULONG_PTR *uResult)
{
    ULONG_PTR Result;
    PWND Window;
    PMSGMEMORY MsgMemoryEntry;
    INT lParamBufferSize;
    LPARAM lParamPacked;
    PTHREADINFO Win32Thread, ptiSendTo = NULL;
    LRESULT Ret = FALSE;
    USER_REFERENCE_ENTRY Ref;
    PUSER_SENT_MESSAGE Message;
    BOOL DoCallBack = TRUE;

    if (!(Window = UserGetWindowObject(hWnd)))
    {
        TRACE("SendMessageWithCallBack: Invalid handle 0x%p\n",hWnd);
        return FALSE;
    }

    UserRefObjectCo(Window, &Ref);

    if (Window->state & WNDS_DESTROYED)
    {
        /* FIXME: last error? */
        ERR("Attempted to send message to window %p that is being destroyed\n", hWnd);
        goto Cleanup; // Return FALSE
    }

    Win32Thread = PsGetCurrentThreadWin32Thread();

    if (Win32Thread == NULL || Win32Thread->TIF_flags & TIF_INCLEANUP)
        goto Cleanup; // Return FALSE

    ptiSendTo = IntSendTo(Window, Win32Thread, Msg);

    if (Msg & 0x80000000 &&
        !ptiSendTo)
    {
        if (Win32Thread->TIF_flags & TIF_INCLEANUP)
            goto Cleanup; // Return FALSE

        TRACE("SMWCB: Internal Message\n");
        Result = (ULONG_PTR)handle_internal_message(Window, Msg, wParam, lParam);
        if (uResult) *uResult = Result;
        Ret = TRUE;
        goto Cleanup;
    }

    /* See if this message type is present in the table */
    MsgMemoryEntry = FindMsgMemory(Msg);
    if (NULL == MsgMemoryEntry)
    {
        lParamBufferSize = -1;
    }
    else
    {
        lParamBufferSize = MsgMemorySize(MsgMemoryEntry, wParam, lParam);
        if (!lParamBufferSize) lParamBufferSize = -1;
    }

    if (!NT_SUCCESS(PackParam(&lParamPacked, Msg, wParam, lParam, !!ptiSendTo)))
    {
        ERR("Failed to pack message parameters\n");
        goto Cleanup; // Return FALSE
    }

    /* If it can be sent now, then send it. */
    if (!ptiSendTo)
    {
        if (Win32Thread->TIF_flags & TIF_INCLEANUP)
        {
            UnpackParam(lParamPacked, Msg, wParam, lParam, FALSE);
            /* Never send messages to exiting threads */
            goto Cleanup; // Return FALSE
        }

        IntCallWndProc(Window, hWnd, Msg, wParam, lParam);

        if (Window->state & WNDS_SERVERSIDEWINDOWPROC)
        {
           TRACE("SMWCB: Server Side Window Procedure\n");
           switch(Window->fnid)
           {
              case FNID_DESKTOP:
                DoCallBack = !DesktopWindowProc(Window, Msg, wParam, lParamPacked, (LRESULT*)&Result);
                break;
              case FNID_MESSAGEWND:
                DoCallBack = !UserMessageWindowProc(Window, Msg, wParam, lParam, (LRESULT*)&Result);
                break;
              case FNID_MENU:
                DoCallBack = !PopupMenuWndProc(Window, Msg, wParam, lParam, (LRESULT*)&Result);
                break;
           }
        }

        if (DoCallBack)
        Result = (ULONG_PTR)co_IntCallWindowProc(Window->lpfnWndProc,
                                                 !Window->Unicode,
                                                 hWnd,
                                                 Msg,
                                                 wParam,
                                                 lParamPacked,
                                                 lParamBufferSize);
        if(uResult)
        {
            *uResult = Result;
        }

        IntCallWndProcRet(Window, hWnd, Msg, wParam, lParam, (LRESULT *)uResult);

        if (CompletionCallback)
        {
            co_IntCallSentMessageCallback(CompletionCallback,
                                          hWnd,
                                          Msg,
                                          CompletionCallbackContext,
                                          Result);
        }
    }

    if (!ptiSendTo)
    {
        if (!NT_SUCCESS(UnpackParam(lParamPacked, Msg, wParam, lParam, FALSE)))
        {
            ERR("Failed to unpack message parameters\n");
        }
        Ret = TRUE;
        goto Cleanup;
    }

    if(!(Message = AllocateUserMessage(FALSE)))
    {
        ERR("Failed to allocate message\n");
        goto Cleanup; // Return FALSE
    }

    Message->Msg.hwnd = hWnd;
    Message->Msg.message = Msg;
    Message->Msg.wParam = wParam;
    Message->Msg.lParam = lParamPacked;
    Message->pkCompletionEvent = NULL; // No event needed.
    Message->lResult = 0;
    Message->QS_Flags = 0;
    Message->ptiReceiver = ptiSendTo;
    Message->ptiSender = NULL;
    Message->ptiCallBackSender = Win32Thread;
    Message->CompletionCallback = CompletionCallback;
    Message->CompletionCallbackContext = CompletionCallbackContext;
    Message->HookMessage = MSQ_NORMAL;
    Message->HasPackedLParam = (lParamBufferSize > 0);
    Message->QS_Flags = QS_SENDMESSAGE;
    Message->flags = SMF_RECEIVERFREE;

    if (Msg & 0x80000000) // Higher priority event message!
        InsertHeadList(&ptiSendTo->SentMessagesListHead, &Message->ListEntry);
    else
        InsertTailList(&ptiSendTo->SentMessagesListHead, &Message->ListEntry);
    MsqWakeQueue(ptiSendTo, QS_SENDMESSAGE, TRUE);

    Ret = TRUE;

Cleanup:
    UserDerefObjectCo(Window);
    return Ret;
}

#if 0
/*
  This HACK function posts a message if the destination's message queue belongs to
  another thread, otherwise it sends the message. It does not support broadcast
  messages!
*/
LRESULT FASTCALL
co_IntPostOrSendMessage( HWND hWnd,
                         UINT Msg,
                         WPARAM wParam,
                         LPARAM lParam )
{
    ULONG_PTR Result;
    PTHREADINFO pti;
    PWND Window;

    if ( hWnd == HWND_BROADCAST )
    {
        return 0;
    }

    if(!(Window = UserGetWindowObject(hWnd)))
    {
        TRACE("PostOrSendMessage: Invalid handle 0x%p!\n",hWnd);
        return 0;
    }

    pti = PsGetCurrentThreadWin32Thread();

    if ( IntSendTo(Window, pti, Msg) &&
         FindMsgMemory(Msg) == 0 )
    {
        Result = UserPostMessage(hWnd, Msg, wParam, lParam);
    }
    else
    {
        if ( !co_IntSendMessageTimeoutSingle(hWnd, Msg, wParam, lParam, SMTO_NORMAL, 0, &Result) )
        {
            Result = 0;
        }
    }

    return (LRESULT)Result;
}
#endif

static LRESULT FASTCALL
co_IntDoSendMessage( HWND hWnd,
                     UINT Msg,
                     WPARAM wParam,
                     LPARAM lParam,
                     PDOSENDMESSAGE dsm)
{
    LRESULT Result = TRUE;
    NTSTATUS Status;
    PWND Window = NULL;
    MSG UserModeMsg, KernelModeMsg;
    PMSGMEMORY MsgMemoryEntry;
    PTHREADINFO ptiSendTo;

    if (hWnd != HWND_BROADCAST && hWnd != HWND_TOPMOST)
    {
        Window = UserGetWindowObject(hWnd);
        if ( !Window )
        {
            return 0;
        }
    }

    /* Check for an exiting window. */
    if (Window && Window->state & WNDS_DESTROYED)
    {
        ERR("co_IntDoSendMessage Window Exiting!\n");
    }

    /* See if the current thread can handle this message */
    ptiSendTo = IntSendTo(Window, gptiCurrent, Msg);

    // If broadcasting or sending to another thread, save the users data.
    if (!Window || ptiSendTo )
    {
       UserModeMsg.hwnd    = hWnd;
       UserModeMsg.message = Msg;
       UserModeMsg.wParam  = wParam;
       UserModeMsg.lParam  = lParam;
       MsgMemoryEntry = FindMsgMemory(UserModeMsg.message);
       Status = CopyMsgToKernelMem(&KernelModeMsg, &UserModeMsg, MsgMemoryEntry);
       if (!NT_SUCCESS(Status))
       {
          EngSetLastError(ERROR_INVALID_PARAMETER);
          return (dsm ? 0 : -1);
       }
    }
    else
    {
       KernelModeMsg.hwnd    = hWnd;
       KernelModeMsg.message = Msg;
       KernelModeMsg.wParam  = wParam;
       KernelModeMsg.lParam  = lParam;
    }

    if (!dsm)
    {
       Result = co_IntSendMessage( KernelModeMsg.hwnd,
                                   KernelModeMsg.message,
                                   KernelModeMsg.wParam,
                                   KernelModeMsg.lParam );
    }
    else
    {
       Result = co_IntSendMessageTimeout( KernelModeMsg.hwnd,
                                          KernelModeMsg.message,
                                          KernelModeMsg.wParam,
                                          KernelModeMsg.lParam,
                                          dsm->uFlags,
                                          dsm->uTimeout,
                                         &dsm->Result );
    }

    if (!Window || ptiSendTo )
    {
       Status = CopyMsgToUserMem(&UserModeMsg, &KernelModeMsg);
       if (!NT_SUCCESS(Status))
       {
          EngSetLastError(ERROR_INVALID_PARAMETER);
          return(dsm ? 0 : -1);
       }
    }

    return (LRESULT)Result;
}

BOOL FASTCALL
UserSendNotifyMessage( HWND hWnd,
                       UINT Msg,
                       WPARAM wParam,
                       LPARAM lParam )
{
    BOOL Ret = TRUE;

    if (is_pointer_message(Msg, wParam))
    {
        EngSetLastError(ERROR_MESSAGE_SYNC_ONLY );
        return FALSE;
    }

    // Basicly the same as IntPostOrSendMessage
    if (hWnd == HWND_BROADCAST) // Handle Broadcast
    {
        HWND *List;
        PWND DesktopWindow;
        ULONG i;

        DesktopWindow = UserGetDesktopWindow();
        List = IntWinListChildren(DesktopWindow);

        if (List != NULL)
        {
            UserSendNotifyMessage(UserHMGetHandle(DesktopWindow), Msg, wParam, lParam);
            for (i = 0; List[i]; i++)
            {
               PWND pwnd = UserGetWindowObject(List[i]);
               if (!pwnd) continue;

               if ( pwnd->fnid == FNID_MENU ||
                    pwnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_SWITCH] )
                  continue;

               Ret = UserSendNotifyMessage(List[i], Msg, wParam, lParam);
            }
            ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
        }
    }
    else
    {
        Ret = co_IntSendMessageNoWait( hWnd, Msg, wParam, lParam);
    }
    return Ret;
}