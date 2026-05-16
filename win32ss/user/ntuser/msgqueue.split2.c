/* msgqueue.split2.c — #include fragment of msgqueue.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original msgqueue.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL FASTCALL
co_MsqSendMessageAsync(PTHREADINFO ptiReceiver,
                       HWND hwnd,
                       UINT Msg,
                       WPARAM wParam,
                       LPARAM lParam,
                       SENDASYNCPROC CompletionCallback,
                       ULONG_PTR CompletionCallbackContext,
                       BOOL HasPackedLParam,
                       INT HookMessage)
{
    PTHREADINFO ptiSender;
    PUSER_SENT_MESSAGE Message;

    if(!(Message = AllocateUserMessage(FALSE)))
    {
        ERR("MsqSendMessageAsync(): Not enough memory to allocate a message\n");
        return FALSE;
    }

    ptiSender = PsGetCurrentThreadWin32Thread();

    Message->Msg.hwnd = hwnd;
    Message->Msg.message = Msg;
    Message->Msg.wParam = wParam;
    Message->Msg.lParam = lParam;
    Message->pkCompletionEvent = NULL; // No event needed.
    Message->ptiReceiver = ptiReceiver;
    Message->ptiCallBackSender = ptiSender;
    Message->CompletionCallback = CompletionCallback;
    Message->CompletionCallbackContext = CompletionCallbackContext;
    Message->HookMessage = HookMessage;
    Message->HasPackedLParam = HasPackedLParam;
    Message->QS_Flags = QS_SENDMESSAGE;
    Message->flags = SMF_RECEIVERFREE;

    InsertTailList(&ptiReceiver->SentMessagesListHead, &Message->ListEntry);
    MsqWakeQueue(ptiReceiver, QS_SENDMESSAGE, TRUE);

    return TRUE;
}

NTSTATUS FASTCALL
co_MsqSendMessage(PTHREADINFO ptirec,
                  HWND Wnd,
                  UINT Msg,
                  WPARAM wParam,
                  LPARAM lParam,
                  UINT uTimeout,
                  BOOL Block,
                  INT HookMessage,
                  ULONG_PTR *uResult)
{
   PTHREADINFO pti;
   PUSER_SENT_MESSAGE SaveMsg, Message;
   NTSTATUS WaitStatus;
   LARGE_INTEGER Timeout;
   PLIST_ENTRY Entry;
   PWND pWnd;
   BOOLEAN SwapStateEnabled;
   LRESULT Result = 0;   //// Result could be trashed. ////

   pti = PsGetCurrentThreadWin32Thread();
   ASSERT(pti != ptirec);
   ASSERT(ptirec->pcti); // Send must have a client side to receive it!!!!

   /* Don't send from or to a dying thread */
   if (pti->TIF_flags & TIF_INCLEANUP || ptirec->TIF_flags & TIF_INCLEANUP)
   {
       // Unless we are dying and need to tell our parents.
       if (pti->TIF_flags & TIF_INCLEANUP && !(ptirec->TIF_flags & TIF_INCLEANUP))
       {
           // Parent notify is the big one. Fire and forget!
           TRACE("Send message from dying thread %u\n", Msg);
           co_MsqSendMessageAsync(ptirec, Wnd, Msg, wParam, lParam, NULL, 0, FALSE, HookMessage);
       }
       if (uResult) *uResult = -1;
       TRACE("MsqSM: Msg %u Current pti %lu or Rec pti %lu\n", Msg, pti->TIF_flags & TIF_INCLEANUP, ptirec->TIF_flags & TIF_INCLEANUP);
       return STATUS_UNSUCCESSFUL;
   }

   if (IsThreadSuspended(ptirec))
   {
      ERR("Sending to Suspended Thread Msg %lx\n",Msg);
      if (uResult) *uResult = -1;
      return STATUS_UNSUCCESSFUL;
   }

   // Should we do the same for No Wait?
   if ( HookMessage == MSQ_NORMAL )
   {
      pWnd = ValidateHwndNoErr(Wnd);

      // These can not cross International Border lines!
      if ( pti->ppi != ptirec->ppi && pWnd )
      {
         switch(Msg)
         {
             // Handle the special case when working with password transfers across bordering processes.
             case EM_GETLINE:
             case EM_SETPASSWORDCHAR:
             case WM_GETTEXT:
                // Look for edit controls setup for passwords.
                if ( gpsi->atomSysClass[ICLS_EDIT] == pWnd->pcls->atomClassName && // Use atomNVClassName.
                     pWnd->style & ES_PASSWORD )
                {
                   if (uResult) *uResult = -1;
                   ERR("Running across the border without a passport!\n");
                   EngSetLastError(ERROR_ACCESS_DENIED);
                   return STATUS_UNSUCCESSFUL;
                }
                break;
             case WM_NOTIFY:
                if (uResult) *uResult = -1;
                ERR("Running across the border without a passport!\n");
                return STATUS_UNSUCCESSFUL;
         }
      }

      // These can not cross State lines!
      if ( Msg == WM_CREATE || Msg == WM_NCCREATE )
      {
         if (uResult) *uResult = -1;
         ERR("Can not tell the other State we have Create!\n");
         return STATUS_UNSUCCESSFUL;
      }
   }

   if(!(Message = AllocateUserMessage(TRUE)))
   {
      ERR("MsqSendMessage(): Not enough memory to allocate a message\n");
      if (uResult) *uResult = -1;
      return STATUS_INSUFFICIENT_RESOURCES;
   }

   Timeout.QuadPart = Int32x32To64(-10000,uTimeout); // Pass SMTO test with a TO of 0x80000000.
   TRACE("Timeout val %lld\n",Timeout.QuadPart);

   Message->Msg.hwnd = Wnd;
   Message->Msg.message = Msg;
   Message->Msg.wParam = wParam;
   Message->Msg.lParam = lParam;
   Message->ptiReceiver = ptirec;
   Message->ptiSender = pti;
   Message->HookMessage = HookMessage;
   Message->QS_Flags = QS_SENDMESSAGE;

   SaveMsg = pti->pusmSent;
   pti->pusmSent = Message;

   /* Queue it in the destination's message queue */
   InsertTailList(&ptirec->SentMessagesListHead, &Message->ListEntry);

   MsqWakeQueue(ptirec, QS_SENDMESSAGE, TRUE);

   // First time in, turn off swapping of the stack.
   if (pti->cEnterCount == 0)
   {
      SwapStateEnabled = KeSetKernelStackSwapEnable(FALSE);
   }
   pti->cEnterCount++;

   if (Block)
   {
      PVOID WaitObjects[2];

      WaitObjects[0] = Message->pkCompletionEvent; // Wait 0
      WaitObjects[1] = ptirec->pEThread;           // Wait 1

      UserLeaveCo();

      WaitStatus = KeWaitForMultipleObjects( 2,
                                             WaitObjects,
                                             WaitAny,
                                             UserRequest,
                                             UserMode,
                                             FALSE,
                                            (uTimeout ? &Timeout : NULL),
                                             NULL );

      UserEnterCo();

      if (WaitStatus == STATUS_TIMEOUT)
      {
         /* Look up if the message has not yet dispatched, if so
            make sure it can't pass a result and it must not set the completion event anymore */
         Entry = ptirec->SentMessagesListHead.Flink;
         while (Entry != &ptirec->SentMessagesListHead)
         {
            if (CONTAINING_RECORD(Entry, USER_SENT_MESSAGE, ListEntry) == Message)
            {
               Message->pkCompletionEvent = NULL;
               RemoveEntryList(&Message->ListEntry);
               ClearMsgBitsMask(ptirec, Message->QS_Flags);
               InsertTailList(&usmList, &Message->ListEntry);
               break;
            }
            Entry = Entry->Flink;
         }

         ERR("MsqSendMessage (blocked) timed out 1 Status %lx\n", WaitStatus);
      }
      // Receiving thread passed on and left us hanging with issues still pending.
      else if (WaitStatus == STATUS_WAIT_1)
      {
         ERR("Bk Receiving Thread woken up dead!\n");
         Message->flags |= SMF_RECEIVERDIED;
      }

      while (co_MsqDispatchOneSentMessage(pti))
         ;
   }
   else
   {
      PVOID WaitObjects[3];

      WaitObjects[0] = Message->pkCompletionEvent; // Wait 0
      WaitObjects[1] = pti->pEventQueueServer;     // Wait 1
      WaitObjects[2] = ptirec->pEThread;           // Wait 2

      do
      {
         UserLeaveCo();

         WaitStatus = KeWaitForMultipleObjects( 3,
                                                WaitObjects,
                                                WaitAny,
                                                UserRequest,
                                                UserMode,
                                                FALSE,
                                               (uTimeout ? &Timeout : NULL),
                                                NULL);

         UserEnterCo();

         if (WaitStatus == STATUS_TIMEOUT)
         {
            /* Look up if the message has not yet been dispatched, if so
               make sure it can't pass a result and it must not set the completion event anymore */
            Entry = ptirec->SentMessagesListHead.Flink;
            while (Entry != &ptirec->SentMessagesListHead)
            {
               if (CONTAINING_RECORD(Entry, USER_SENT_MESSAGE, ListEntry) == Message)
               {
                  Message->pkCompletionEvent = NULL;
                  RemoveEntryList(&Message->ListEntry);
                  ClearMsgBitsMask(ptirec, Message->QS_Flags);
                  InsertTailList(&usmList, &Message->ListEntry);
                  break;
               }
               Entry = Entry->Flink;
            }

            WARN("MsqSendMessage timed out 2 Status %lx\n", WaitStatus);
            break;
         }
         // Receiving thread passed on and left us hanging with issues still pending.
         else if (WaitStatus == STATUS_WAIT_2)
         {
            ERR("NB Receiving Thread woken up dead!\n");
            Message->flags |= SMF_RECEIVERDIED;
            break;
         }

         if (WaitStatus == STATUS_USER_APC) break;

         while (co_MsqDispatchOneSentMessage(pti))
            ;
      } while (WaitStatus == STATUS_WAIT_1);
   }

   // Count is nil, restore swapping of the stack.
   if (--pti->cEnterCount == 0 )
   {
      KeSetKernelStackSwapEnable(SwapStateEnabled);
   }

   // Handle User APC
   if (WaitStatus == STATUS_USER_APC)
   {
     // The current thread is dying!
     TRACE("User APC\n");

     // The Message will be on the Trouble list until Thread cleanup.
     Message->flags |= SMF_SENDERDIED;

     co_IntDeliverUserAPC();
     ERR("User APC Returned\n"); // Should not see this message.
   }

   // Force this thread to wake up for the next go around.
   KeSetEvent(pti->pEventQueueServer, IO_NO_INCREMENT, FALSE);

   Result = Message->lResult;

   // Determine whether this message is being processed or not.
   if ((Message->flags & (SMF_RECEIVERBUSY|SMF_RECEIVEDMESSAGE)) != SMF_RECEIVEDMESSAGE)
   {
      Message->flags |= SMF_RECEIVERFREE;
   }

   if (!(Message->flags & SMF_RECEIVERFREE))
   {
      TRACE("Sender Freeing Message %p ptirec %p bit %d list empty %d\n",Message,ptirec,!!(ptirec->pcti->fsChangeBits & QS_SENDMESSAGE),IsListEmpty(&ptirec->SentMessagesListHead));
      // Make it to this point, the message was received.
      FreeUserMessage(Message);
   }

   pti->pusmSent = SaveMsg;

   TRACE("MSM Allocation Count %d Status %lx Result %d\n",SendMsgCount,WaitStatus,Result);

   if (WaitStatus != STATUS_TIMEOUT)
   {
      if (uResult)
      {
         *uResult = (STATUS_WAIT_0 == WaitStatus ? Result : 0);
      }
   }

   return WaitStatus;
}

VOID FASTCALL
MsqPostMessage(PTHREADINFO pti,
               MSG* Msg,
               BOOLEAN HardwareMessage,
               DWORD MessageBits,
               DWORD dwQEvent,
               LONG_PTR ExtraInfo)
{
   PUSER_MESSAGE Message;
   PUSER_MESSAGE_QUEUE MessageQueue;

   MessageQueue = pti->MessageQueue;

   if ((pti->TIF_flags & TIF_INCLEANUP) || (MessageQueue->QF_flags & QF_INDESTROY))
   {
      ERR("Post Msg; Thread or Q is Dead!\n");
      return;
   }

   Message = MsqCreateMessage(Msg);
   if (!Message)
      return;

   if (Msg->message == WM_HOTKEY)
      MessageBits |= QS_HOTKEY;

   Message->dwQEvent = dwQEvent;
   Message->ExtraInfo = ExtraInfo;
   Message->QS_Flags = MessageBits;
   Message->pti = pti;

   if (!HardwareMessage)
   {
       InsertTailList(&pti->PostedMessagesListHead, &Message->ListEntry);
   }
   else
   {
       InsertTailList(&MessageQueue->HardwareMessagesListHead, &Message->ListEntry);
   }

   MsqWakeQueue(pti, MessageBits, TRUE);
   TRACE("Post Message %d\n", PostMsgCount);
}

VOID FASTCALL
MsqPostQuitMessage(PTHREADINFO pti, ULONG ExitCode)
{
   pti->QuitPosted = TRUE;
   pti->exitCode = ExitCode;
   MsqWakeQueue(pti, QS_POSTMESSAGE|QS_ALLPOSTMESSAGE, TRUE);
}

/***********************************************************************
 *           MsqSendParentNotify
 *
 * Send a WM_PARENTNOTIFY to all ancestors of the given window, unless
 * the window has the WS_EX_NOPARENTNOTIFY style.
 */
static void MsqSendParentNotify( PWND pwnd, WORD event, WORD idChild, POINT pt )
{
    PWND pwndDesktop = UserGetDesktopWindow();

    /* pt has to be in the client coordinates of the parent window */
    pt.x += pwndDesktop->rcClient.left - pwnd->rcClient.left;
    pt.y += pwndDesktop->rcClient.top - pwnd->rcClient.top;

    for (;;)
    {
        PWND pwndParent;

        if (!(pwnd->style & WS_CHILD)) break;
        if (pwnd->ExStyle & WS_EX_NOPARENTNOTIFY) break;
        if (!(pwndParent = IntGetParent(pwnd))) break;
        if (pwndParent == pwndDesktop) break;
        pt.x += pwnd->rcClient.left - pwndParent->rcClient.left;
        pt.y += pwnd->rcClient.top - pwndParent->rcClient.top;

        pwnd = pwndParent;
        co_IntSendMessage( UserHMGetHandle(pwnd), WM_PARENTNOTIFY,
                      MAKEWPARAM( event, idChild ), MAKELPARAM( pt.x, pt.y ) );
    }
}

VOID
FASTCALL
IntTrackMouseMove(PWND pwndTrack, PDESKTOP pDesk, PMSG msg, USHORT hittest)
{
//   PWND pwndTrack = IntChildrenWindowFromPoint(pwndMsg, msg->pt.x, msg->pt.y);
//   hittest = (USHORT)GetNCHitEx(pwndTrack, msg->pt); /// @todo WTF is this???

   if ( pDesk->spwndTrack != pwndTrack || // Change with tracking window or
        msg->message != WM_MOUSEMOVE   || // Mouse click changes or
        pDesk->htEx != hittest)           // Change in current hit test states.
   {
      TRACE("ITMM: Track Mouse Move!\n");

      /* Handle only the changing window track and mouse move across a border. */
      if ( pDesk->spwndTrack != pwndTrack ||
          (pDesk->htEx == HTCLIENT) ^ (hittest == HTCLIENT) )
      {
         TRACE("ITMM: Another Wnd %d or Across Border %d\n",
              pDesk->spwndTrack != pwndTrack,(pDesk->htEx == HTCLIENT) ^ (hittest == HTCLIENT));

         if ( pDesk->dwDTFlags & DF_TME_LEAVE )
            UserPostMessage( UserHMGetHandle(pDesk->spwndTrack),
                            (pDesk->htEx != HTCLIENT) ? WM_NCMOUSELEAVE : WM_MOUSELEAVE,
                             0, 0);

         if ( pDesk->dwDTFlags & DF_TME_HOVER )
            IntKillTimer(pDesk->spwndTrack, ID_EVENT_SYSTIMER_MOUSEHOVER, TRUE);

         /* Clear the flags to sign a change. */
         pDesk->dwDTFlags &= ~(DF_TME_LEAVE|DF_TME_HOVER);
      }
      /* Set the Track window and hit test. */
      pDesk->spwndTrack = pwndTrack;
      pDesk->htEx = hittest;
   }

   /* Reset, Same Track window, Hover set and Mouse Clicks or Clobbered Hover box. */
   if ( pDesk->spwndTrack == pwndTrack &&
       ( msg->message != WM_MOUSEMOVE || !RECTL_bPointInRect(&pDesk->rcMouseHover, msg->pt.x, msg->pt.y)) &&
        pDesk->dwDTFlags & DF_TME_HOVER )
   {
      TRACE("ITMM: Reset Hover points!\n");
      // Restart timer for the hover period.
      IntSetTimer(pDesk->spwndTrack, ID_EVENT_SYSTIMER_MOUSEHOVER, pDesk->dwMouseHoverTime, SystemTimerProc, TMRF_SYSTEM);
      // Reset desktop mouse hover from the system default hover rectangle.
      RECTL_vSetRect(&pDesk->rcMouseHover,
                      msg->pt.x - gspv.iMouseHoverWidth  / 2,
                      msg->pt.y - gspv.iMouseHoverHeight / 2,
                      msg->pt.x + gspv.iMouseHoverWidth  / 2,
                      msg->pt.y + gspv.iMouseHoverHeight / 2);
   }
}

BOOL co_IntProcessMouseMessage(MSG* msg, BOOL* RemoveMessages, BOOL* NotForUs, LONG_PTR ExtraInfo, UINT first, UINT last)
{
    MSG clk_msg;
    POINT pt;
    UINT message;
    USHORT hittest;
    EVENTMSG event;
    MOUSEHOOKSTRUCT hook;
    BOOL eatMsg = FALSE;

    PWND pwndMsg, pwndDesktop;
    PUSER_MESSAGE_QUEUE MessageQueue;
    PTHREADINFO pti;
    PSYSTEM_CURSORINFO CurInfo;
    PDESKTOP pDesk;

    pti = PsGetCurrentThreadWin32Thread();
    pwndDesktop = UserGetDesktopWindow();
    if (pwndDesktop == NULL)
    {
        ERR("No desktop window!\n");
        return FALSE;
    }

    MessageQueue = pti->MessageQueue;
    CurInfo = IntGetSysCursorInfo();
    pwndMsg = ValidateHwndNoErr(msg->hwnd);
    clk_msg = MessageQueue->msgDblClk;
    pDesk = pwndDesktop->head.rpdesk;

    /* find the window to dispatch this mouse message to */
    if (MessageQueue->spwndCapture)
    {
        hittest = HTCLIENT;
        pwndMsg = MessageQueue->spwndCapture;
    }
    else
    {
        /*
           Start with null window. See wine win.c:test_mouse_input:WM_COMMAND tests.
        */
        pwndMsg = co_WinPosWindowFromPoint( NULL, &msg->pt, &hittest, FALSE);
    }

    TRACE("Got mouse message for %p, hittest: 0x%x\n", msg->hwnd, hittest);

    // Null window or not the same "Hardware" message queue.
    if (pwndMsg == NULL || pwndMsg->head.pti->MessageQueue != MessageQueue)
    {
        // Crossing a boundary, so set cursor. See default message queue cursor.
        IntSystemSetCursor(SYSTEMCUR(ARROW));
        /* Remove and ignore the message */
        *RemoveMessages = TRUE;
        return FALSE;
    }

    // Check to see if this is attached,
    if ( pwndMsg->head.pti != pti &&  // window thread is not current,
         MessageQueue->cThreads > 1 ) // and is attached...
    {
        // This is not for us and we should leave so the other thread can check for messages!!!
        *NotForUs = TRUE;
        *RemoveMessages = FALSE;
        return FALSE;
    }

    if ( MessageQueue == gpqCursor ) // Cursor must use the same Queue!
    {
       IntTrackMouseMove(pwndMsg, pDesk, msg, hittest);
    }
    else
    {
       WARN("Not the same cursor!\n");
    }

    msg->hwnd = UserHMGetHandle(pwndMsg);

    pt = msg->pt;
    message = msg->message;

    /* Note: windows has no concept of a non-client wheel message */
    if (message != WM_MOUSEWHEEL)
    {
        if (hittest != HTCLIENT)
        {
            message += WM_NCMOUSEMOVE - WM_MOUSEMOVE;
            msg->wParam = hittest; // Caution! This might break wParam check in DblClk.
        }
        else
        {
            /* coordinates don't get translated while tracking a menu */
            /* FIXME: should differentiate popups and top-level menus */
            if (!(MessageQueue->MenuOwner))
            {
                pt.x += pwndDesktop->rcClient.left - pwndMsg->rcClient.left;
                pt.y += pwndDesktop->rcClient.top - pwndMsg->rcClient.top;
            }
        }
    }
    msg->lParam = MAKELONG( pt.x, pt.y );

    /* translate double-clicks */

    if ((msg->message == WM_LBUTTONDOWN) ||
        (msg->message == WM_RBUTTONDOWN) ||
        (msg->message == WM_MBUTTONDOWN) ||
        (msg->message == WM_XBUTTONDOWN))
    {
        BOOL update = *RemoveMessages;

        /* translate double-clicks -
         * note that ...MOUSEMOVEs can slip in between
         * ...BUTTONDOWN and ...BUTTONDBLCLK messages */

        if ((MessageQueue->MenuOwner || MessageQueue->MoveSize) ||
            hittest != HTCLIENT ||
            (pwndMsg->pcls->style & CS_DBLCLKS))
        {
           if ((msg->message == clk_msg.message) &&
               (msg->hwnd == clk_msg.hwnd) &&
               // Only worry about XButton wParam.
               (msg->message != WM_XBUTTONDOWN || GET_XBUTTON_WPARAM(msg->wParam) == GET_XBUTTON_WPARAM(clk_msg.wParam)) &&
               ((msg->time - clk_msg.time) < (ULONG)gspv.iDblClickTime) &&
               (abs(msg->pt.x - clk_msg.pt.x) < UserGetSystemMetrics(SM_CXDOUBLECLK)/2) &&
               (abs(msg->pt.y - clk_msg.pt.y) < UserGetSystemMetrics(SM_CYDOUBLECLK)/2))
           {
               message += (WM_LBUTTONDBLCLK - WM_LBUTTONDOWN);
               if (update)
               {
                   MessageQueue->msgDblClk.message = 0;  /* clear the double-click conditions */
                   update = FALSE;
               }
           }
        }

        if (!((first ==  0 && last == 0) || (message >= first || message <= last)))
        {
            TRACE("Message out of range!!!\n");
            return FALSE;
        }

        /* update static double-click conditions */
        if (update) MessageQueue->msgDblClk = *msg;
    }
    else
    {
        if (!((first ==  0 && last == 0) || (message >= first || message <= last)))
        {
            TRACE("Message out of range!!!\n");
            return FALSE;
        }

        // Update mouse move down keys.
        if (message == WM_MOUSEMOVE)
        {
           msg->wParam = MsqGetDownKeyState(MessageQueue);
        }
    }

    if (gspv.bMouseClickLock)
    {
        BOOL IsClkLck = FALSE;

        if(msg->message == WM_LBUTTONUP)
        {
            IsClkLck = ((msg->time - CurInfo->ClickLockTime) >= gspv.dwMouseClickLockTime);
            if (IsClkLck && (!CurInfo->ClickLockActive))
            {
                CurInfo->ClickLockActive = TRUE;
            }
        }
        else if (msg->message == WM_LBUTTONDOWN)
        {
            if (CurInfo->ClickLockActive)
            {
                IsClkLck = TRUE;
                CurInfo->ClickLockActive = FALSE;
            }

            CurInfo->ClickLockTime = msg->time;
        }

        if(IsClkLck)
        {
            /* Remove and ignore the message */
            *RemoveMessages = TRUE;
            TRACE("Remove and ignore the message\n");
            return FALSE;
        }
    }

    if (pti->TIF_flags & TIF_MSGPOSCHANGED)
    {
        pti->TIF_flags &= ~TIF_MSGPOSCHANGED;
        IntNotifyWinEvent(EVENT_OBJECT_LOCATIONCHANGE, NULL, OBJID_CLIENT, CHILDID_SELF, 0);
    }

    /* message is accepted now (but still get dropped) */

    event.message = msg->message;
    event.time    = msg->time;
    event.hwnd    = msg->hwnd;
    event.paramL  = msg->pt.x;
    event.paramH  = msg->pt.y;
    co_HOOK_CallHooks( WH_JOURNALRECORD, HC_ACTION, 0, (LPARAM)&event );

    hook.pt           = msg->pt;
    hook.hwnd         = msg->hwnd;
    hook.wHitTestCode = hittest;
    hook.dwExtraInfo  = ExtraInfo;
    if (co_HOOK_CallHooks( WH_MOUSE, *RemoveMessages ? HC_ACTION : HC_NOREMOVE,
                        message, (LPARAM)&hook ))
    {
        hook.pt           = msg->pt;
        hook.hwnd         = msg->hwnd;
        hook.wHitTestCode = hittest;
        hook.dwExtraInfo  = ExtraInfo;
        co_HOOK_CallHooks( WH_CBT, HCBT_CLICKSKIPPED, message, (LPARAM)&hook );

        ERR("WH_MOUSE dropped mouse message!\n");

        /* Remove and skip message */
        *RemoveMessages = TRUE;
        return FALSE;
    }

    if ((hittest == (USHORT)HTERROR) || (hittest == (USHORT)HTNOWHERE))
    {
        co_IntSendMessage( msg->hwnd, WM_SETCURSOR, (WPARAM)msg->hwnd, MAKELONG( hittest, msg->message ));

        /* Remove and skip message */
        *RemoveMessages = TRUE;
        return FALSE;
    }

    if ((*RemoveMessages == FALSE) || MessageQueue->spwndCapture)
    {
        /* Accept the message */
        msg->message = message;
        return TRUE;
    }

    if ((msg->message == WM_LBUTTONDOWN) ||
        (msg->message == WM_RBUTTONDOWN) ||
        (msg->message == WM_MBUTTONDOWN) ||
        (msg->message == WM_XBUTTONDOWN))
    {
        /* Send the WM_PARENTNOTIFY,
         * note that even for double/nonclient clicks
         * notification message is still WM_L/M/RBUTTONDOWN.
         */
        MsqSendParentNotify(pwndMsg, msg->message, 0, msg->pt );

        /* Activate the window if needed */

        if (pwndMsg != MessageQueue->spwndActive)
        {
            PWND pwndTop = pwndMsg;
            pwndTop = IntGetNonChildAncestor(pwndTop);

            TRACE("Mouse pti %p pwndMsg pti %p pwndTop pti %p\n",MessageQueue->ptiMouse,pwndMsg->head.pti,pwndTop->head.pti);

            if (pwndTop && pwndTop != pwndDesktop)
            {
                LONG ret = co_IntSendMessage( msg->hwnd,
                                              WM_MOUSEACTIVATE,
                                              (WPARAM)UserHMGetHandle(pwndTop),
                                              MAKELONG( hittest, msg->message));
                switch(ret)
                {
                case MA_NOACTIVATEANDEAT:
                    eatMsg = TRUE;
                    /* fall through */
                case MA_NOACTIVATE:
                    break;
                case MA_ACTIVATEANDEAT:
                    eatMsg = TRUE;
                    /* fall through */
                case MA_ACTIVATE:
                case 0:
                    if (!co_IntMouseActivateWindow( pwndTop )) eatMsg = TRUE;
                    break;
                default:
                    ERR( "unknown WM_MOUSEACTIVATE code %d\n", ret );
                    break;
                }
            }
        }
    }

    /* send the WM_SETCURSOR message */

    /* Windows sends the normal mouse message as the message parameter
       in the WM_SETCURSOR message even if it's non-client mouse message */
    co_IntSendMessage( msg->hwnd, WM_SETCURSOR, (WPARAM)msg->hwnd, MAKELONG( hittest, msg->message ));

    msg->message = message;
    return !eatMsg;
}

BOOL co_IntProcessKeyboardMessage(MSG* Msg, BOOL* RemoveMessages)
{
    EVENTMSG Event;
    USER_REFERENCE_ENTRY Ref;
    PWND pWnd;
    UINT ImmRet;
    BOOL Ret = TRUE, bKeyUpDown = FALSE;
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
    const UINT uMsg = Msg->message;

    if (uMsg == VK_PACKET)
        pti->wchInjected = HIWORD(Msg->wParam);

    if (uMsg == WM_KEYDOWN || uMsg == WM_SYSKEYDOWN || uMsg == WM_KEYUP || uMsg == WM_SYSKEYUP)
    {
        bKeyUpDown = TRUE;
        switch (Msg->wParam)
        {
            case VK_LSHIFT: case VK_RSHIFT:
                Msg->wParam = VK_SHIFT;
                break;
            case VK_LCONTROL: case VK_RCONTROL:
                Msg->wParam = VK_CONTROL;
                break;
            case VK_LMENU: case VK_RMENU:
                Msg->wParam = VK_MENU;
                break;
        }
    }

    pWnd = ValidateHwndNoErr(Msg->hwnd);
    if (pWnd) UserRefObjectCo(pWnd, &Ref);

    Event.message = uMsg;
    Event.hwnd    = Msg->hwnd;
    Event.time    = Msg->time;
    Event.paramL  = (Msg->wParam & 0xFF) | (HIWORD(Msg->lParam) << 8);
    Event.paramH  = Msg->lParam & 0x7FFF;
    if (HIWORD(Msg->lParam) & 0x0100) Event.paramH |= 0x8000;
    co_HOOK_CallHooks( WH_JOURNALRECORD, HC_ACTION, 0, (LPARAM)&Event);

    if (*RemoveMessages)
    {
        if ((uMsg == WM_KEYDOWN) &&
            (Msg->hwnd != IntGetDesktopWindow()))
        {
            /* Handle F1 key by sending out WM_HELP message */
            if (Msg->wParam == VK_F1)
            {
                UserPostMessage( Msg->hwnd, WM_KEYF1, 0, 0 );
            }
            else if (Msg->wParam >= VK_BROWSER_BACK &&
                     Msg->wParam <= VK_LAUNCH_APP2)
            {
                /* FIXME: Process keystate */
                co_IntSendMessage(Msg->hwnd, WM_APPCOMMAND, (WPARAM)Msg->hwnd, MAKELPARAM(0, (FAPPCOMMAND_KEY | (Msg->wParam - VK_BROWSER_BACK + 1))));
            }
        }
        else if (uMsg == WM_KEYUP)
        {
            /* Handle VK_APPS key by posting a WM_CONTEXTMENU message */
            if (Msg->wParam == VK_APPS && pti->MessageQueue->MenuOwner == NULL)
                UserPostMessage( Msg->hwnd, WM_CONTEXTMENU, (WPARAM)Msg->hwnd, -1 );
        }
    }

    //// Key Down!
    if (*RemoveMessages && uMsg == WM_SYSKEYDOWN)
    {
        if ( HIWORD(Msg->lParam) & KF_ALTDOWN )
        {
            if ( Msg->wParam == VK_ESCAPE || Msg->wParam == VK_TAB ) // Alt-Tab/ESC Alt-Shift-Tab/ESC
            {
                WPARAM wParamTmp;

                wParamTmp = UserGetKeyState(VK_SHIFT) & 0x8000 ? SC_PREVWINDOW : SC_NEXTWINDOW;
                TRACE("Send WM_SYSCOMMAND Alt-Tab/ESC Alt-Shift-Tab/ESC\n");
                co_IntSendMessage( Msg->hwnd, WM_SYSCOMMAND, wParamTmp, Msg->wParam );

                //// Keep looping.
                Ret = FALSE;
                //// Skip the rest.
                goto Exit;
            }
        }
    }

    if (co_HOOK_CallHooks( WH_KEYBOARD,
                          *RemoveMessages ? HC_ACTION : HC_NOREMOVE,
                           LOWORD(Msg->wParam),
                           Msg->lParam))
    {
        /* skip this message */
        co_HOOK_CallHooks( WH_CBT,
                           HCBT_KEYSKIPPED,
                           LOWORD(Msg->wParam),
                           Msg->lParam );

        ERR("KeyboardMessage WH_KEYBOARD Call Hook return!\n");

        *RemoveMessages = TRUE;

        Ret = FALSE;
    }

    if (pWnd && Ret && *RemoveMessages && bKeyUpDown && !(pti->TIF_flags & TIF_DISABLEIME))
    {
        ImmRet = IntImmProcessKey(pti->MessageQueue, pWnd, uMsg, Msg->wParam, Msg->lParam);
        if (ImmRet)
        {
            if ( ImmRet & (IPHK_HOTKEY|IPHK_SKIPTHISKEY) )
            {
               ImmRet = 0;
            }
            if ( ImmRet & IPHK_PROCESSBYIME )
            {
               Msg->wParam = VK_PROCESSKEY;
            }
        }
    }
Exit:
    if (pWnd) UserDerefObjectCo(pWnd);
    return Ret;
}

BOOL co_IntProcessHardwareMessage(MSG* Msg, BOOL* RemoveMessages, BOOL* NotForUs, LONG_PTR ExtraInfo, UINT first, UINT last)
{
    if ( IS_MOUSE_MESSAGE(Msg->message))
    {
        return co_IntProcessMouseMessage(Msg, RemoveMessages, NotForUs, ExtraInfo, first, last);
    }
    else if ( IS_KBD_MESSAGE(Msg->message))
    {
        return co_IntProcessKeyboardMessage(Msg, RemoveMessages);
    }

    return TRUE;
}

/* check whether a message filter contains at least one potential hardware message */
static INT FASTCALL
filter_contains_hw_range( UINT first, UINT last )
{
   /* hardware message ranges are (in numerical order):
    *   WM_NCMOUSEFIRST .. WM_NCMOUSELAST
    *   WM_KEYFIRST .. WM_KEYLAST
    *   WM_MOUSEFIRST .. WM_MOUSELAST
    */
    if (!last) --last;
    if (last < WM_NCMOUSEFIRST) return 0;
    if (first > WM_NCMOUSELAST && last < WM_KEYFIRST) return 0;
    if (first > WM_KEYLAST && last < WM_MOUSEFIRST) return 0;
    if (first > WM_MOUSELAST) return 0;
    return 1;
}

/* check whether message is in the range of mouse messages */
static inline BOOL is_mouse_message( UINT message )
{
    return ( //( message >= WM_NCMOUSEFIRST && message <= WM_NCMOUSELAST )   || This seems to break tests...
             ( message >= WM_MOUSEFIRST   && message <= WM_MOUSELAST )     ||
             ( message >= WM_XBUTTONDOWN  && message <= WM_XBUTTONDBLCLK ) ||
             ( message >= WM_MBUTTONDOWN  && message <= WM_MBUTTONDBLCLK ) ||
             ( message >= WM_LBUTTONDOWN  && message <= WM_RBUTTONDBLCLK ) );
}