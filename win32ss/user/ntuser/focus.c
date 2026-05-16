/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS Win32k subsystem
 * PURPOSE:          Focus functions
 * FILE:             win32ss/user/ntuser/focus.c
 * PROGRAMER:        ReactOS Team
 */

#include <win32k.h>
#include <immdev.h>
DBG_DEFAULT_CHANNEL(UserFocus);

PUSER_MESSAGE_QUEUE gpqForeground = NULL;
PUSER_MESSAGE_QUEUE gpqForegroundPrev = NULL;
PTHREADINFO gptiForeground = NULL;
PPROCESSINFO gppiLockSFW = NULL;
ULONG guSFWLockCount = 0; // Rule #8, No menus are active. So should be zero.
PTHREADINFO ptiLastInput = NULL;
HWND ghwndOldFullscreen = NULL;

/*
 * Check locking of a process or one or more menus are active.
 */
BOOL FASTCALL
IsFGLocked(VOID)
{
    return (gppiLockSFW || guSFWLockCount);
}

/*
 * Get capture window via foreground Queue.
 */
HWND FASTCALL
IntGetCaptureWindow(VOID)
{
    PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue();
    if (!ForegroundQueue)
        return NULL;
    return (ForegroundQueue->spwndCapture ? UserHMGetHandle(ForegroundQueue->spwndCapture) : NULL);
}

HWND FASTCALL
IntGetThreadFocusWindow(VOID)
{
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
    PUSER_MESSAGE_QUEUE ThreadQueue = pti->MessageQueue;
    if (!ThreadQueue)
        return NULL;
    return (ThreadQueue->spwndFocus ? UserHMGetHandle(ThreadQueue->spwndFocus) : NULL);
}

BOOL FASTCALL IntIsWindowFullscreen(PWND Window)
{
    RECTL rclAnd, rclMonitor, rclWindow;
    PMONITOR pMonitor;

    if (!Window || !(Window->style & WS_VISIBLE) || (Window->style & WS_CHILD) ||
        !IntGetWindowRect(Window, &rclWindow))
    {
        return FALSE;
    }

    pMonitor = UserGetPrimaryMonitor();
    if (!pMonitor)
    {
        RECTL_vSetRect(&rclMonitor, 0, 0,
                       UserGetSystemMetrics(SM_CXSCREEN), UserGetSystemMetrics(SM_CYSCREEN));
    }
    else
    {
        rclMonitor = *(LPRECTL)&pMonitor->rcMonitor;
    }

    RECTL_bIntersectRect(&rclAnd, &rclMonitor, &rclWindow);
    return RtlEqualMemory(&rclAnd, &rclMonitor, sizeof(RECTL));
}

BOOL FASTCALL IntCheckFullscreen(PWND Window)
{
    HWND hWnd;

    if (ghwndOldFullscreen && !IntIsWindowFullscreen(ValidateHwndNoErr(ghwndOldFullscreen)))
        ghwndOldFullscreen = NULL;

    if (!IntIsWindowFullscreen(Window))
        return FALSE;

    hWnd = UserHMGetHandle(Window);
    if (ghwndOldFullscreen != hWnd)
    {
        co_IntShellHookNotify(HSHELL_RUDEAPPACTIVATED, (WPARAM)hWnd, TRUE);
        ghwndOldFullscreen = hWnd;
    }
    return TRUE;
}

VOID FASTCALL
UpdateShellHook(PWND Window)
{
   if (IntCheckFullscreen(Window))
       return;

   if ( Window->spwndParent == UserGetDesktopWindow() &&
       (!(Window->ExStyle & WS_EX_TOOLWINDOW) ||
         (Window->ExStyle & WS_EX_APPWINDOW)))
   {
       // FIXME lParam; The value is TRUE if the window is in full-screen mode, or FALSE otherwise.
       co_IntShellHookNotify(HSHELL_WINDOWACTIVATED, (WPARAM) UserHMGetHandle(Window), FALSE);
   }
   else
   {
       co_IntShellHookNotify(HSHELL_WINDOWACTIVATED, 0, FALSE);
   }
}

BOOL FASTCALL
co_IntSendDeactivateMessages(HWND hWndPrev, HWND hWnd, BOOL Clear)
{
   USER_REFERENCE_ENTRY RefPrev;
   PWND WndPrev;
   BOOL Ret = TRUE;
   LPARAM lParam = hWnd ? (LPARAM)hWnd : 0;

   if (hWndPrev && (WndPrev = ValidateHwndNoErr(hWndPrev)))
   {
      UserRefObjectCo(WndPrev, &RefPrev);

      if (co_IntSendMessage(hWndPrev, WM_NCACTIVATE, FALSE, lParam))
      {
         co_IntSendMessage(hWndPrev, WM_ACTIVATE,
                    MAKEWPARAM(WA_INACTIVE, (WndPrev->style & WS_MINIMIZE) != 0),
                    (LPARAM)hWnd);

         if (WndPrev && Clear)
            WndPrev->state &= ~(WNDS_ACTIVEFRAME|WNDS_HASCAPTION);
      }
      else
      {
         ERR("Application is keeping itself Active to prevent the change!\n");
         Ret = FALSE;
      }

      UserDerefObjectCo(WndPrev);
   }
   return Ret;
}

// Win: xxxFocusSetInputContext
VOID IntFocusSetInputContext(PWND pWnd, BOOL bActivate, BOOL bCallback)
{
    PTHREADINFO pti;
    PWND pImeWnd;
    USER_REFERENCE_ENTRY Ref;
    HWND hImeWnd;
    WPARAM wParam;
    LPARAM lParam;

    if (!pWnd || !pWnd->pcls || IS_WND_IMELIKE(pWnd))
        return;

    pti = pWnd->head.pti;
    if (!pti || (pti->TIF_flags & TIF_INCLEANUP))
        return;

    pImeWnd = pti->spwndDefaultIme;
    if (!pImeWnd)
        return;

    UserRefObjectCo(pImeWnd, &Ref);

    hImeWnd = UserHMGetHandle(pImeWnd);
    wParam = (bActivate ? IMS_IMEACTIVATE : IMS_IMEDEACTIVATE);
    lParam = (LPARAM)UserHMGetHandle(pWnd);

    if (bCallback)
        co_IntSendMessageWithCallBack(hImeWnd, WM_IME_SYSTEM, wParam, lParam, NULL, 0, NULL);
    else
        co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, wParam, lParam);

    UserDerefObjectCo(pImeWnd);
}

//
// Deactivating the foreground message queue.
//
// Release Active, Capture and Focus Windows associated with this message queue.
//
// Win: xxxDeactivate
BOOL FASTCALL
IntDeactivateWindow(PTHREADINFO pti, HANDLE tid)
{
   USER_REFERENCE_ENTRY Ref;
   PTHREADINFO ptiPrev;
   PWND pwndPrev;
   BOOL InAAPM = FALSE;
   PTHREADINFO ptiCurrent = PsGetCurrentThreadWin32Thread();

   if ( !pti->MessageQueue->spwndActive )
   {
      TRACE("IDAW E : Nothing to do, Active is NULL! pti 0x%p tid 0x%p\n",pti,tid);
      return TRUE;
   }

   TRACE("IDAW : pti 0x%p tid 0x%p\n",pti,tid);

   if (ptiCurrent != pti)
   {
      IntReferenceThreadInfo(pti);
      IntReferenceThreadInfo(ptiCurrent);
   }

   if (!(pti->TIF_flags & TIF_INACTIVATEAPPMSG) )
   {
      pti->TIF_flags |= TIF_INACTIVATEAPPMSG;
      InAAPM = TRUE;
   }

   //
   // Check for Capture and release it.
   //
   if ( pti->MessageQueue->spwndCapture )
   {
      MSG msg;
      PWND pwndCapture = pti->MessageQueue->spwndCapture;

      UserRefObjectCo(pwndCapture, &Ref);
      co_IntSendMessage(UserHMGetHandle(pwndCapture), WM_CANCELMODE, 0, 0);
      UserDerefObjectCo(pwndCapture);

      /* Generate mouse move message */
      msg.message = WM_MOUSEMOVE;
      msg.wParam = UserGetMouseButtonsState();
      msg.lParam = MAKELPARAM(gpsi->ptCursor.x, gpsi->ptCursor.y);
      msg.pt = gpsi->ptCursor;
      co_MsqInsertMouseMessage(&msg, 0, 0, TRUE);
   }

   //
   // Check for Active and release it.
   //
   if ( pti->MessageQueue->spwndActive )
   {
      pwndPrev = pti->MessageQueue->spwndActive;
      ptiPrev = pwndPrev->head.pti;

      if (!co_IntSendDeactivateMessages(UserHMGetHandle(pwndPrev), 0, TRUE))
      {
         if (InAAPM) pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
         if (ptiCurrent != pti)
         {
            IntDereferenceThreadInfo(pti);
            IntDereferenceThreadInfo(ptiCurrent);
         }
         return FALSE;
      }

      if ( pti->MessageQueue->spwndActive == pwndPrev )
      {
          pti->MessageQueue->spwndActivePrev = pwndPrev;
          pti->MessageQueue->spwndActive     = NULL;
      }

      if (ptiPrev->TIF_flags & TIF_INCLEANUP) ptiPrev = NULL;
   }
   else
   {
      ptiPrev = pti;
      pwndPrev = (PWND)-1; // Avoid zero Active window.
   }

   if ( ptiPrev )
   {
      HANDLE OldTID = PsGetThreadId(ptiPrev->pEThread);
      PWND cWindow;
      HWND *List, *phWnd;

      List = IntWinListChildren(UserGetDesktopWindow());
      if ( List )
      {
         if ( OldTID )
         {
            for (phWnd = List; *phWnd; ++phWnd)
            {
               cWindow = ValidateHwndNoErr(*phWnd);
               if ( cWindow && cWindow->head.pti == ptiPrev )
               { // FALSE if the window is being deactivated,
                 // ThreadId that owns the window being activated.
                 //ERR("IDW : WM_ACTIVATEAPP(0) hwnd %p tid Old %p New %p\n",UserHMGetHandle(cWindow),OldTID,tid);
                 UserRefObjectCo(cWindow, &Ref);
                 co_IntSendMessage(*phWnd, WM_ACTIVATEAPP, FALSE, (LPARAM)tid);
                 UserDerefObjectCo(cWindow);
               }
            }
         }
         ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
      }
   }

   //
   // Now check for a change (Bounce), if Active same as previous window, release it too.
   //
   if ( pti->MessageQueue->spwndActive == pwndPrev )
   {
      if (!co_IntSendDeactivateMessages(UserHMGetHandle(pwndPrev), 0, FALSE))
      {
         if (InAAPM) pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
         if (ptiCurrent != pti)
         {
            IntDereferenceThreadInfo(pti);
            IntDereferenceThreadInfo(ptiCurrent);
         }
         return FALSE;
      }

      if ( pti->MessageQueue->spwndActive == pwndPrev )
      {
          pti->MessageQueue->spwndActivePrev = pwndPrev;
          pti->MessageQueue->spwndActive     = NULL;
      }
   }

   //
   // Check for Focus and release it.
   //
   if ( pti->MessageQueue->spwndFocus )
   {
      PWND pwndFocus = pti->MessageQueue->spwndFocus;

      //
      // Fix win.c:test_SetForegroundWindow:SetActiveWindow(0)!
      //
      pti->MessageQueue->spwndFocus = NULL; // Null out Focus.

      UserRefObjectCo(pwndFocus, &Ref);
      co_IntSendMessage(UserHMGetHandle(pwndFocus), WM_KILLFOCUS, 0, 0);
      if (IS_IMM_MODE())
      {
         IntFocusSetInputContext(pwndFocus, FALSE, FALSE);
      }
      UserDerefObjectCo(pwndFocus);
   }

   /* Check for keyboard modifiers and release them (CORE-14768) */
   MsqReleaseModifierKeys(pti->MessageQueue);

   if (InAAPM) pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
   if (ptiCurrent != pti)
   {
      IntDereferenceThreadInfo(pti);
      IntDereferenceThreadInfo(ptiCurrent);
   }
   return TRUE;
}

//
// Activating another threads foreground window after a switch.
//
VOID FASTCALL
IntActivateWindow(PWND Wnd, PTHREADINFO pti, HANDLE tid, DWORD Type)
{
   USER_REFERENCE_ENTRY Ref;
   PUSER_MESSAGE_QUEUE pmq = pti->MessageQueue;

   if (Wnd)
   {
      Wnd = VerifyWnd(Wnd);

      if (!Wnd) return;

      UserRefObjectCo(Wnd, &Ref);

      if (!gpqForeground)
      {
          // No foreground queue set.
          co_IntSetForegroundMessageQueue( Wnd, pti, (BOOL)Type, 0);
      }
      else
      {
          // Same Active and Wnd.
          if ( pmq->spwndActive == Wnd )
          {
              WPARAM wParam = (Wnd->head.pti->MessageQueue == gpqForeground);

              co_IntSendMessage( UserHMGetHandle(Wnd), WM_NCACTIVATE, wParam, 0);

              if (wParam)
              {
                  UpdateShellHook(Wnd);

                  co_WinPosSetWindowPos(Wnd, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);
              }
          }
          else // Not the same, set the active Wnd.
          {
              co_IntSetActiveWindow(Wnd,(BOOL)Type,TRUE,TRUE);
          }
      }
      UserDerefObjectCo(Wnd);
   }
   else // Handle no Wnd!
   {
      if ( tid &&                  // TID,
           pmq->spwndActive &&     // Active WND not zero,
           gpqForeground == pmq )  // Same message queues.
      {
           Wnd = pmq->spwndActive; // Use active window from current queue.

           UserRefObjectCo(Wnd, &Ref);

           co_IntSendMessage( UserHMGetHandle(Wnd), WM_NCACTIVATE, TRUE, 0);

           UpdateShellHook(Wnd);

           co_WinPosSetWindowPos(Wnd, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);

           UserDerefObjectCo(Wnd);
      }
      else if (gpqForeground != pmq)
      {
           // Not the same message queue so clear flags for foreground switching.
           pti->TIF_flags &= ~TIF_ALLOWFOREGROUNDACTIVATE;
           pti->ppi->W32PF_flags &= ~W32PF_ALLOWFOREGROUNDACTIVATE;
      }
   }
}

BOOL FASTCALL
co_IntMakeWindowActive(PWND Window)
{
   PWND spwndOwner;
   if (VerifyWnd(Window))
   {  // Set last active for window and it's owner.
      spwndOwner = Window;
      while (spwndOwner->spwndOwner)
      {
         spwndOwner = spwndOwner->spwndOwner;
      }
      WndSetLastActive(spwndOwner, Window);
      return TRUE;
   }
   ERR("MakeWindowActive Failed!\n");
   return FALSE;
}

BOOL FASTCALL
co_IntSendActivateMessages(PWND WindowPrev, PWND Window, BOOL MouseActivate, BOOL Async)
{
   USER_REFERENCE_ENTRY Ref, RefPrev, RefCall;
   HANDLE OldTID, NewTID;
   PTHREADINFO pti, ptiOld, ptiNew;
   BOOL InAAPM = FALSE;

   //ERR("SendActivateMessages\n");

   pti = PsGetCurrentThreadWin32Thread();

   if (Window)
   {
      UserRefObjectCo(Window, &Ref);

      if (WindowPrev) UserRefObjectCo(WindowPrev, &RefPrev);

      pti->MessageQueue->QF_flags &= ~QF_EVENTDEACTIVATEREMOVED;

      /* Send palette messages */
      if (gpsi->PUSIFlags & PUSIF_PALETTEDISPLAY &&
          //co_IntPostOrSendMessage(UserHMGetHandle(Window), WM_QUERYNEWPALETTE, 0, 0))
          co_IntSendMessage(UserHMGetHandle(Window), WM_QUERYNEWPALETTE, 0, 0))
      {
         UserSendNotifyMessage( HWND_BROADCAST,
                                WM_PALETTEISCHANGING,
                               (WPARAM)UserHMGetHandle(Window),
                                0);
      }
      //// Fixes CORE-6434.
      if (!(Window->style & WS_CHILD))
      {
         PWND pwndTemp = co_GetDesktopWindow(Window)->spwndChild;

         while (pwndTemp && !(pwndTemp->style & WS_VISIBLE)) pwndTemp = pwndTemp->spwndNext;

         if (Window != pwndTemp || (WindowPrev && !IntIsWindowVisible(WindowPrev)))
         {
            if (!Async || pti->MessageQueue == gpqForeground)
            {
               UINT flags = SWP_NOSIZE | SWP_NOMOVE;
               if (Window == pwndTemp) flags |= SWP_NOACTIVATE;
               //ERR("co_IntSendActivateMessages SetWindowPos! Async %d pti Q == FGQ %d\n",Async,pti->MessageQueue == gpqForeground);
               co_WinPosSetWindowPos(Window, HWND_TOP, 0, 0, 0, 0, flags);
            }
         }
      }
      ////
      //// CORE-1161 and CORE-6651
      if (Window->spwndPrev)
      {
         HWND *phwndTopLevel, *phwndCurrent;
         PWND pwndCurrent, pwndDesktop;

         pwndDesktop = co_GetDesktopWindow(Window);//UserGetDesktopWindow();
         if (Window->spwndParent == pwndDesktop )
         {
            phwndTopLevel = IntWinListChildren(pwndDesktop);
            phwndCurrent = phwndTopLevel;
            while(*phwndCurrent)
            {
                pwndCurrent = UserGetWindowObject(*phwndCurrent);

                if (pwndCurrent && pwndCurrent->spwndOwner == Window )
                {
                    co_WinPosSetWindowPos(pwndCurrent, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE|SWP_NOMOVE|SWP_NOACTIVATE);
                }
                phwndCurrent++;
            }
            ExFreePoolWithTag(phwndTopLevel, USERTAG_WINDOWLIST);
          }
      }
      ////
   }

   OldTID = WindowPrev ? IntGetWndThreadId(WindowPrev) : NULL;
   NewTID = Window ? IntGetWndThreadId(Window) : NULL;
   ptiOld = WindowPrev ? WindowPrev->head.pti : NULL;
   ptiNew = Window ? Window->head.pti : NULL;

   //ERR("SendActivateMessage Old -> %x, New -> %x\n", OldTID, NewTID);

   if (!(pti->TIF_flags & TIF_INACTIVATEAPPMSG) &&
        (OldTID != NewTID) )
   {
      PWND cWindow;
      HWND *List, *phWnd;

      List = IntWinListChildren(UserGetDesktopWindow());
      if ( List )
      {
         if ( OldTID )
         {
            ptiOld->TIF_flags |= TIF_INACTIVATEAPPMSG;
            // Note: Do not set pci flags, this does crash!
            for (phWnd = List; *phWnd; ++phWnd)
            {
               cWindow = ValidateHwndNoErr(*phWnd);
               if (cWindow && cWindow->head.pti == ptiOld)
               { // FALSE if the window is being deactivated,
                 // ThreadId that owns the window being activated.
                 //ERR("SAM : WM_ACTIVATEAPP(0) tid Old %p New %p\n",OldTID,NewTID);
                 UserRefObjectCo(cWindow, &RefCall);
                 co_IntSendMessage(*phWnd, WM_ACTIVATEAPP, FALSE, (LPARAM)NewTID);
                 UserDerefObjectCo(cWindow);
               }
            }
            ptiOld->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
         }
         if ( NewTID )
         {  //// Prevents a resource crash due to reentrance!
            InAAPM = TRUE;
            pti->TIF_flags |= TIF_INACTIVATEAPPMSG;
            ////
            for (phWnd = List; *phWnd; ++phWnd)
            {
               cWindow = ValidateHwndNoErr(*phWnd);
               if (cWindow && cWindow->head.pti == ptiNew)
               { // TRUE if the window is being activated,
                 // ThreadId that owns the window being deactivated.
                 //ERR("SAM : WM_ACTIVATEAPP(1) hwnd %p tid New %p Old %p\n",UserHMGetHandle(cWindow),NewTID,OldTID);
                 UserRefObjectCo(cWindow, &RefCall);
                 co_IntSendMessage(*phWnd, WM_ACTIVATEAPP, TRUE, (LPARAM)OldTID);
                 UserDerefObjectCo(cWindow);
               }
            }
         }
         ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
      }
   }

   if (Window)
   {
      if (WindowPrev)
         UserDerefObjectCo(WindowPrev); // Now allow the previous window to die.

      if (Window->state & WNDS_ACTIVEFRAME)
      {  // If already active frame do not allow NCPaint.
         //ERR("SendActivateMessage Is Active Frame!\n");
         Window->state |= WNDS_NONCPAINT;
      }

      if (Window->style & WS_MINIMIZE)
      {
         TRACE("Widow was minimized\n");
      }

      co_IntMakeWindowActive(Window);

      co_IntSendMessage( UserHMGetHandle(Window),
                         WM_NCACTIVATE,
                        (WPARAM)(Window == (gpqForeground ? gpqForeground->spwndActive : NULL)),
                         0);

      co_IntSendMessage( UserHMGetHandle(Window),
                         WM_ACTIVATE,
                         MAKEWPARAM(MouseActivate ? WA_CLICKACTIVE : WA_ACTIVE, (Window->style & WS_MINIMIZE) != 0),
                        (LPARAM)(WindowPrev ? UserHMGetHandle(WindowPrev) : 0));

      if (Window->style & WS_VISIBLE)
         UpdateShellHook(Window);

      Window->state &= ~WNDS_NONCPAINT;

      UserDerefObjectCo(Window);
   }
   return InAAPM;
}

// Win: xxxSendFocusMessages
VOID FASTCALL
IntSendFocusMessages( PTHREADINFO pti, PWND pWnd)
{
   PWND pWndPrev;
   PUSER_MESSAGE_QUEUE ThreadQueue = pti->MessageQueue; // Queue can change...
   HWND hwndPrev;
   USER_REFERENCE_ENTRY Ref;

   ThreadQueue->QF_flags &= ~QF_FOCUSNULLSINCEACTIVE;
   if (!pWnd && ThreadQueue->spwndActive)
   {
      ThreadQueue->QF_flags |= QF_FOCUSNULLSINCEACTIVE;
   }

   pWndPrev = ThreadQueue->spwndFocus;
   if (pWndPrev)
      UserRefObjectCo(pWndPrev, &Ref);

   /* check if the specified window can be set in the input data of a given queue */
   if (!pWnd || ThreadQueue == pWnd->head.pti->MessageQueue)
      /* set the current thread focus window */
      ThreadQueue->spwndFocus = pWnd;

   if (pWnd)
   {
      if (pWndPrev)
      {
         co_IntSendMessage(UserHMGetHandle(pWndPrev), WM_KILLFOCUS, (WPARAM)UserHMGetHandle(pWnd), 0);
         if (IS_IMM_MODE())
         {
            IntFocusSetInputContext(pWndPrev, FALSE, FALSE);
         }
      }
      if (ThreadQueue->spwndFocus == pWnd)
      {
         if (IS_IMM_MODE())
         {
            IntFocusSetInputContext(pWnd, TRUE, FALSE);
         }

         IntNotifyWinEvent(EVENT_OBJECT_FOCUS, pWnd, OBJID_CLIENT, CHILDID_SELF, 0);

         hwndPrev = (pWndPrev ? UserHMGetHandle(pWndPrev) : NULL);
         co_IntSendMessage(UserHMGetHandle(pWnd), WM_SETFOCUS, (WPARAM)hwndPrev, 0);
      }
   }
   else
   {
      if (pWndPrev)
      {
         IntNotifyWinEvent(EVENT_OBJECT_FOCUS, NULL, OBJID_CLIENT, CHILDID_SELF, 0);

         co_IntSendMessage(UserHMGetHandle(pWndPrev), WM_KILLFOCUS, 0, 0);
         if (IS_IMM_MODE())
         {
            IntFocusSetInputContext(pWndPrev, FALSE, FALSE);
         }
      }
   }

   if (pWndPrev)
      UserDerefObjectCo(pWndPrev);
}

BOOL FASTCALL
FindRemoveEventMsg(PTHREADINFO pti, DWORD Event, DWORD EventLast)
{
   PUSER_MESSAGE Message;
   PLIST_ENTRY Entry;
   BOOL Ret = FALSE;

   Entry = pti->PostedMessagesListHead.Flink;
   while (Entry != &pti->PostedMessagesListHead)
   {
      // Scan posted queue messages to see if we received async messages.
      Message = CONTAINING_RECORD(Entry, USER_MESSAGE, ListEntry);
      Entry = Entry->Flink;

      if (Message->dwQEvent == EventLast)
      {
         //ERR("Event D/SAW: Last Activate/Deactivate %d\n", EventLast);
         return Ret;
      }

      if (Message->dwQEvent == Event)
      {
         //ERR("Event D/SAW: Found one in the Post Msg Queue! Activate/Deactivate %d\n", Event);
         ClearMsgBitsMask(pti, Message->QS_Flags);
         MsqDestroyMessage(Message);
         Ret = TRUE;
      }
   }
   return Ret;
}

BOOL FASTCALL
ToggleFGActivate(PTHREADINFO pti)
{
   BOOL Ret;
   PPROCESSINFO ppi = pti->ppi;

   Ret = !!(pti->TIF_flags & TIF_ALLOWFOREGROUNDACTIVATE);
   if (Ret)
   {
      pti->TIF_flags &= ~TIF_ALLOWFOREGROUNDACTIVATE;
   }
   else
      Ret = !!(ppi->W32PF_flags & W32PF_ALLOWFOREGROUNDACTIVATE);

   if (Ret)
      ppi->W32PF_flags &= ~W32PF_ALLOWFOREGROUNDACTIVATE;
   //ERR("ToggleFGActivate is %d\n",Ret);
   return Ret;
}

BOOL FASTCALL
IsAllowedFGActive(PTHREADINFO pti, PWND Wnd)
{
   // Not allowed if one or more,,
   if (!ToggleFGActivate(pti) ||              // bits not set,
        pti->rpdesk != gpdeskInputDesktop ||  // not current Desktop,
        pti->MessageQueue == gpqForeground || // if already the queue foreground,
        IsFGLocked() ||                       // foreground is locked,
        (Wnd->ExStyle & WS_EX_NOACTIVATE))    // or, does not become the foreground window when the user clicks it.
   {
      return FALSE;
   }
   //ERR("IsAllowedFGActive is TRUE\n");
   return TRUE;
}

/*
   Can the system force foreground from one or more conditions.
 */
BOOL FASTCALL
CanForceFG(PPROCESSINFO ppi)
{
   if (!ptiLastInput ||
        ptiLastInput->ppi == ppi ||
       !gptiForeground ||
        gptiForeground->ppi == ppi ||
        ppi->W32PF_flags & (W32PF_ALLOWFOREGROUNDACTIVATE | W32PF_SETFOREGROUNDALLOWED) ||
        gppiInputProvider == ppi ||
       !gpqForeground
      ) return TRUE;
   //ERR("CanForceFG is FALSE\n");
   return FALSE;
}

//
//  Switching out foreground message queues.
//
BOOL FASTCALL
co_IntSetForegroundMessageQueue(
    _In_opt_ PWND Wnd,
    _In_ PTHREADINFO pti,
    _In_ BOOL MouseActivate,
    _In_ DWORD Type )
{
   PTHREADINFO ptiChg, ptiPrev;
   PUSER_MESSAGE_QUEUE pumq, pumqChg, pumqPrev;
   BOOL Removed, Ret = TRUE;

   if (Wnd && !VerifyWnd(Wnd))
   {
      return FALSE;
   }

   if ( !gptiForeground || gptiForeground->TIF_flags & TIF_INCLEANUP )
      ptiPrev = NULL;
   else
      ptiPrev = gptiForeground;

   if (Wnd)
   {
      ptiChg = Wnd->head.pti;
      IntSetFocusMessageQueue(Wnd->head.pti->MessageQueue);
      gptiForeground = Wnd->head.pti;
      //ERR("Set Foreground pti 0x%p Q 0x%p hWnd 0x%p\n", Wnd->head.pti, Wnd->head.pti->MessageQueue, UserHMGetHandle(Wnd));
   }
   else
   {
      ptiChg = NULL;
      IntSetFocusMessageQueue(NULL);
      gptiForeground = NULL;
      //ERR("Set Foreground pti 0x0 Q 0x0 hWnd 0x0\n");
   }

   //
   // Process the changing out of the message queues.
   //
   if (gpqForegroundPrev != gpqForeground)
   {
      pumqPrev = NULL;
      if ( ptiPrev && !(ptiPrev->TIF_flags & TIF_INCLEANUP) )
      {
         pumqPrev = ptiPrev->MessageQueue;
      }

      pumq = pti ? pti->MessageQueue : NULL;

      // Deactivate the previous message queue.
      if (pumqPrev)
      {
          if ( pumq != pumqPrev )
          {
              MSG Msg;
              HWND hWndPrev = pumqPrev->spwndActive ? UserHMGetHandle(pumqPrev->spwndActive) : NULL;
              HANDLE tid = gptiForeground ? PsGetThreadId(gptiForeground->pEThread) : NULL; // TID from changing Window PTI.

              Msg.message = WM_ASYNC_SETACTIVEWINDOW;
              Msg.hwnd = hWndPrev;
              Msg.wParam = (WPARAM)pumqPrev->spwndActive;
              Msg.lParam = 0;
              Msg.time = 0;
              //ERR("SFWAMQ : DAW P pti 0x%p tid 0x%p hWndPrev 0x%p\n",ptiPrev,tid,hWndPrev);
              MsqPostMessage(ptiPrev, &Msg, FALSE, QS_EVENT, POSTEVENT_DAW, (LONG_PTR)tid);
          }
      }

      pumqChg = NULL;
      if ( ptiChg && !(ptiChg->TIF_flags & TIF_INCLEANUP) )
      {
         pumqChg = ptiChg->MessageQueue;
      }

      pumq = pti ? pti->MessageQueue : NULL;

      // Activate changing message queue.
      if (pumqChg)
      {
         /*
            Henri Verbeet,
            What happens is that we get the WM_WINE_SETACTIVEWINDOW message sent by the
            other thread after we already changed the foreground window back to our own
            window.
          */
          //ERR("SFWAMQ : 1\n");
          Removed = FindRemoveEventMsg(ptiChg, POSTEVENT_DAW, POSTEVENT_NONE);

          if (pumqChg != pumq)
          {
              MSG Msg;
              HWND hWnd = Wnd ? UserHMGetHandle(Wnd) : NULL;
              HANDLE tid = ptiPrev ? PsGetThreadId(ptiPrev->pEThread) : NULL;

              if (Removed) pumqChg->QF_flags |= QF_EVENTDEACTIVATEREMOVED;

              Msg.message = WM_ASYNC_SETACTIVEWINDOW;
              Msg.hwnd = hWnd;
              Msg.wParam = (WPARAM)Wnd;
              Msg.lParam = (LPARAM)tid; //// Fixme! Type flags?
              Msg.time = 0;
              //ERR("SFWAMQ : SAW P pti 0x%p tid 0x%p hWnd 0x%p\n",ptiChg,tid,hWnd);
              MsqPostMessage(ptiChg, &Msg, FALSE, QS_EVENT, POSTEVENT_SAW, (LONG_PTR)Type|MouseActivate);
          }
          else // Current message queue same as changed message queue.
          {
              if (pumq->spwndActive == Wnd)
              {
                 co_IntSendMessage( UserHMGetHandle(Wnd), WM_NCACTIVATE, TRUE, (LPARAM)UserHMGetHandle(Wnd));

                 UpdateShellHook(Wnd);

                 co_WinPosSetWindowPos(Wnd, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);
              }
              else
              {
                 //ERR("SFWAMQ : SAW I pti 0x%p hWnd 0x%p\n", ptiChg, Wnd ? UserHMGetHandle(Wnd) : NULL);
                 Ret = co_IntSetActiveWindow(Wnd, MouseActivate, TRUE/*Type*/, FALSE);
                 //if (!Ret) ERR("SFWAMQ : ISAW : return error\n");
                 return Ret;
              }
          }
      }

      // Handle same message queue after switch out.
      pumqPrev = NULL;
      if ( ptiPrev && !(ptiPrev->TIF_flags & TIF_INCLEANUP) )
      {
         pumqPrev = ptiPrev->MessageQueue;
      }
      pumq = pti ? pti->MessageQueue : NULL;

      if ( pumqPrev && pumq == pumqPrev )
      {
          HANDLE tid = Wnd ? PsGetThreadId(Wnd->head.pti->pEThread) : NULL;
          //ERR("SFWAMQ : DAW I pti 0x%p tid 0x%p hWnd 0x%p\n", ptiPrev, tid, Wnd ? UserHMGetHandle(Wnd) : NULL);
          IntDeactivateWindow(pti, tid);
      }
   }
   return Ret;
}

/*
   MSDN:
   The system restricts which processes can set the foreground window. A process
   can set the foreground window only if one of the following conditions is true:

    * The process is the foreground process.
    * The process was started by the foreground process.
    * The process received the last input event.
    * There is no foreground process.
    * The foreground process is being debugged.
    * The foreground is not locked (see LockSetForegroundWindow).
    * The foreground lock time-out has expired (see SPI_GETFOREGROUNDLOCKTIMEOUT in SystemParametersInfo).
    * No menus are active.
*/
static
BOOL FASTCALL
co_IntSetForegroundAndFocusWindow(
    _In_opt_ PWND Wnd,
    _In_ BOOL MouseActivate,
    _In_ BOOL bFlash )
{
   HWND hWnd = Wnd ? UserHMGetHandle(Wnd) : NULL;
   PUSER_MESSAGE_QUEUE PrevForegroundQueue;
   PTHREADINFO pti;
   BOOL Ret = FALSE;

   if (Wnd) ASSERT_REFS_CO(Wnd);

   TRACE("co_IntSetForegroundAndFocusWindow hwnd=%p mouse=%d\n", hWnd, MouseActivate);

   PrevForegroundQueue = IntGetFocusMessageQueue(); // Use this active desktop.
   pti = PsGetCurrentThreadWin32Thread();

   if (Wnd && PrevForegroundQueue)
   {  // Same Window Q as foreground just do active.
      if (Wnd->head.pti->MessageQueue == PrevForegroundQueue)
      {
         //ERR("Same Window Q as foreground just do active.\n");
         if (pti->MessageQueue == PrevForegroundQueue)
         { // Same WQ and TQ go active.
            //ERR("Same WQ and TQ go active.\n");
            Ret = IntUserSetActiveWindow(Wnd, MouseActivate, TRUE, FALSE);
         }
         else if (Wnd->head.pti->MessageQueue->spwndActive == Wnd)
         { // Same WQ and it is active.
            //ERR("Same WQ and it is active.\n");
            Ret = TRUE;
         }
         else
         { // Same WQ as FG but not the same TQ send active.
            //ERR("Same WQ as FG but not the same TQ send active.\n");
            MSG Msg;
            PTHREADINFO ptiNew = Wnd->head.pti;

            Msg.message = WM_ASYNC_SETACTIVEWINDOW;
            Msg.hwnd = hWnd;
            Msg.wParam = (WPARAM)Wnd;
            Msg.lParam = 0;
            Msg.time = 0;
            //ERR("SFAFW 1 : SAW P pti 0x%p hWnd 0x%p\n",ptiNew,hWnd);
            MsqPostMessage(ptiNew, &Msg, FALSE, QS_EVENT, POSTEVENT_SAW, (LONG_PTR)MouseActivate);

            Ret = TRUE;
         }
         return Ret;
      }
   }

   if ( (( !IsFGLocked() || pti->ppi == gppiInputProvider ) &&
         ( CanForceFG(pti->ppi) || pti->TIF_flags & (TIF_SYSTEMTHREAD|TIF_CSRSSTHREAD|TIF_ALLOWFOREGROUNDACTIVATE) )) ||
        pti->ppi == ppiScrnSaver
      )
   {

      ToggleFGActivate(pti);

      return co_IntSetForegroundMessageQueue( Wnd, pti, MouseActivate, 0 );
   }

   if (!Wnd) return FALSE; // No window, always return FALSE.

   //// if (bFlash) FIXME : handle flash!!!

   if (pti->MessageQueue == Wnd->head.pti->MessageQueue)
   {
       //ERR("Same PQ and WQ go active.\n");
       Ret = IntUserSetActiveWindow(Wnd, MouseActivate, TRUE, FALSE);
       //if (!Ret) ERR("ISFAFW : IUSAW : return error\n");
   }
   else if (Wnd->head.pti->MessageQueue->spwndActive == Wnd)
   {
       TRACE("Same Active and Wnd.\n"); // Leave this for now.
   }
   else
   {
       //ERR("Activate Not same PQ and WQ and Wnd.\n");
       //// CORE-10785 fix hang, ROSTESTS-208 allows test to run.
       MSG Msg;
       PTHREADINFO ptiNew = Wnd->head.pti;

       Msg.message = WM_ASYNC_SETACTIVEWINDOW;
       Msg.hwnd = hWnd;
       Msg.wParam = (WPARAM)Wnd;
       Msg.lParam = 0;
       Msg.time = 0;
       //ERR("SFAFW 2 : SAW P pti 0x%p hWnd 0x%p\n",ptiNew,hWnd);
       MsqPostMessage(ptiNew, &Msg, FALSE, QS_EVENT, POSTEVENT_SAW, (LONG_PTR)MouseActivate);
   }
   // Always return FALSE.
   return FALSE;
}
/* --- pretranslation split: the remaining function groups live in focus.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "focus.split2.c"
