/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS Win32k subsystem
 * PURPOSE:          Message queues
 * FILE:             win32ss/user/ntuser/msgqueue.c
 * PROGRAMER:        Casper S. Hornstrup (chorns@users.sourceforge.net)
                     Alexandre Julliard
                     Maarten Lankhorst
 */

#include <win32k.h>
#include <safe_user_io.h>
DBG_DEFAULT_CHANNEL(UserMsgQ);

/* GLOBALS *******************************************************************/

static PPAGED_LOOKASIDE_LIST pgMessageLookasideList;
static PPAGED_LOOKASIDE_LIST pgSendMsgLookasideList;
INT PostMsgCount = 0;
INT SendMsgCount = 0;
PUSER_MESSAGE_QUEUE gpqCursor;
ULONG_PTR gdwMouseMoveExtraInfo = 0;
DWORD gdwMouseMoveTimeStamp = 0;
LIST_ENTRY usmList;

/* FUNCTIONS *****************************************************************/

CODE_SEG("INIT")
NTSTATUS
NTAPI
MsqInitializeImpl(VOID)
{
   // Setup Post Messages
   pgMessageLookasideList = ExAllocatePoolWithTag(NonPagedPool, sizeof(PAGED_LOOKASIDE_LIST), TAG_USRMSG);
   if (!pgMessageLookasideList)
      return STATUS_NO_MEMORY;
   ExInitializePagedLookasideList(pgMessageLookasideList,
                                  NULL,
                                  NULL,
                                  0,
                                  sizeof(USER_MESSAGE),
                                  TAG_USRMSG,
                                  256);
   // Setup Send Messages
   pgSendMsgLookasideList = ExAllocatePoolWithTag(NonPagedPool, sizeof(PAGED_LOOKASIDE_LIST), TAG_USRMSG);
   if (!pgSendMsgLookasideList)
      return STATUS_NO_MEMORY;
   ExInitializePagedLookasideList(pgSendMsgLookasideList,
                                  NULL,
                                  NULL,
                                  0,
                                  sizeof(USER_SENT_MESSAGE),
                                  TAG_USRMSG,
                                  16);

   InitializeListHead(&usmList);

   return(STATUS_SUCCESS);
}

PWND FASTCALL
IntTopLevelWindowFromPoint(INT x, INT y)
{
    PWND pWnd, pwndDesktop;

    /* Get the desktop window */
    pwndDesktop = UserGetDesktopWindow();
    if (!pwndDesktop)
        return NULL;

    /* Loop all top level windows */
    for (pWnd = pwndDesktop->spwndChild;
         pWnd != NULL;
         pWnd = pWnd->spwndNext)
    {
        if (pWnd->state2 & WNDS2_INDESTROY || pWnd->state & WNDS_DESTROYED)
        {
            TRACE("The Window is in DESTROY!\n");
            continue;
        }

        if ((pWnd->style & WS_VISIBLE) &&
            (pWnd->ExStyle & (WS_EX_LAYERED|WS_EX_TRANSPARENT)) != (WS_EX_LAYERED|WS_EX_TRANSPARENT) &&
            IntPtInWindow(pWnd, x, y))
            return pWnd;
    }

    /* Window has not been found */
    return pwndDesktop;
}

PCURICON_OBJECT
FASTCALL
UserSetCursor(
    PCURICON_OBJECT NewCursor,
    BOOL ForceChange)
{
    PCURICON_OBJECT OldCursor;
    HDC hdcScreen;
    PTHREADINFO pti;
    PUSER_MESSAGE_QUEUE MessageQueue;
    PWND pWnd;

    pti = PsGetCurrentThreadWin32Thread();
    MessageQueue = pti->MessageQueue;

    OldCursor = MessageQueue->CursorObject;

    /* Check if cursors are different */
    if (OldCursor == NewCursor)
        return OldCursor;

    /* Update cursor for this message queue */
    MessageQueue->CursorObject = NewCursor;

    /* If cursor is not visible we have nothing to do */
    if (MessageQueue->iCursorLevel < 0)
        return OldCursor;

    // Fixes the error message "Not the same cursor!".
    if (gpqCursor == NULL)
    {
       gpqCursor = MessageQueue;
    }

    /* Update cursor if this message queue controls it */
    pWnd = IntTopLevelWindowFromPoint(gpsi->ptCursor.x, gpsi->ptCursor.y);
    if (pWnd && pWnd->head.pti->MessageQueue == MessageQueue)
    {
       /* Get the screen DC */
        if (!(hdcScreen = IntGetScreenDC()))
        {
            return NULL;
        }

        if (NewCursor)
        {
            /* Call GDI to set the new screen cursor */
            PCURICON_OBJECT CursorFrame = NewCursor;
            if(NewCursor->CURSORF_flags & CURSORF_ACON)
            {
                FIXME("Should animate the cursor, using only the first frame now.\n");
                CursorFrame = ((PACON)NewCursor)->aspcur[0];
            }
            GreSetPointerShape(hdcScreen,
                               CursorFrame->hbmAlpha ? NULL : NewCursor->hbmMask,
                               CursorFrame->hbmAlpha ? NewCursor->hbmAlpha : NewCursor->hbmColor,
                               CursorFrame->xHotspot,
                               CursorFrame->yHotspot,
                               gpsi->ptCursor.x,
                               gpsi->ptCursor.y,
                               CursorFrame->hbmAlpha ? SPS_ALPHA : 0);
        }
        else /* Note: OldCursor != NewCursor so we have to hide cursor */
        {
            /* Remove the cursor */
            GreMovePointer(hdcScreen, -1, -1);
            TRACE("Removing pointer!\n");
        }
        IntGetSysCursorInfo()->CurrentCursorObject = NewCursor;
    }

    /* Return the old cursor */
    return OldCursor;
}

/* Called from NtUserCallOneParam with Routine ONEPARAM_ROUTINE_SHOWCURSOR
 * User32 macro NtUserShowCursor */
int UserShowCursor(BOOL bShow)
{
    HDC hdcScreen;
    PTHREADINFO pti;
    PUSER_MESSAGE_QUEUE MessageQueue;
    PWND pWnd;

    if (!(hdcScreen = IntGetScreenDC()))
    {
        return -1; /* No mouse */
    }

    pti = PsGetCurrentThreadWin32Thread();
    MessageQueue = pti->MessageQueue;

    /* Update counter */
    MessageQueue->iCursorLevel += bShow ? 1 : -1;
    pti->iCursorLevel += bShow ? 1 : -1;

    /* Check for trivial cases */
    if ((bShow && MessageQueue->iCursorLevel != 0) ||
        (!bShow && MessageQueue->iCursorLevel != -1))
    {
        /* Note: w don't update global info here because it is used only
          internally to check if cursor is visible */
        return MessageQueue->iCursorLevel;
    }

    /* Check if cursor is above window owned by this MessageQueue */
    pWnd = IntTopLevelWindowFromPoint(gpsi->ptCursor.x, gpsi->ptCursor.y);
    if (pWnd && pWnd->head.pti->MessageQueue == MessageQueue)
    {
        if (bShow)
        {
            /* Show the pointer */
            GreMovePointer(hdcScreen, gpsi->ptCursor.x, gpsi->ptCursor.y);
            TRACE("Showing pointer!\n");
        }
        else
        {
            /* Remove the pointer */
            GreMovePointer(hdcScreen, -1, -1);
            TRACE("Removing pointer!\n");
        }

        /* Update global info */
        IntGetSysCursorInfo()->ShowingCursor = MessageQueue->iCursorLevel;
    }

    return MessageQueue->iCursorLevel;
}

DWORD FASTCALL
UserGetKeyState(DWORD dwKey)
{
   DWORD dwRet = 0;
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE MessageQueue;

   pti = PsGetCurrentThreadWin32Thread();
   MessageQueue = pti->MessageQueue;

   if (dwKey < 0x100)
   {
       if (IS_KEY_DOWN(MessageQueue->afKeyState, dwKey))
           dwRet |= 0xFF80; // If down, windows returns 0xFF80.
       if (IS_KEY_LOCKED(MessageQueue->afKeyState, dwKey))
           dwRet |= 0x1;
   }
   else
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
   }
   return dwRet;
}

/* change the input key state for a given key */
static VOID
UpdateKeyState(PUSER_MESSAGE_QUEUE MessageQueue, WORD wVk, BOOL bIsDown)
{
    TRACE("UpdateKeyState wVk: %u, bIsDown: %d\n", wVk, bIsDown);

    if (bIsDown)
    {
        /* If it's first key down event, xor lock bit */
        if (!IS_KEY_DOWN(MessageQueue->afKeyState, wVk))
            SET_KEY_LOCKED(MessageQueue->afKeyState, wVk, !IS_KEY_LOCKED(MessageQueue->afKeyState, wVk));

        SET_KEY_DOWN(MessageQueue->afKeyState, wVk, TRUE);
        MessageQueue->afKeyRecentDown[wVk / 8] |= (1 << (wVk % 8));
    }
    else
        SET_KEY_DOWN(MessageQueue->afKeyState, wVk, FALSE);
}

/* update the input key state for a keyboard message */
static VOID
UpdateKeyStateFromMsg(PUSER_MESSAGE_QUEUE MessageQueue, MSG* msg)
{
    UCHAR key;
    BOOL down = FALSE;

    TRACE("UpdateKeyStateFromMsg message:%u\n", msg->message);

    switch (msg->message)
    {
    case WM_LBUTTONDOWN:
        down = TRUE;
        /* fall through */
    case WM_LBUTTONUP:
        UpdateKeyState(MessageQueue, VK_LBUTTON, down);
        break;
    case WM_MBUTTONDOWN:
        down = TRUE;
        /* fall through */
    case WM_MBUTTONUP:
        UpdateKeyState(MessageQueue, VK_MBUTTON, down);
        break;
    case WM_RBUTTONDOWN:
        down = TRUE;
        /* fall through */
    case WM_RBUTTONUP:
        UpdateKeyState(MessageQueue, VK_RBUTTON, down);
        break;
    case WM_XBUTTONDOWN:
        down = TRUE;
        /* fall through */
    case WM_XBUTTONUP:
        if (msg->wParam == XBUTTON1)
            UpdateKeyState(MessageQueue, VK_XBUTTON1, down);
        else if (msg->wParam == XBUTTON2)
            UpdateKeyState(MessageQueue, VK_XBUTTON2, down);
        break;
    case WM_KEYDOWN:
    case WM_SYSKEYDOWN:
        down = TRUE;
        /* fall through */
    case WM_KEYUP:
    case WM_SYSKEYUP:
        key = (UCHAR)msg->wParam;
        UpdateKeyState(MessageQueue, key, down);
        switch(key)
        {
        case VK_LCONTROL:
        case VK_RCONTROL:
            down = IS_KEY_DOWN(MessageQueue->afKeyState, VK_LCONTROL) || IS_KEY_DOWN(MessageQueue->afKeyState, VK_RCONTROL);
            UpdateKeyState(MessageQueue, VK_CONTROL, down);
            break;
        case VK_LMENU:
        case VK_RMENU:
            down = IS_KEY_DOWN(MessageQueue->afKeyState, VK_LMENU) || IS_KEY_DOWN(MessageQueue->afKeyState, VK_RMENU);
            UpdateKeyState(MessageQueue, VK_MENU, down);
            break;
        case VK_LSHIFT:
        case VK_RSHIFT:
            down = IS_KEY_DOWN(MessageQueue->afKeyState, VK_LSHIFT) || IS_KEY_DOWN(MessageQueue->afKeyState, VK_RSHIFT);
            UpdateKeyState(MessageQueue, VK_SHIFT, down);
            break;
        }
        break;
    }
}

/*
    Get down key states from the queue of prior processed input message key states.

    This fixes the left button dragging on the desktop and release sticking outline issue.
    USB Tablet pointer seems to stick the most and leaves the box outline displayed.
 */
WPARAM FASTCALL
MsqGetDownKeyState(PUSER_MESSAGE_QUEUE MessageQueue)
{
    WPARAM ret = 0;

    if (gspv.bMouseBtnSwap)
    {
       if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_RBUTTON)) ret |= MK_LBUTTON;
       if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_LBUTTON)) ret |= MK_RBUTTON;
    }
    else
    {
       if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_LBUTTON)) ret |= MK_LBUTTON;
       if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_RBUTTON)) ret |= MK_RBUTTON;
    }

    if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_MBUTTON))  ret |= MK_MBUTTON;
    if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_SHIFT))    ret |= MK_SHIFT;
    if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_CONTROL))  ret |= MK_CONTROL;
    if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_XBUTTON1)) ret |= MK_XBUTTON1;
    if (IS_KEY_DOWN(MessageQueue->afKeyState, VK_XBUTTON2)) ret |= MK_XBUTTON2;
    return ret;
}

HANDLE FASTCALL
IntMsqSetWakeMask(DWORD WakeMask)
{
   PTHREADINFO Win32Thread;
   HANDLE MessageEventHandle;
   DWORD dwFlags = HIWORD(WakeMask);

   Win32Thread = PsGetCurrentThreadWin32Thread();
   if (Win32Thread == NULL || Win32Thread->MessageQueue == NULL)
      return 0;

// Win32Thread->pEventQueueServer; IntMsqSetWakeMask returns Win32Thread->hEventQueueClient
   MessageEventHandle = Win32Thread->hEventQueueClient;

   if (Win32Thread->pcti)
   {
      if ( (Win32Thread->pcti->fsChangeBits & LOWORD(WakeMask)) ||
           ( (dwFlags & MWMO_INPUTAVAILABLE) && (Win32Thread->pcti->fsWakeBits & LOWORD(WakeMask)) ) )
      {
         ERR("Chg 0x%x Wake 0x%x Mask 0x%x\n",Win32Thread->pcti->fsChangeBits, Win32Thread->pcti->fsWakeBits, WakeMask);
         KeSetEvent(Win32Thread->pEventQueueServer, IO_NO_INCREMENT, FALSE); // Wake it up!
         return MessageEventHandle;
      }
   }

   IdlePing();

   return MessageEventHandle;
}

BOOL FASTCALL
IntMsqClearWakeMask(VOID)
{
   PTHREADINFO Win32Thread;

   Win32Thread = PsGetCurrentThreadWin32Thread();
   if (Win32Thread == NULL || Win32Thread->MessageQueue == NULL)
      return FALSE;
   // Very hacky, but that is what they do.
   Win32Thread->pcti->fsWakeBits = 0;

   IdlePong();

   return TRUE;
}

/*
   Due to the uncertainty of knowing what was set in our multilevel message queue,
   and even if the bits are all cleared. The same as cTimers/cPaintsReady.
   I think this is the best solution... (jt) */
VOID FASTCALL
MsqWakeQueue(PTHREADINFO pti, DWORD MessageBits, BOOL KeyEvent)
{
   PUSER_MESSAGE_QUEUE Queue;

   Queue = pti->MessageQueue;

   if (Queue->QF_flags & QF_INDESTROY)
   {
      ERR("This Message Queue is in Destroy!\n");
   }
   pti->pcti->fsWakeBits |= MessageBits;
   pti->pcti->fsChangeBits |= MessageBits;

   // Start bit accounting to help clear the main set of bits.
   if (MessageBits & QS_KEY)
   {
      pti->nCntsQBits[QSRosKey]++;
   }
   if (MessageBits & QS_MOUSE)
   {
      if (MessageBits & QS_MOUSEMOVE)   pti->nCntsQBits[QSRosMouseMove]++;
      if (MessageBits & QS_MOUSEBUTTON) pti->nCntsQBits[QSRosMouseButton]++;
   }
   if (MessageBits & QS_POSTMESSAGE) pti->nCntsQBits[QSRosPostMessage]++;
   if (MessageBits & QS_SENDMESSAGE) pti->nCntsQBits[QSRosSendMessage]++;
   if (MessageBits & QS_HOTKEY)      pti->nCntsQBits[QSRosHotKey]++;
   if (MessageBits & QS_EVENT)       pti->nCntsQBits[QSRosEvent]++;

   if (KeyEvent)
      KeSetEvent(pti->pEventQueueServer, IO_NO_INCREMENT, FALSE);
}

VOID FASTCALL
ClearMsgBitsMask(PTHREADINFO pti, UINT MessageBits)
{
   UINT ClrMask = 0;

   if (MessageBits & QS_KEY)
   {
      if (--pti->nCntsQBits[QSRosKey] == 0) ClrMask |= QS_KEY;
   }
   if (MessageBits & QS_MOUSEMOVE)
   {  // Account for tracking mouse moves..
      if (pti->nCntsQBits[QSRosMouseMove])
      {
         pti->nCntsQBits[QSRosMouseMove] = 0; // Throttle down count. Up to > 3:1 entries are ignored.
         ClrMask |= QS_MOUSEMOVE;
      }
   }
   if (MessageBits & QS_MOUSEBUTTON)
   {
      if (--pti->nCntsQBits[QSRosMouseButton] == 0) ClrMask |= QS_MOUSEBUTTON;
   }
   if (MessageBits & QS_POSTMESSAGE)
   {
      if (--pti->nCntsQBits[QSRosPostMessage] == 0) ClrMask |= QS_POSTMESSAGE;
   }
   if (MessageBits & QS_TIMER) // ReactOS hard coded.
   {  // Handle timer bits here.
      if ( pti->cTimersReady )
      {
         if (--pti->cTimersReady == 0) ClrMask |= QS_TIMER;
      }
   }
   if (MessageBits & QS_PAINT) // ReactOS hard coded.
   {  // Handle paint bits here.
      if ( pti->cPaintsReady )
      {
         if (--pti->cPaintsReady == 0) ClrMask |= QS_PAINT;
      }
   }
   if (MessageBits & QS_SENDMESSAGE)
   {
      if (--pti->nCntsQBits[QSRosSendMessage] == 0) ClrMask |= QS_SENDMESSAGE;
   }
   if (MessageBits & QS_HOTKEY)
   {
      if (--pti->nCntsQBits[QSRosHotKey] == 0) ClrMask |= QS_HOTKEY;
   }
   if (MessageBits & QS_EVENT)
   {
      if (--pti->nCntsQBits[QSRosEvent] == 0) ClrMask |= QS_EVENT;
   }

   pti->pcti->fsWakeBits &= ~ClrMask;
   pti->pcti->fsChangeBits &= ~ClrMask;
}

VOID FASTCALL
MsqIncPaintCountQueue(PTHREADINFO pti)
{
   pti->cPaintsReady++;
   MsqWakeQueue(pti, QS_PAINT, TRUE);
}

VOID FASTCALL
MsqDecPaintCountQueue(PTHREADINFO pti)
{
   ClearMsgBitsMask(pti, QS_PAINT);
}

/*
    Post the move or update the message still pending to be processed.
    Do not overload the queue with mouse move messages.
 */
VOID FASTCALL
MsqPostMouseMove(PTHREADINFO pti, MSG* Msg, LONG_PTR ExtraInfo)
{
    PUSER_MESSAGE Message;
    PLIST_ENTRY ListHead;
    PUSER_MESSAGE_QUEUE MessageQueue = pti->MessageQueue;

    ListHead = &MessageQueue->HardwareMessagesListHead;

    // Do nothing if empty.
    if (!IsListEmpty(ListHead->Flink))
    {
       // Look at the end of the list,
       Message = CONTAINING_RECORD(ListHead->Blink, USER_MESSAGE, ListEntry);

       // If the mouse move message is existing on the list,
       if (Message->Msg.message == WM_MOUSEMOVE)
       {
          // Overwrite the message with updated data!
          Message->Msg = *Msg;

          MsqWakeQueue(pti, QS_MOUSEMOVE, TRUE);
          return;
       }
    }

    MsqPostMessage(pti, Msg, TRUE, QS_MOUSEMOVE, 0, ExtraInfo);
}

/*
    Bring together the mouse move message.
    Named "Coalesce" from Amine email ;^) (jt).
 */
VOID FASTCALL
IntCoalesceMouseMove(PTHREADINFO pti)
{
    MSG Msg;

    // Force time stamp to update, keeping message time in sync.
    if (gdwMouseMoveTimeStamp == 0)
    {
        gdwMouseMoveTimeStamp = EngGetTickCount32();
    }

    // Build mouse move message.
    Msg.hwnd    = NULL;
    Msg.message = WM_MOUSEMOVE;
    Msg.wParam  = 0;
    Msg.lParam  = MAKELONG(gpsi->ptCursor.x, gpsi->ptCursor.y);
    Msg.time    = gdwMouseMoveTimeStamp;
    Msg.pt      = gpsi->ptCursor;

    // Post the move.
    MsqPostMouseMove(pti, &Msg, gdwMouseMoveExtraInfo);

    // Zero the time stamp.
    gdwMouseMoveTimeStamp = 0;

    // Clear flag since the move was posted.
    pti->MessageQueue->QF_flags &= ~QF_MOUSEMOVED;
}

VOID FASTCALL
co_MsqInsertMouseMessage(MSG* Msg, DWORD flags, ULONG_PTR dwExtraInfo, BOOL Hook)
{
   MSLLHOOKSTRUCT MouseHookData;
//   PDESKTOP pDesk;
   PWND pwnd, pwndDesktop;
   HDC hdcScreen;
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE MessageQueue;
   PSYSTEM_CURSORINFO CurInfo;

   Msg->time = EngGetTickCount32();

   MouseHookData.pt.x = LOWORD(Msg->lParam);
   MouseHookData.pt.y = HIWORD(Msg->lParam);
   switch (Msg->message)
   {
      case WM_MOUSEWHEEL:
         MouseHookData.mouseData = MAKELONG(0, GET_WHEEL_DELTA_WPARAM(Msg->wParam));
         break;
      case WM_XBUTTONDOWN:
      case WM_XBUTTONUP:
      case WM_XBUTTONDBLCLK:
      case WM_NCXBUTTONDOWN:
      case WM_NCXBUTTONUP:
      case WM_NCXBUTTONDBLCLK:
         MouseHookData.mouseData = MAKELONG(0, HIWORD(Msg->wParam));
         break;
      default:
         MouseHookData.mouseData = 0;
         break;
   }

   MouseHookData.flags = flags; // LLMHF_INJECTED
   MouseHookData.time = Msg->time;
   MouseHookData.dwExtraInfo = dwExtraInfo;

   /* If the hook procedure returned non zero, dont send the message */
   if (Hook)
   {
      if (co_HOOK_CallHooks(WH_MOUSE_LL, HC_ACTION, Msg->message, (LPARAM) &MouseHookData))
         return;
   }

   /* Get the desktop window */
   pwndDesktop = UserGetDesktopWindow();
   if (!pwndDesktop) return;
//   pDesk = pwndDesktop->head.rpdesk;

   /* Check if the mouse is captured */
   Msg->hwnd = IntGetCaptureWindow();
   if (Msg->hwnd != NULL)
   {
       pwnd = UserGetWindowObject(Msg->hwnd);
   }
   else
   {
       pwnd = IntTopLevelWindowFromPoint(Msg->pt.x, Msg->pt.y);
       if (pwnd) Msg->hwnd = UserHMGetHandle(pwnd);
   }

   hdcScreen = IntGetScreenDC();
   CurInfo = IntGetSysCursorInfo();

   /* Check if we found a window */
   if (Msg->hwnd != NULL && pwnd != NULL)
   {
       pti = pwnd->head.pti;
       MessageQueue = pti->MessageQueue;

       if (MessageQueue->QF_flags & QF_INDESTROY)
       {
          ERR("Mouse is over a Window with a Dead Message Queue!\n");
          return;
       }

       // Check to see if this is attached.
       if ( pti != MessageQueue->ptiMouse &&
            MessageQueue->cThreads > 1 )
       {
          // Set the send pti to the message queue mouse pti.
          pti = MessageQueue->ptiMouse;
       }

       if (Msg->message == WM_MOUSEMOVE)
       {
          /* Check if cursor should be visible */
           if(hdcScreen &&
              MessageQueue->CursorObject &&
              MessageQueue->iCursorLevel >= 0)
           {
               /* Check if shape has changed */
               if(CurInfo->CurrentCursorObject != MessageQueue->CursorObject)
               {
                   /* Call GDI to set the new screen cursor */
                    GreSetPointerShape(hdcScreen,
                                       MessageQueue->CursorObject->hbmAlpha ?
                                           NULL : MessageQueue->CursorObject->hbmMask,
                                       MessageQueue->CursorObject->hbmAlpha ?
                                           MessageQueue->CursorObject->hbmAlpha : MessageQueue->CursorObject->hbmColor,
                                       MessageQueue->CursorObject->xHotspot,
                                       MessageQueue->CursorObject->yHotspot,
                                       gpsi->ptCursor.x,
                                       gpsi->ptCursor.y,
                                       MessageQueue->CursorObject->hbmAlpha ? SPS_ALPHA : 0);

               } else
                   GreMovePointer(hdcScreen, Msg->pt.x, Msg->pt.y);
           }
           /* Check if we have to hide cursor */
           else if (CurInfo->ShowingCursor >= 0)
               GreMovePointer(hdcScreen, -1, -1);

           /* Update global cursor info */
           CurInfo->ShowingCursor = MessageQueue->iCursorLevel;
           CurInfo->CurrentCursorObject = MessageQueue->CursorObject;
           gpqCursor = MessageQueue;

           /* Mouse move is a special case */
           MessageQueue->QF_flags |= QF_MOUSEMOVED;
           gdwMouseMoveExtraInfo = dwExtraInfo;
           gdwMouseMoveTimeStamp = Msg->time;
           MsqWakeQueue(pti, QS_MOUSEMOVE, TRUE);
       }
       else
       {
           if (!IntGetCaptureWindow())
           {
             // ERR("ptiLastInput is set\n");
             // ptiLastInput = pti; // Once this is set during Reboot or Shutdown, this prevents the exit window having foreground.
             // Find all the Move Mouse calls and fix mouse set active focus issues......
           }

           // Post mouse move before posting mouse buttons, keep it in sync.
           if (pti->MessageQueue->QF_flags & QF_MOUSEMOVED)
           {
              IntCoalesceMouseMove(pti);
           }

           TRACE("Posting mouse message to hwnd=%p!\n", UserHMGetHandle(pwnd));
           MsqPostMessage(pti, Msg, TRUE, QS_MOUSEBUTTON, 0, dwExtraInfo);
       }
   }
   else if (hdcScreen)
   {
       /* always show cursor on background; FIXME: set default pointer */
       GreMovePointer(hdcScreen, Msg->pt.x, Msg->pt.y);
       CurInfo->ShowingCursor = 0;
   }
}

PUSER_MESSAGE FASTCALL
MsqCreateMessage(LPMSG Msg)
{
   PUSER_MESSAGE Message;

   Message = ExAllocateFromPagedLookasideList(pgMessageLookasideList);
   if (!Message)
   {
      return NULL;
   }

   RtlZeroMemory(Message, sizeof(*Message));
   RtlMoveMemory(&Message->Msg, Msg, sizeof(MSG));
   PostMsgCount++;
   return Message;
}

VOID FASTCALL
MsqDestroyMessage(PUSER_MESSAGE Message)
{
   TRACE("Post Destroy %d\n",PostMsgCount);
   if (Message->pti == NULL)
   {
      ERR("Double Free Message\n");
      return;
   }
   RemoveEntryList(&Message->ListEntry);
   Message->pti = NULL;
   ExFreeToPagedLookasideList(pgMessageLookasideList, Message);
   PostMsgCount--;
}

PUSER_SENT_MESSAGE FASTCALL
AllocateUserMessage(BOOL KEvent)
{
   PUSER_SENT_MESSAGE Message;

   if(!(Message = ExAllocateFromPagedLookasideList(pgSendMsgLookasideList)))
   {
       ERR("AllocateUserMessage(): Not enough memory to allocate a message\n");
       return NULL;
   }
   RtlZeroMemory(Message, sizeof(USER_SENT_MESSAGE));

   if (KEvent)
   {
      Message->pkCompletionEvent = &Message->CompletionEvent;

      KeInitializeEvent(Message->pkCompletionEvent, NotificationEvent, FALSE);
   }
   SendMsgCount++;
   TRACE("AUM pti %p msg %p\n",PsGetCurrentThreadWin32Thread(),Message);
   return Message;
}

VOID FASTCALL
FreeUserMessage(PUSER_SENT_MESSAGE Message)
{
   Message->pkCompletionEvent = NULL;

   /* Remove it from the list */
   RemoveEntryList(&Message->ListEntry);

   ExFreeToPagedLookasideList(pgSendMsgLookasideList, Message);
   SendMsgCount--;
}

VOID APIENTRY
MsqRemoveWindowMessagesFromQueue(PWND Window)
{
   PTHREADINFO pti;
   PUSER_SENT_MESSAGE SentMessage;
   PUSER_MESSAGE PostedMessage;
   PLIST_ENTRY CurrentEntry, ListHead;

   ASSERT(Window);

   pti = Window->head.pti;

   /* remove the posted messages for this window */
   CurrentEntry = pti->PostedMessagesListHead.Flink;
   ListHead = &pti->PostedMessagesListHead;
   while (CurrentEntry != ListHead)
   {
      PostedMessage = CONTAINING_RECORD(CurrentEntry, USER_MESSAGE, ListEntry);

      if (PostedMessage->Msg.hwnd == UserHMGetHandle(Window))
      {
         if (PostedMessage->Msg.message == WM_QUIT && pti->QuitPosted == 0)
         {
            pti->QuitPosted = 1;
            pti->exitCode = PostedMessage->Msg.wParam;
         }
         ClearMsgBitsMask(pti, PostedMessage->QS_Flags);
         MsqDestroyMessage(PostedMessage);
         CurrentEntry = pti->PostedMessagesListHead.Flink;
      }
      else
      {
         CurrentEntry = CurrentEntry->Flink;
      }
   }

   /* remove the sent messages for this window */
   CurrentEntry = pti->SentMessagesListHead.Flink;
   ListHead = &pti->SentMessagesListHead;
   while (CurrentEntry != ListHead)
   {
      SentMessage = CONTAINING_RECORD(CurrentEntry, USER_SENT_MESSAGE, ListEntry);

      if(SentMessage->Msg.hwnd == UserHMGetHandle(Window))
      {
         ERR("Remove Window Messages %p From Sent Queue\n",SentMessage);
#if 0 // Should mark these as invalid and allow the rest clean up, so far no harm by just commenting out. See CORE-9210.
         ClearMsgBitsMask(pti, SentMessage->QS_Flags);

         /* wake the sender's thread */
         if (SentMessage->pkCompletionEvent != NULL)
         {
            KeSetEvent(SentMessage->pkCompletionEvent, IO_NO_INCREMENT, FALSE);
         }

         if (SentMessage->HasPackedLParam)
         {
            if (SentMessage->Msg.lParam)
               ExFreePool((PVOID)SentMessage->Msg.lParam);
         }

         /* free the message */
         FreeUserMessage(SentMessage);

         CurrentEntry = pti->SentMessagesListHead.Flink;
#endif
         CurrentEntry = CurrentEntry->Flink;
      }
      else
      {
         CurrentEntry = CurrentEntry->Flink;
      }
   }
}

BOOLEAN FASTCALL
co_MsqDispatchOneSentMessage(
    _In_ PTHREADINFO pti)
{
   PUSER_SENT_MESSAGE SaveMsg, Message;
   PLIST_ENTRY Entry;
   BOOL Ret;
   LRESULT Result = 0;

   ASSERT(pti == PsGetCurrentThreadWin32Thread());

   if (IsListEmpty(&pti->SentMessagesListHead))
   {
      return(FALSE);
   }

   /* remove it from the list of pending messages */
   Entry = RemoveHeadList(&pti->SentMessagesListHead);
   Message = CONTAINING_RECORD(Entry, USER_SENT_MESSAGE, ListEntry);

   // Signal this message is being processed.
   Message->flags |= SMF_RECEIVERBUSY|SMF_RECEIVEDMESSAGE;

   SaveMsg = pti->pusmCurrent;
   pti->pusmCurrent = Message;

   // Processing a message sent to it from another thread.
   if ( ( Message->ptiSender && pti != Message->ptiSender) ||
        ( Message->ptiCallBackSender && pti != Message->ptiCallBackSender ))
   {  // most likely, but, to be sure.
      pti->pcti->CTI_flags |= CTI_INSENDMESSAGE; // Let the user know...
   }

   /* Now insert it to the global list of messages that can be removed Justin Case there's Trouble */
   InsertTailList(&usmList, &Message->ListEntry);

   ClearMsgBitsMask(pti, Message->QS_Flags);

   if (Message->HookMessage == MSQ_ISHOOK)
   {  // Direct Hook Call processor
      Result = co_CallHook( Message->Msg.message,           // HookId
                            HandleToLong(Message->Msg.hwnd), // Code
                            Message->Msg.wParam,
                            Message->Msg.lParam);
   }
   else if(Message->HookMessage == MSQ_INJECTMODULE)
   {
       Result = IntLoadHookModule(Message->Msg.message,
                                  (HHOOK)Message->Msg.lParam,
                                  Message->Msg.wParam);
   }
   else if ((Message->CompletionCallback) &&
            (Message->ptiCallBackSender == pti))
   {   /* Call the callback routine */
      if (Message->QS_Flags & QS_SMRESULT)
      {
         co_IntCallSentMessageCallback(Message->CompletionCallback,
                                       Message->Msg.hwnd,
                                       Message->Msg.message,
                                       Message->CompletionCallbackContext,
                                       Message->lResult);
         /* Set callback to NULL to prevent reentry */
         Message->CompletionCallback = NULL;
      }
      else
      {
         /* The message has not been processed yet, reinsert it. */
         RemoveEntryList(&Message->ListEntry);
         InsertTailList(&Message->ptiCallBackSender->SentMessagesListHead, &Message->ListEntry);
         // List is occupied need to set the bit.
         MsqWakeQueue(Message->ptiCallBackSender, QS_SENDMESSAGE, TRUE);
         ERR("Callback Message not processed yet. Requeuing the message\n"); //// <---- Need to see if this happens.
         Ret = FALSE;
         goto Exit;
      }
   }
   else
   {  /* Call the window procedure. */
      Result = co_IntSendMessage( Message->Msg.hwnd,
                                  Message->Msg.message,
                                  Message->Msg.wParam,
                                  Message->Msg.lParam);
   }

   /* If the message is a callback, insert it in the callback senders MessageQueue */
   if (Message->CompletionCallback)
   {
      if (Message->ptiCallBackSender)
      {
         Message->lResult = Result;
         Message->QS_Flags |= QS_SMRESULT;

         /* insert it in the callers message queue */
         RemoveEntryList(&Message->ListEntry);
         InsertTailList(&Message->ptiCallBackSender->SentMessagesListHead, &Message->ListEntry);
         MsqWakeQueue(Message->ptiCallBackSender, QS_SENDMESSAGE, TRUE);
      }
      Ret = TRUE;
      goto Exit;
   }

   // Retrieve the result from callback.
   if (Message->QS_Flags & QS_SMRESULT)
   {
      Result = Message->lResult;
   }

   /* Let the sender know the result. */
   Message->lResult = Result;

   if (Message->HasPackedLParam)
   {
      if (Message->Msg.lParam)
         ExFreePool((PVOID)Message->Msg.lParam);
   }

   // Clear busy signal.
   Message->flags &= ~SMF_RECEIVERBUSY;

   /* Notify the sender. */
   if (Message->pkCompletionEvent != NULL)
   {
      KeSetEvent(Message->pkCompletionEvent, IO_NO_INCREMENT, FALSE);
   }

   /* free the message */
   if (Message->flags & SMF_RECEIVERFREE)
   {
      TRACE("Receiver Freeing Message %p\n",Message);
      FreeUserMessage(Message);
   }

   Ret = TRUE;
Exit:
   /* do not hangup on the user if this is reentering */
   if (!SaveMsg) pti->pcti->CTI_flags &= ~CTI_INSENDMESSAGE;
   pti->pusmCurrent = SaveMsg;

   return Ret;
}
/* --- pretranslation split: the remaining function groups live in msgqueue.split2..3.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "msgqueue.split2.c"
#include "msgqueue.split3.c"
