/* msgqueue.split3.c — #include fragment of msgqueue.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original msgqueue.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL APIENTRY
co_MsqPeekHardwareMessage(IN PTHREADINFO pti,
                         IN BOOL Remove,
                         IN PWND Window,
                         IN UINT MsgFilterLow,
                         IN UINT MsgFilterHigh,
                         IN UINT QSflags,
                         OUT MSG* pMsg)
{
   BOOL AcceptMessage, NotForUs;
   PUSER_MESSAGE CurrentMessage;
   PLIST_ENTRY ListHead;
   MSG msg;
   ULONG_PTR idSave;
   DWORD QS_Flags;
   LONG_PTR ExtraInfo;
   MSG clk_msg;
   BOOL Ret = FALSE;
   PUSER_MESSAGE_QUEUE MessageQueue = pti->MessageQueue;

   if (!filter_contains_hw_range( MsgFilterLow, MsgFilterHigh )) return FALSE;

   ListHead = MessageQueue->HardwareMessagesListHead.Flink;

   if (IsListEmpty(ListHead)) return FALSE;

   if (!MessageQueue->ptiSysLock)
   {
      MessageQueue->ptiSysLock = pti;
      pti->pcti->CTI_flags |= CTI_THREADSYSLOCK;
   }

   if (MessageQueue->ptiSysLock != pti)
   {
      ERR("Thread Q is locked to ptiSysLock 0x%p pti 0x%p\n",MessageQueue->ptiSysLock,pti);
      return FALSE;
   }

   while (ListHead != &MessageQueue->HardwareMessagesListHead)
   {
      CurrentMessage = CONTAINING_RECORD(ListHead, USER_MESSAGE, ListEntry);
      ListHead = ListHead->Flink;

      if (MessageQueue->idSysPeek == (ULONG_PTR)CurrentMessage)
      {
         TRACE("Skip this message due to it is in play!\n");
         continue;
      }
/*
 MSDN:
 1: any window that belongs to the current thread, and any messages on the current thread's message queue whose hwnd value is NULL.
 2: retrieves only messages on the current thread's message queue whose hwnd value is NULL.
 3: handle to the window whose messages are to be retrieved.
 */
      if ( ( !Window || // 1
            ( Window == PWND_BOTTOM && CurrentMessage->Msg.hwnd == NULL ) || // 2
            ( Window != PWND_BOTTOM && UserHMGetHandle(Window) == CurrentMessage->Msg.hwnd ) || // 3
            ( is_mouse_message(CurrentMessage->Msg.message) ) ) && // Null window for anything mouse.
            ( CurrentMessage->QS_Flags & QSflags ) )
      {
         idSave = MessageQueue->idSysPeek;
         MessageQueue->idSysPeek = (ULONG_PTR)CurrentMessage;

         msg = CurrentMessage->Msg;
         ExtraInfo = CurrentMessage->ExtraInfo;
         QS_Flags = CurrentMessage->QS_Flags;
         clk_msg = MessageQueue->msgDblClk;

         NotForUs = FALSE;

         UpdateKeyStateFromMsg(MessageQueue, &msg);
         AcceptMessage = co_IntProcessHardwareMessage(&msg, &Remove, &NotForUs, ExtraInfo, MsgFilterLow, MsgFilterHigh);

         if (!NotForUs && (MsgFilterLow != 0 || MsgFilterHigh != 0))
         {
             /* Don't return message if not in range */
             if (msg.message < MsgFilterLow || msg.message > MsgFilterHigh)
             {
                 MessageQueue->msgDblClk = clk_msg;
                 MessageQueue->idSysPeek = idSave;
                 continue;
             }
         }

         if (Remove)
         {
             if (CurrentMessage->pti != NULL && (MessageQueue->idSysPeek == (ULONG_PTR)CurrentMessage))
             {
                MsqDestroyMessage(CurrentMessage);
             }
             ClearMsgBitsMask(pti, QS_Flags);
         }

         MessageQueue->idSysPeek = idSave;

         if (NotForUs)
         {
            Ret = FALSE;
            break;
         }

         if (AcceptMessage)
         {
            *pMsg = msg;
            // Fix all but one wine win:test_GetMessagePos WM_TIMER tests. See PostTimerMessages.
            if (!RtlEqualMemory(&pti->ptLast, &msg.pt, sizeof(POINT)))
            {
               pti->TIF_flags |= TIF_MSGPOSCHANGED;
            }
            pti->timeLast = msg.time;
            pti->ptLast   = msg.pt;
            MessageQueue->ExtraInfo = ExtraInfo;
            Ret = TRUE;
            break;
         }
      }
   }

   MessageQueue->ptiSysLock = NULL;
   pti->pcti->CTI_flags &= ~CTI_THREADSYSLOCK;
   return Ret;
}

BOOLEAN APIENTRY
MsqPeekMessage(IN PTHREADINFO pti,
                  IN BOOLEAN Remove,
                  IN PWND Window,
                  IN UINT MsgFilterLow,
                  IN UINT MsgFilterHigh,
                  IN UINT QSflags,
                  OUT LONG_PTR *ExtraInfo,
                  OUT DWORD *dwQEvent,
                  OUT PMSG Message)
{
   PUSER_MESSAGE CurrentMessage;
   PLIST_ENTRY ListHead;
   DWORD QS_Flags;
   BOOL Ret = FALSE;

   ListHead = pti->PostedMessagesListHead.Flink;

   if (IsListEmpty(ListHead)) return FALSE;

   while(ListHead != &pti->PostedMessagesListHead)
   {
      CurrentMessage = CONTAINING_RECORD(ListHead, USER_MESSAGE, ListEntry);
      ListHead = ListHead->Flink;
/*
 MSDN:
 1: any window that belongs to the current thread, and any messages on the current thread's message queue whose hwnd value is NULL.
 2: retrieves only messages on the current thread's message queue whose hwnd value is NULL.
 3: handle to the window whose messages are to be retrieved.
 */
      if ( ( !Window || // 1
            ( Window == PWND_BOTTOM && CurrentMessage->Msg.hwnd == NULL ) || // 2
            ( Window != PWND_BOTTOM && UserHMGetHandle(Window) == CurrentMessage->Msg.hwnd ) ) && // 3
            ( ( ( MsgFilterLow == 0 && MsgFilterHigh == 0 ) && CurrentMessage->QS_Flags & QSflags ) ||
              ( MsgFilterLow <= CurrentMessage->Msg.message && MsgFilterHigh >= CurrentMessage->Msg.message ) ) )
      {
         *Message   = CurrentMessage->Msg;
         *ExtraInfo = CurrentMessage->ExtraInfo;
         QS_Flags   = CurrentMessage->QS_Flags;
         if (dwQEvent) *dwQEvent = CurrentMessage->dwQEvent;

         if (Remove)
         {
             if (CurrentMessage->pti != NULL)
             {
                MsqDestroyMessage(CurrentMessage);
             }
             ClearMsgBitsMask(pti, QS_Flags);
         }
         Ret = TRUE;
         break;
      }
   }

   return Ret;
}

NTSTATUS FASTCALL
co_MsqWaitForNewMessages(PTHREADINFO pti, PWND WndFilter,
                         UINT MsgFilterMin, UINT MsgFilterMax)
{
   NTSTATUS ret = STATUS_SUCCESS;

   // Post mouse moves before waiting for messages.
   if (pti->MessageQueue->QF_flags & QF_MOUSEMOVED)
   {
      IntCoalesceMouseMove(pti);
   }

   UserLeaveCo();

   ZwYieldExecution(); // Let someone else run!

   ret = KeWaitForSingleObject( pti->pEventQueueServer,
                                UserRequest,
                                UserMode,
                                FALSE,
                                NULL );
   UserEnterCo();
   if ( ret == STATUS_USER_APC )
   {
      TRACE("MWFNW User APC\n");
      co_IntDeliverUserAPC();
   }
   return ret;
}

BOOL FASTCALL
MsqIsHung(PTHREADINFO pti, DWORD TimeOut)
{
    DWORD dwTimeStamp = EngGetTickCount32();
    if (dwTimeStamp - pti->pcti->timeLastRead > TimeOut &&
       !(pti->pcti->fsWakeMask & QS_INPUT) &&
       !PsGetThreadFreezeCount(pti->pEThread) &&
       !(pti->ppi->W32PF_flags & W32PF_APPSTARTING))
    {
        TRACE("\nMsqIsHung(pti %p, TimeOut %lu)\n"
            "pEThread %p, ThreadsProcess %p, ImageFileName '%s'\n"
            "dwTimeStamp = %lu\n"
            "pti->pcti->timeLastRead = %lu\n"
            "pti->timeLast = %lu\n"
            "PsGetThreadFreezeCount(pti->pEThread) = %lu\n",
            pti, TimeOut,
            pti->pEThread,
            pti->pEThread ? pti->pEThread->ThreadsProcess : NULL,
            (pti->pEThread && pti->pEThread->ThreadsProcess)
                ? pti->pEThread->ThreadsProcess->ImageFileName : "(None)",
            dwTimeStamp,
            pti->pcti->timeLastRead,
            pti->timeLast,
            PsGetThreadFreezeCount(pti->pEThread));

        return TRUE;
    }

   return FALSE;
}

BOOL FASTCALL
IsThreadSuspended(PTHREADINFO pti)
{
   if (pti->pEThread)
   {
      BOOL Ret = TRUE;
      if (!(pti->pEThread->Tcb.SuspendCount) && !PsGetThreadFreezeCount(pti->pEThread)) Ret = FALSE;
      return Ret;
   }
   return FALSE;
}

VOID
CALLBACK
HungAppSysTimerProc(HWND hwnd, UINT uMsg, UINT_PTR idEvent, DWORD dwTime)
{
   DoTheScreenSaver();
   TRACE("HungAppSysTimerProc\n");
   // Process list of windows that are hung and waiting.
}

BOOLEAN FASTCALL
MsqInitializeMessageQueue(PTHREADINFO pti, PUSER_MESSAGE_QUEUE MessageQueue)
{
   InitializeListHead(&MessageQueue->HardwareMessagesListHead); // Keep here!
   MessageQueue->spwndFocus = NULL;
   MessageQueue->iCursorLevel = 0;
   MessageQueue->CursorObject = SYSTEMCUR(WAIT); // See test_initial_cursor.
   if (MessageQueue->CursorObject)
   {
      TRACE("Default cursor hcur %p\n",UserHMGetHandle(MessageQueue->CursorObject));
      UserReferenceObject(MessageQueue->CursorObject);
   }
   RtlCopyMemory(MessageQueue->afKeyState, gafAsyncKeyState, sizeof(gafAsyncKeyState));
   MessageQueue->ptiMouse = pti;
   MessageQueue->ptiKeyboard = pti;
   MessageQueue->cThreads++;

   return TRUE;
}

VOID FASTCALL
MsqCleanupThreadMsgs(PTHREADINFO pti)
{
   PLIST_ENTRY CurrentEntry;
   PUSER_MESSAGE CurrentMessage;
   PUSER_SENT_MESSAGE CurrentSentMessage;

   TRACE("MsqCleanupThreadMsgs %p\n",pti);

   // Clear it all out.
   if (pti->pcti)
   {
       pti->pcti->fsWakeBits = 0;
       pti->pcti->fsChangeBits = 0;
   }

   pti->nCntsQBits[QSRosKey] = 0;
   pti->nCntsQBits[QSRosMouseMove] = 0;
   pti->nCntsQBits[QSRosMouseButton] = 0;
   pti->nCntsQBits[QSRosPostMessage] = 0;
   pti->nCntsQBits[QSRosSendMessage] = 0;
   pti->nCntsQBits[QSRosHotKey] = 0;
   pti->nCntsQBits[QSRosEvent] = 0;

   /* cleanup posted messages */
   while (!IsListEmpty(&pti->PostedMessagesListHead))
   {
      CurrentEntry = pti->PostedMessagesListHead.Flink;
      CurrentMessage = CONTAINING_RECORD(CurrentEntry, USER_MESSAGE, ListEntry);
      ERR("Thread Cleanup Post Messages %p\n",CurrentMessage);
      if (CurrentMessage->dwQEvent)
      {
         if (CurrentMessage->dwQEvent == POSTEVENT_NWE)
         {
            ExFreePoolWithTag( (PVOID)CurrentMessage->ExtraInfo, TAG_HOOK);
         }
      }
      MsqDestroyMessage(CurrentMessage);
   }

   /* remove the messages that have not yet been dispatched */
   while (!IsListEmpty(&pti->SentMessagesListHead))
   {
      CurrentEntry = pti->SentMessagesListHead.Flink;
      CurrentSentMessage = CONTAINING_RECORD(CurrentEntry, USER_SENT_MESSAGE, ListEntry);

      ERR("Thread Cleanup Sent Messages %p\n",CurrentSentMessage);

      /* wake the sender's thread */
      if (CurrentSentMessage->pkCompletionEvent != NULL)
      {
         KeSetEvent(CurrentSentMessage->pkCompletionEvent, IO_NO_INCREMENT, FALSE);
      }

      if (CurrentSentMessage->HasPackedLParam)
      {
         if (CurrentSentMessage->Msg.lParam)
            ExFreePool((PVOID)CurrentSentMessage->Msg.lParam);
      }

      /* free the message */
      FreeUserMessage(CurrentSentMessage);
   }

   // Process Trouble Message List
   if (!IsListEmpty(&usmList))
   {
      CurrentEntry = usmList.Flink;
      while (CurrentEntry != &usmList)
      {
         CurrentSentMessage = CONTAINING_RECORD(CurrentEntry, USER_SENT_MESSAGE, ListEntry);
         CurrentEntry = CurrentEntry->Flink;

         TRACE("Found troubled messages %p on the list\n",CurrentSentMessage);

         if ( pti == CurrentSentMessage->ptiReceiver )
         {
            if (CurrentSentMessage->HasPackedLParam)
            {
               if (CurrentSentMessage->Msg.lParam)
                  ExFreePool((PVOID)CurrentSentMessage->Msg.lParam);
            }

            /* free the message */
            FreeUserMessage(CurrentSentMessage);
         }
         else if ( pti == CurrentSentMessage->ptiSender ||
                   pti == CurrentSentMessage->ptiCallBackSender )
         {
            // Determine whether this message is being processed or not.
            if ((CurrentSentMessage->flags & (SMF_RECEIVERBUSY|SMF_RECEIVEDMESSAGE)) != SMF_RECEIVEDMESSAGE)
            {
               CurrentSentMessage->flags |= SMF_RECEIVERFREE;
            }

            if (!(CurrentSentMessage->flags & SMF_RECEIVERFREE))
            {

               if (CurrentSentMessage->HasPackedLParam)
               {
                  if (CurrentSentMessage->Msg.lParam)
                     ExFreePool((PVOID)CurrentSentMessage->Msg.lParam);
               }

               /* free the message */
               FreeUserMessage(CurrentSentMessage);
            }
         }
      }
   }
}

VOID FASTCALL
MsqCleanupMessageQueue(PTHREADINFO pti)
{
   PUSER_MESSAGE_QUEUE MessageQueue;
   PLIST_ENTRY CurrentEntry;
   PUSER_MESSAGE CurrentMessage;

   MessageQueue = pti->MessageQueue;
   MessageQueue->cThreads--;

   if (MessageQueue->cThreads)
   {
      if (MessageQueue->ptiSysLock == pti) MessageQueue->ptiSysLock = NULL;
   }

   if (MessageQueue->cThreads == 0) //// Fix a crash related to CORE-10471 testing.
   {
      /* cleanup posted messages */
      while (!IsListEmpty(&MessageQueue->HardwareMessagesListHead))
      {
         CurrentEntry = MessageQueue->HardwareMessagesListHead.Flink;
         CurrentMessage = CONTAINING_RECORD(CurrentEntry, USER_MESSAGE, ListEntry);
         ERR("MQ Cleanup Post Messages %p\n",CurrentMessage);
         MsqDestroyMessage(CurrentMessage);
      }
   } ////

   if (MessageQueue->CursorObject)
   {
       PCURICON_OBJECT pCursor = MessageQueue->CursorObject;

       /* Change to another cursor if we going to dereference current one
          Note: we can't use UserSetCursor because it uses current thread
                message queue instead of queue given for cleanup */
       if (IntGetSysCursorInfo()->CurrentCursorObject == pCursor)
       {
           HDC hdcScreen;

           /* Get the screen DC */
           hdcScreen = IntGetScreenDC();
           if (hdcScreen)
               GreMovePointer(hdcScreen, -1, -1);
           IntGetSysCursorInfo()->CurrentCursorObject = NULL;
       }

       TRACE("DereferenceObject pCursor\n");
       UserDereferenceObject(pCursor);
   }

   if (gpqForeground == MessageQueue)
   {
      IntSetFocusMessageQueue(NULL);
   }
   if (gpqForegroundPrev == MessageQueue)
   {
      gpqForegroundPrev = NULL;
   }
   if (gpqCursor == MessageQueue)
   {
      gpqCursor = NULL;
   }
}

PUSER_MESSAGE_QUEUE FASTCALL
MsqCreateMessageQueue(PTHREADINFO pti)
{
   PUSER_MESSAGE_QUEUE MessageQueue;

   MessageQueue = ExAllocatePoolWithTag(NonPagedPool,
                                        sizeof(*MessageQueue),
                                        USERTAG_Q);

   if (!MessageQueue)
   {
      return NULL;
   }

   RtlZeroMemory(MessageQueue, sizeof(*MessageQueue));
   /* hold at least one reference until it'll be destroyed */
   IntReferenceMessageQueue(MessageQueue);
   /* initialize the queue */
   if (!MsqInitializeMessageQueue(pti, MessageQueue))
   {
      IntDereferenceMessageQueue(MessageQueue);
      return NULL;
   }

   return MessageQueue;
}

VOID FASTCALL
MsqDestroyMessageQueue(_In_ PTHREADINFO pti)
{
   PDESKTOP desk;
   PUSER_MESSAGE_QUEUE MessageQueue = pti->MessageQueue;

   NT_ASSERT(MessageQueue != NULL);
   MessageQueue->QF_flags |= QF_INDESTROY;

   /* remove the message queue from any desktops */
   if ((desk = InterlockedExchangePointer((PVOID*)&MessageQueue->Desktop, 0)))
   {
      (void)InterlockedExchangePointer((PVOID*)&desk->ActiveMessageQueue, 0);
      IntDereferenceMessageQueue(MessageQueue);
   }

   /* clean it up */
   MsqCleanupMessageQueue(pti);

   /* decrease the reference counter, if it hits zero, the queue will be freed */
   _PRAGMA_WARNING_SUPPRESS(__WARNING_USING_UNINIT_VAR);
   IntDereferenceMessageQueue(MessageQueue);
}

LPARAM FASTCALL
MsqSetMessageExtraInfo(LPARAM lParam)
{
   LPARAM Ret;
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE MessageQueue;

   pti = PsGetCurrentThreadWin32Thread();
   MessageQueue = pti->MessageQueue;
   if(!MessageQueue)
   {
      return 0;
   }

   Ret = MessageQueue->ExtraInfo;
   MessageQueue->ExtraInfo = lParam;

   return Ret;
}

LPARAM FASTCALL
MsqGetMessageExtraInfo(VOID)
{
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE MessageQueue;

   pti = PsGetCurrentThreadWin32Thread();
   MessageQueue = pti->MessageQueue;
   if(!MessageQueue)
   {
      return 0;
   }

   return MessageQueue->ExtraInfo;
}

// ReplyMessage is called by the thread receiving the window message.
BOOL FASTCALL
co_MsqReplyMessage( LRESULT lResult )
{
   PUSER_SENT_MESSAGE Message;
   PTHREADINFO pti;

   pti = PsGetCurrentThreadWin32Thread();
   Message = pti->pusmCurrent;

   if (!Message) return FALSE;

   if (Message->QS_Flags & QS_SMRESULT) return FALSE;

   //     SendMessageXxx  || Callback msg and not a notify msg
   if (Message->ptiSender || Message->CompletionCallback)
   {
      Message->lResult = lResult;
      Message->QS_Flags |= QS_SMRESULT;
   // See co_MsqDispatchOneSentMessage, change bits already accounted for and cleared and this msg is going away..
   }
   return TRUE;
}

HWND FASTCALL
MsqSetStateWindow(PTHREADINFO pti, ULONG Type, HWND hWnd)
{
   HWND Prev;
   PUSER_MESSAGE_QUEUE MessageQueue;

   MessageQueue = pti->MessageQueue;

   switch(Type)
   {
      case MSQ_STATE_CAPTURE:
         Prev = MessageQueue->spwndCapture ? UserHMGetHandle(MessageQueue->spwndCapture) : 0;
         MessageQueue->spwndCapture = ValidateHwndNoErr(hWnd);
         return Prev;
      case MSQ_STATE_ACTIVE:
         Prev = MessageQueue->spwndActive ? UserHMGetHandle(MessageQueue->spwndActive) : 0;
         MessageQueue->spwndActive = ValidateHwndNoErr(hWnd);
         return Prev;
      case MSQ_STATE_FOCUS:
         Prev = MessageQueue->spwndFocus ? UserHMGetHandle(MessageQueue->spwndFocus) : 0;
         MessageQueue->spwndFocus = ValidateHwndNoErr(hWnd);
         return Prev;
      case MSQ_STATE_MENUOWNER:
         Prev = MessageQueue->MenuOwner;
         MessageQueue->MenuOwner = hWnd;
         return Prev;
      case MSQ_STATE_MOVESIZE:
         Prev = MessageQueue->MoveSize;
         MessageQueue->MoveSize = hWnd;
         return Prev;
      case MSQ_STATE_CARET:
         Prev = MessageQueue->CaretInfo.hWnd;
         MessageQueue->CaretInfo.hWnd = hWnd;
         return Prev;
   }

   return NULL;
}

VOID FASTCALL
MsqReleaseModifierKeys(PUSER_MESSAGE_QUEUE MessageQueue)
{
    WORD ModifierKeys[] = { VK_LCONTROL, VK_RCONTROL, VK_CONTROL,
                            VK_LMENU,    VK_RMENU,    VK_MENU,
                            VK_LSHIFT,   VK_RSHIFT,   VK_SHIFT };
    UINT i;

    for (i = 0; i < _countof(ModifierKeys); ++i)
    {
        if (IS_KEY_DOWN(MessageQueue->afKeyState, ModifierKeys[i]))
            SET_KEY_DOWN(MessageQueue->afKeyState, ModifierKeys[i], FALSE);
    }
}

SHORT
APIENTRY
NtUserGetKeyState(INT key)
{
   DWORD Ret;

   UserEnterShared();

   Ret = UserGetKeyState(key);

   UserLeave();

   return (SHORT)Ret;
}


DWORD
APIENTRY
NtUserGetKeyboardState(LPBYTE lpKeyState)
{
   DWORD i, ret = TRUE;
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE MessageQueue;
   NTSTATUS Status;

   UserEnterShared();

   pti = PsGetCurrentThreadWin32Thread();
   MessageQueue = pti->MessageQueue;

   Status = SafeProbeForWrite(lpKeyState, 256 * sizeof(BYTE), 1);
   if (!NT_SUCCESS(Status))
   {
       SetLastNtError(Status);
       ret = FALSE;
   }
   else
   {
       /* Probe and copy key state to an array */
       for (i = 0; i < 256; ++i)
       {
           lpKeyState[i] = 0;
           if (IS_KEY_DOWN(MessageQueue->afKeyState, i))
               lpKeyState[i] |= KS_DOWN_BIT;
           if (IS_KEY_LOCKED(MessageQueue->afKeyState, i))
               lpKeyState[i] |= KS_LOCK_BIT;
       }
   }

   UserLeave();

   return ret;
}


BOOL
APIENTRY
NtUserSetKeyboardState(LPBYTE pKeyState)
{
   UINT i;
   BOOL bRet = TRUE;
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE MessageQueue;
   NTSTATUS Status;

   UserEnterExclusive();

   pti = PsGetCurrentThreadWin32Thread();
   MessageQueue = pti->MessageQueue;

   Status = SafeProbeForRead(pKeyState, 256 * sizeof(BYTE), 1);
   if (!NT_SUCCESS(Status))
   {
       SetLastNtError(Status);
       bRet = FALSE;
   }
   else
   {
       for (i = 0; i < 256; ++i)
       {
            SET_KEY_DOWN(MessageQueue->afKeyState, i, pKeyState[i] & KS_DOWN_BIT);
            SET_KEY_LOCKED(MessageQueue->afKeyState, i, pKeyState[i] & KS_LOCK_BIT);
       }
   }

   UserLeave();

   return bRet;
}


/* EOF */
