/* focus.split2.c — #include fragment of focus.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original focus.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


//
// Set the Active Window.
//
BOOL FASTCALL
co_IntSetActiveWindow(
   _In_ PWND Wnd,
   _In_ BOOL bMouse,
   _In_ BOOL bFocus,
   _In_ BOOL Async )
{
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE ThreadQueue;
   PWND pWndChg, WndPrev; // State changes.
   HWND hWndPrev;
   HWND hWnd = 0;
   BOOL InAAPM;
   CBTACTIVATESTRUCT cbt;

   //ERR("co_IntSetActiveWindow 1\n");

   pti = PsGetCurrentThreadWin32Thread();
   ThreadQueue = pti->MessageQueue;
   ASSERT(ThreadQueue != 0);

   pWndChg = ThreadQueue->spwndActive; // Keep to notify of a preemptive switch.
   hWndPrev = (pWndChg ? UserHMGetHandle(pWndChg) : NULL);

   if ( !Wnd || Wnd == UserGetDesktopWindow() )
   {
      //ERR("ISAW : NULL %p\n",Wnd);
      return FALSE;
   }

   ASSERT_REFS_CO(Wnd);
   hWnd = UserHMGetHandle(Wnd);
   //ERR("co_IntSetActiveWindow 2 hWnd 0x%p\n",hWnd);

   if (Wnd->ExStyle & WS_EX_NOACTIVATE)
       return TRUE;

   /* check if the specified window can be set in the input data of a given queue */
   if ( ThreadQueue != Wnd->head.pti->MessageQueue )
   {
      //ERR("ISAW : Must have the same Message Queue\n");
      return FALSE;
   }

   if (!VerifyWnd(Wnd))
   {
      //ERR("ISAW : Window is in Destroy!\n");
      return FALSE;
   }

   if ( Wnd == pWndChg )
   {
      //ERR("ISAW : Nothing to do\n");
      return TRUE; // Fix CORE-8780 and CORE-11979. See CORE-11324 for breakage.
   }

   if ( Wnd->state & WNDS_BEINGACTIVATED ) return TRUE;

   /* Call CBT hook chain */
   cbt.fMouse     = bMouse;
   cbt.hWndActive = hWndPrev;
   if (co_HOOK_CallHooks( WH_CBT, HCBT_ACTIVATE, (WPARAM)hWnd, (LPARAM)&cbt))
   {
      ERR("SetActiveWindow: WH_CBT Call Hook return!\n");
      return FALSE;
   }

   ThreadQueue->QF_flags &= ~QF_EVENTDEACTIVATEREMOVED;

   if ( ThreadQueue->spwndActive && ThreadQueue->spwndActive->state & WNDS_DESTROYED )
      ThreadQueue->spwndActive = NULL;
   else
      ThreadQueue->spwndActivePrev = ThreadQueue->spwndActive;

   WndPrev = ThreadQueue->spwndActive; // Keep to save changing active.

   if (WndPrev)
   {
      if (ThreadQueue == gpqForeground) gpqForegroundPrev = ThreadQueue;
      if (!co_IntSendDeactivateMessages(UserHMGetHandle(WndPrev), hWnd, TRUE)) return FALSE;
   }

   WndPrev = ThreadQueue->spwndActive; // Again keep to save changing active.

   // While in calling message proc or hook:
   // Fail if a preemptive switch was made, current active not made previous,
   // focus window is dead or no longer the same thread queue.
   if ( ThreadQueue->spwndActivePrev != ThreadQueue->spwndActive ||
        pWndChg != WndPrev ||
        (Wnd && !VerifyWnd(Wnd)) ||
        ThreadQueue != pti->MessageQueue )
   {
      ERR("SetActiveWindow: Summary ERROR, active state changed!\n");
      return FALSE;
   }

   if (!WndPrev) ThreadQueue->QF_flags &= ~QF_FOCUSNULLSINCEACTIVE;

   /* set the current thread active window */
   ThreadQueue->spwndActive = Wnd;

   // Set state flag to prevent recursions.
   Wnd->state |= WNDS_BEINGACTIVATED;

   IntNotifyWinEvent(EVENT_SYSTEM_FOREGROUND, Wnd, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);

   // Clear out activate EVENT messages.
   FindRemoveEventMsg(pti, POSTEVENT_SAW, POSTEVENT_DAW);

   WndPrev = VerifyWnd(ThreadQueue->spwndActivePrev); // Now should be set but verify it again.

   InAAPM = co_IntSendActivateMessages(WndPrev, Wnd, bMouse, Async);

   /* now change focus if necessary */
   //// Fixes CORE-6452 allows setting focus on window.
   if (bFocus && !(ThreadQueue->QF_flags & QF_FOCUSNULLSINCEACTIVE))
   {
      /* Do not change focus if the window is no longer active */
      if (pti->MessageQueue->spwndActive != IntGetNonChildAncestor(pti->MessageQueue->spwndFocus))
      {
         PWND pWndSend = pti->MessageQueue->spwndActive;
         // Clear focus if the active window is minimized.
         if (pWndSend && pti->MessageQueue->spwndActive->style & WS_MINIMIZE) pWndSend = NULL;
         // Send focus messages and if so, set the focus.
         IntSendFocusMessages( pti, pWndSend);
      }
   }
   ////
   if (InAAPM)
   {
      pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
   }

   // Checked in MENU_TrackMenu
   ThreadQueue->QF_flags |= QF_ACTIVATIONCHANGE;

   //ERR("co_IntSetActiveWindow Exit\n");
   Wnd->state &= ~WNDS_BEINGACTIVATED;
   return (ThreadQueue->spwndActive == Wnd);
}

//
// Set the Active Window.
//
// Window is not optional!
//
BOOL FASTCALL
IntUserSetActiveWindow(
   _In_ PWND Wnd,
   _In_ BOOL bMouse,
   _In_ BOOL bFocus,
   _In_ BOOL Async)
{
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE ThreadQueue;

   //ERR("IntUserSetActiveWindow 1\n");
   ASSERT_REFS_CO(Wnd);
   if ((Wnd->style & (WS_POPUP|WS_CHILD)) == WS_CHILD) return FALSE;
   //ERR("IntUserSetActiveWindow 1a hWnd 0x%p\n",UserHMGetHandle(Wnd));

   //ERR("IntUserSetActiveWindow 2\n");
   pti = PsGetCurrentThreadWin32Thread();
   ThreadQueue = pti->MessageQueue;
   ASSERT(ThreadQueue != 0);

   while (Wnd)
   {
      BOOL Ret, DoFG, AllowFG;

      if (ThreadQueue == Wnd->head.pti->MessageQueue)
      {
         if (IsAllowedFGActive(pti, Wnd))
         {
             DoFG = TRUE;
         }
         else
         {
             //ERR("IntUserSetActiveWindow 3 Go Out!\n");
             break;
         }
         AllowFG = !pti->cVisWindows; // Nothing is visable.
         //ERR("IntUserSetActiveWindow 3a DoFG = %d AllowFG = %d\n",DoFG,AllowFG);
      }
      else //if (ThreadQueue != Wnd->head.pti->MessageQueue)
      {
         //PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue();
         // Rule 1 & 4, We are foreground so set this FG window or NULL foreground....
         if (!gpqForeground || gpqForeground == ThreadQueue)
         {
            DoFG = TRUE;
         }
         else
            DoFG = FALSE;
         if (DoFG)
         {
            if (pti->TIF_flags & TIF_ALLOWFOREGROUNDACTIVATE || pti->cVisWindows)
               AllowFG = TRUE;
            else
               AllowFG = FALSE;
         }
         else
            AllowFG = FALSE;
         //ERR("IntUserSetActiveWindow 3b DoFG = %d AllowFG = %d\n",DoFG,AllowFG);
      }
      Ret = FALSE;
      if (DoFG)
      {
         pti->TIF_flags |= TIF_ALLOWFOREGROUNDACTIVATE;
         //ERR("IntUserSetActiveWindow 3c FG set\n");
         Ret = co_IntSetForegroundAndFocusWindow(Wnd, bMouse, TRUE);
         if (AllowFG)
         {
            pti->TIF_flags |= TIF_ALLOWFOREGROUNDACTIVATE;
         }
         else
         {
            pti->TIF_flags &= ~TIF_ALLOWFOREGROUNDACTIVATE;
         }
      }
      return Ret;
   }

   return co_IntSetActiveWindow(Wnd, bMouse, bFocus, Async);
}

BOOL FASTCALL
co_IntMouseActivateWindow(PWND Wnd)
{
   TRACE("Mouse Active\n");
   if (Wnd && (Wnd->ExStyle & WS_EX_NOACTIVATE))
      return TRUE;
   return co_IntSetForegroundAndFocusWindow(Wnd, TRUE, TRUE);
}

/* Win: PWND xxxSetActiveWindow(Wnd) */
BOOL FASTCALL
UserSetActiveWindow( _In_opt_ PWND Wnd )
{
  PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

  if (Wnd)
  {
     if ((Wnd->style & (WS_POPUP|WS_CHILD)) == WS_CHILD) return FALSE;

     return IntUserSetActiveWindow(Wnd, FALSE, TRUE, FALSE);
  }
  /*
     Yes your eye are not deceiving you~!

     First part of wines Win.c test_SetActiveWindow:

     flush_events( TRUE );
     ShowWindow(hwnd, SW_HIDE);
     SetFocus(0);
     SetActiveWindow(0);
     check_wnd_state(0, 0, 0, 0); <-- This should pass if ShowWindow does it's job!!! As of 10/28/2012 it does!

     Now Handle wines Msg.c test_SetActiveWindow( 0 )...
  */
  TRACE("USAW: Previous active window\n");
  if (  gpqForegroundPrev &&
        gpqForegroundPrev->spwndActivePrev &&
       (gpqForegroundPrev->spwndActivePrev->style & (WS_VISIBLE|WS_DISABLED)) == WS_VISIBLE  &&
      !(gpqForegroundPrev->spwndActivePrev->state2 & WNDS2_BOTTOMMOST) &&
       (Wnd = VerifyWnd(gpqForegroundPrev->spwndActivePrev)) != NULL )
  {
     TRACE("USAW:PAW hwnd %p\n", UserHMGetHandle(Wnd));
     return IntUserSetActiveWindow(Wnd, FALSE, TRUE, FALSE);
  }

  // Activate anyone but the active window.
  if ( pti->MessageQueue->spwndActive &&
      (Wnd = VerifyWnd(pti->MessageQueue->spwndActive)) != NULL )
  {
      //ERR("USAW:AOWM hwnd %p\n", UserHMGetHandle(Wnd));
      if (!ActivateOtherWindowMin(Wnd))
      {
         // Okay, now go find someone else to play with!
         //ERR("USAW: Going to WPAOW\n");
         co_WinPosActivateOtherWindow(Wnd);
      }
      return TRUE;
  }

  TRACE("USAW: Nothing\n");
  return FALSE;
}

// Win: PWND xxxSetFocus(Window)
HWND FASTCALL
co_UserSetFocus(PWND Window)
{
   HWND hWndPrev = 0;
   PWND pwndTop;
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE ThreadQueue;

   if (Window)
      ASSERT_REFS_CO(Window);

   pti = PsGetCurrentThreadWin32Thread();
   ThreadQueue = pti->MessageQueue;
   ASSERT(ThreadQueue != 0);

   TRACE("Enter SetFocus hWnd 0x%p pti 0x%p\n",Window ? UserHMGetHandle(Window) : 0, pti );

   hWndPrev = ThreadQueue->spwndFocus ? UserHMGetHandle(ThreadQueue->spwndFocus) : 0;

   if (Window != 0)
   {
      if (hWndPrev == UserHMGetHandle(Window))
      {
         return hWndPrev ? (IntIsWindow(hWndPrev) ? hWndPrev : 0) : 0; /* Nothing to do */
      }

      if (Window->head.pti->MessageQueue != ThreadQueue)
      {
         ERR("SetFocus Must have the same Q!\n");
         return 0;
      }

      /* Check if we can set the focus to this window */
      //// Fixes wine win test_SetParent both "todo" line 3710 and 3720...
      for (pwndTop = Window; pwndTop; pwndTop = pwndTop->spwndParent)
      {
         if (pwndTop->style & (WS_MINIMIZED|WS_DISABLED)) return 0;
         if ((pwndTop->style & (WS_POPUP|WS_CHILD)) != WS_CHILD) break;
         if (pwndTop->spwndParent == NULL) break;
      }
      ////
      if (co_HOOK_CallHooks( WH_CBT, HCBT_SETFOCUS, (WPARAM)UserHMGetHandle(Window), (LPARAM)hWndPrev))
      {
         ERR("SetFocus 1 WH_CBT Call Hook return!\n");
         return 0;
      }

      /* Activate pwndTop if needed. */
      if (pwndTop != ThreadQueue->spwndActive)
      {
         PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue(); // Keep it based on desktop.
         if (ThreadQueue != ForegroundQueue && IsAllowedFGActive(pti, pwndTop)) // Rule 2 & 3.
         {
            //ERR("SetFocus: Set Foreground!\n");
            if (!(pwndTop->style & WS_VISIBLE))
            {
                pti->ppi->W32PF_flags |= W32PF_ALLOWFOREGROUNDACTIVATE;
            }
            if (!co_IntSetForegroundAndFocusWindow(pwndTop, FALSE, TRUE))
            {
               ERR("SetFocus: Set Foreground and Focus Failed!\n");
               return 0;
            }
         }

         /* Set Active when it is needed. */
         if (pwndTop != ThreadQueue->spwndActive)
         {
            //ERR("SetFocus: Set Active! %p\n",pwndTop?UserHMGetHandle(pwndTop):0);
            if (!co_IntSetActiveWindow(pwndTop, FALSE, FALSE, FALSE))
            {
               ERR("SetFocus: Set Active Failed!\n");
               return 0;
            }
         }

         /* Abort if window destroyed */
         if (Window->state2 & WNDS2_INDESTROY) return 0;
         /* Do not change focus if the window is no longer active */
         if (pwndTop != ThreadQueue->spwndActive)
         {
            ERR("SetFocus: Top window did not go active!\n");
            return 0;
         }
      }

      // Check again! SetActiveWindow could have set the focus via WM_ACTIVATE.
      hWndPrev = ThreadQueue->spwndFocus ? UserHMGetHandle(ThreadQueue->spwndFocus) : 0;

      IntSendFocusMessages( pti, Window);

      TRACE("Focus: %p -> %p\n", hWndPrev, UserHMGetHandle(Window));
   }
   else /* NULL hwnd passed in */
   {
      if (co_HOOK_CallHooks( WH_CBT, HCBT_SETFOCUS, (WPARAM)0, (LPARAM)hWndPrev))
      {
         ERR("SetFocus: 2 WH_CBT Call Hook return!\n");
         return 0;
      }
      //ERR("SetFocus: Set Focus NULL\n");
      /* set the current thread focus window null */
      IntSendFocusMessages( pti, NULL);
   }
   return hWndPrev ? (IntIsWindow(hWndPrev) ? hWndPrev : 0) : 0;
}

HWND FASTCALL
UserGetForegroundWindow(VOID)
{
    PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue();
    if (!ForegroundQueue)
        return NULL;
    return (ForegroundQueue->spwndActive ? UserHMGetHandle(ForegroundQueue->spwndActive) : NULL);
}

HWND FASTCALL UserGetActiveWindow(VOID)
{
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
    PUSER_MESSAGE_QUEUE ThreadQueue = pti->MessageQueue;
    if (!ThreadQueue)
        return NULL;
    return (ThreadQueue->spwndActive ? UserHMGetHandle(ThreadQueue->spwndActive) : NULL);
}

HWND APIENTRY
IntGetCapture(VOID)
{
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE ThreadQueue;
   HWND Ret;

   TRACE("Enter IntGetCapture\n");

   pti = PsGetCurrentThreadWin32Thread();
   ThreadQueue = pti->MessageQueue;
   Ret = ((ThreadQueue && ThreadQueue->spwndCapture) ? UserHMGetHandle(ThreadQueue->spwndCapture) : NULL);

   TRACE("Leave IntGetCapture, ret=%p\n", Ret);
   return Ret;
}

HWND FASTCALL
co_UserSetCapture(HWND hWnd)
{
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE ThreadQueue;
   PWND pWnd, Window = NULL;
   HWND hWndPrev;

   pti = PsGetCurrentThreadWin32Thread();
   ThreadQueue = pti->MessageQueue;

   if (ThreadQueue->QF_flags & QF_CAPTURELOCKED)
      return NULL;

   if (hWnd && (Window = UserGetWindowObject(hWnd)))
   {
      if (Window->head.pti->MessageQueue != ThreadQueue)
      {
         ERR("Window Thread does not match Current!\n");
         return NULL;
      }
   }

   hWndPrev = MsqSetStateWindow(pti, MSQ_STATE_CAPTURE, hWnd);

   if (hWndPrev)
   {
      pWnd = UserGetWindowObject(hWndPrev);
      if (pWnd)
         IntNotifyWinEvent(EVENT_SYSTEM_CAPTUREEND, pWnd, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
   }

   if (Window)
      IntNotifyWinEvent(EVENT_SYSTEM_CAPTURESTART, Window, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);

   //
   // Only send the message if we have a previous Window!
   // Fix msg_menu tracking popup menu and win test_capture_4!!!!
   //
   if (hWndPrev)
   {
      if (ThreadQueue->MenuOwner && Window) ThreadQueue->QF_flags |= QF_CAPTURELOCKED;

      co_IntSendMessage(hWndPrev, WM_CAPTURECHANGED, 0, (LPARAM)hWnd);

      ThreadQueue->QF_flags &= ~QF_CAPTURELOCKED;
   }

   if (hWnd == NULL) // Release mode.
   {
      MOUSEINPUT mi;
   /// These are HACKS!
      /* Also remove other windows if not capturing anymore */
      MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, NULL);
      MsqSetStateWindow(pti, MSQ_STATE_MOVESIZE, NULL);
   ///
      /* Somebody may have missed some mouse movements */
      mi.dx = 0;
      mi.dy = 0;
      mi.mouseData = 0;
      mi.dwFlags = MOUSEEVENTF_MOVE;
      mi.time = 0;
      mi.dwExtraInfo = 0;
      UserSendMouseInput(&mi, FALSE);
   }
   return hWndPrev;
}

/*
  API Call
*/
BOOL
FASTCALL
IntReleaseCapture(VOID)
{
   PTHREADINFO pti;
   PUSER_MESSAGE_QUEUE ThreadQueue;

   pti = PsGetCurrentThreadWin32Thread();
   ThreadQueue = pti->MessageQueue;

   // Can not release inside WM_CAPTURECHANGED!!
   if (ThreadQueue->QF_flags & QF_CAPTURELOCKED) return FALSE;

   co_UserSetCapture(NULL);

   return TRUE;
}

/*
  API Call
*/
BOOL FASTCALL
co_IntSetForegroundWindow(PWND Window)
{
   if (Window) ASSERT_REFS_CO(Window);

   return co_IntSetForegroundAndFocusWindow(Window, FALSE, TRUE);
}

/*
  API Call
*/
BOOL FASTCALL
co_IntSetForegroundWindowMouse(PWND Window)
{
   if (Window) ASSERT_REFS_CO(Window);

   return co_IntSetForegroundAndFocusWindow(Window, TRUE, FALSE);
}

/*
  API Call
*/
BOOL FASTCALL
IntLockSetForegroundWindow(UINT uLockCode)
{
   ULONG Err = ERROR_ACCESS_DENIED;
   PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();
   switch (uLockCode)
   {
      case LSFW_LOCK:
         if ( CanForceFG(ppi) && !gppiLockSFW )
         {
            gppiLockSFW = ppi;
            return TRUE;
         }
         break;
      case LSFW_UNLOCK:
         if ( gppiLockSFW == ppi)
         {
            gppiLockSFW = NULL;
            return TRUE;
         }
         break;
      default:
         Err = ERROR_INVALID_PARAMETER;
   }
   EngSetLastError(Err);
   return FALSE;
}

/*
  API Call
*/
BOOL FASTCALL
IntAllowSetForegroundWindow(DWORD dwProcessId)
{
   PPROCESSINFO ppi, ppiCur;
   PEPROCESS Process = NULL;

   ppi = NULL;
   if (dwProcessId != ASFW_ANY)
   {
      if (!NT_SUCCESS(PsLookupProcessByProcessId(UlongToHandle(dwProcessId), &Process)))
      {
         EngSetLastError(ERROR_INVALID_PARAMETER);
         return FALSE;
      }
      ppi = PsGetProcessWin32Process(Process);
      if (!ppi)
      {
         ObDereferenceObject(Process);
         return FALSE;
      }
   }
   ppiCur = PsGetCurrentProcessWin32Process();
   if (!CanForceFG(ppiCur))
   {
      if (Process) ObDereferenceObject(Process);
      EngSetLastError(ERROR_ACCESS_DENIED);
      return FALSE;
   }
   if (dwProcessId == ASFW_ANY)
   {  // All processes will be enabled to set the foreground window.
      //ERR("ptiLastInput is CLEARED!!\n");
      ptiLastInput = NULL;
   }
   else
   {  // Rule #3, last input event in force.
      ERR("ptiLastInput is SET!!\n");
      //ptiLastInput = ppi->ptiList; // See CORE-6384 & CORE-7030.
      ObDereferenceObject(Process);
   }
   return TRUE;
}

/*
 * @implemented
 */
HWND APIENTRY
NtUserGetForegroundWindow(VOID)
{
   HWND Ret;

   TRACE("Enter NtUserGetForegroundWindow\n");
   UserEnterExclusive();

   Ret = UserGetForegroundWindow();

   TRACE("Leave NtUserGetForegroundWindow, ret=%p\n", Ret);
   UserLeave();
   return Ret;
}

HWND APIENTRY
NtUserSetActiveWindow(HWND hWnd)
{
   USER_REFERENCE_ENTRY Ref;
   HWND hWndPrev;
   PWND Window, pwndPrev;
   HWND Ret = NULL;

   TRACE("Enter NtUserSetActiveWindow(%p)\n", hWnd);
   UserEnterExclusive();

   Window = NULL;
   if (hWnd)
   {
      if (!(Window = UserGetWindowObject(hWnd)))
      {
         ERR("NtUserSetActiveWindow: Invalid handle 0x%p!\n",hWnd);
         goto Exit; // Return NULL
      }
   }

   if (!Window ||
        Window->head.pti->MessageQueue == gptiCurrent->MessageQueue)
   {
      pwndPrev = gptiCurrent->MessageQueue->spwndActive;
      hWndPrev = (pwndPrev ? UserHMGetHandle(pwndPrev) : NULL);
      if (Window) UserRefObjectCo(Window, &Ref);
      UserSetActiveWindow(Window);
      if (Window) UserDerefObjectCo(Window);
      Ret = ((hWndPrev && IntIsWindow(hWndPrev)) ? hWndPrev : NULL);
   }

Exit:
   TRACE("Leave NtUserSetActiveWindow, ret=%p\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * @implemented
 */
HWND APIENTRY
NtUserSetCapture(HWND hWnd)
{
   HWND Ret;

   TRACE("Enter NtUserSetCapture(%p)\n", hWnd);
   UserEnterExclusive();

   Ret = co_UserSetCapture(hWnd);

   TRACE("Leave NtUserSetCapture, ret=%p\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * @implemented
 */
HWND APIENTRY
NtUserSetFocus(HWND hWnd)
{
   PWND Window;
   USER_REFERENCE_ENTRY Ref;
   HWND ret = NULL;

   TRACE("Enter NtUserSetFocus(%p)\n", hWnd);
   UserEnterExclusive();

   if (hWnd)
   {
      if (!(Window = UserGetWindowObject(hWnd)))
      {
         ERR("NtUserSetFocus: Invalid handle 0x%p!\n",hWnd);
         goto Exit; // Return NULL
      }

      UserRefObjectCo(Window, &Ref);
      ret = co_UserSetFocus(Window);
      UserDerefObjectCo(Window);
   }
   else
   {
      ret = co_UserSetFocus(NULL);
   }

Exit:
   TRACE("Leave NtUserSetFocus, ret=%p\n", ret);
   UserLeave();
   return ret;
}

/* EOF */
