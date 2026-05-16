/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS Win32k subsystem
 * PURPOSE:          Windows
 * FILE:             win32ss/user/ntuser/window.c
 * PROGRAMERS:       Casper S. Hornstrup (chorns@users.sourceforge.net)
 *                   Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
 */

#include <win32k.h>
#include <safe_user_io.h>
#include <immdev.h>
#include <unaligned.h>

DBG_DEFAULT_CHANNEL(UserWnd);

INT gNestedWindowLimit = 50;

PWINDOWLIST gpwlList = NULL;
PWINDOWLIST gpwlCache = NULL;

/* HELPER FUNCTIONS ***********************************************************/

PVOID FASTCALL
IntReAllocatePoolWithTag(
    POOL_TYPE PoolType,
    PVOID pOld,
    SIZE_T cbOld,
    SIZE_T cbNew,
    ULONG Tag)
{
    PVOID pNew = ExAllocatePoolWithTag(PoolType, cbNew, Tag);
    if (!pNew)
        return NULL;

    RtlCopyMemory(pNew, pOld, min(cbOld, cbNew));
    ExFreePoolWithTag(pOld, Tag);
    return pNew;
}

BOOL FASTCALL UserUpdateUiState(PWND Wnd, WPARAM wParam)
{
    WORD Action = LOWORD(wParam);
    WORD Flags = HIWORD(wParam);

    if (Flags & ~(UISF_HIDEFOCUS | UISF_HIDEACCEL | UISF_ACTIVE))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    switch (Action)
    {
        case UIS_INITIALIZE:
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;

        case UIS_SET:
            if (Flags & UISF_HIDEFOCUS)
                Wnd->HideFocus = TRUE;
            if (Flags & UISF_HIDEACCEL)
                Wnd->HideAccel = TRUE;
            break;

        case UIS_CLEAR:
            if (Flags & UISF_HIDEFOCUS)
                Wnd->HideFocus = FALSE;
            if (Flags & UISF_HIDEACCEL)
                Wnd->HideAccel = FALSE;
            break;
    }

    return TRUE;
}

PWND FASTCALL IntGetWindowObject(HWND hWnd)
{
   PWND Window;

   if (!hWnd) return NULL;

   Window = UserGetWindowObject(hWnd);
   if (Window)
      Window->head.cLockObj++;

   return Window;
}

PWND FASTCALL VerifyWnd(PWND pWnd)
{
    ULONG Error;

    if (!pWnd ||
        (pWnd->state & WNDS_DESTROYED) ||
        (pWnd->state2 & WNDS2_INDESTROY))
    {
        return NULL;
    }

    Error = EngGetLastError();

    if (UserObjectInDestroy(UserHMGetHandle(pWnd)))
        pWnd = NULL;

    EngSetLastError(Error);
    return pWnd;
}

PWND FASTCALL ValidateHwndNoErr(HWND hWnd)
{
    PWND Window;

    if (!hWnd)
        return NULL;

    Window = (PWND)UserGetObjectNoErr(gHandleTable, hWnd, TYPE_WINDOW);
    if (!Window || (Window->state & WNDS_DESTROYED))
        return NULL;

    return Window;
}

/* Temp HACK */
// Win: ValidateHwnd
PWND FASTCALL UserGetWindowObject(HWND hWnd)
{
    PWND Window;

   if (!hWnd)
   {
      EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
      return NULL;
   }

   Window = (PWND)UserGetObject(gHandleTable, hWnd, TYPE_WINDOW);
   if (!Window || 0 != (Window->state & WNDS_DESTROYED))
   {
      EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
      return NULL;
   }

   return Window;
}

ULONG FASTCALL
IntSetStyle( PWND pwnd, ULONG set_bits, ULONG clear_bits )
{
    ULONG styleOld, styleNew;
    styleOld = pwnd->style;
    styleNew = (pwnd->style | set_bits) & ~clear_bits;
    if (styleNew == styleOld) return styleNew;
    pwnd->style = styleNew;
    if ((styleOld ^ styleNew) & WS_VISIBLE) // State Change.
    {
       if (styleOld & WS_VISIBLE) pwnd->head.pti->cVisWindows--;
       if (styleNew & WS_VISIBLE) pwnd->head.pti->cVisWindows++;
       DceResetActiveDCEs( pwnd );
    }
    return styleOld;
}

/*
 * IntIsWindow
 *
 * The function determines whether the specified window handle identifies
 * an existing window.
 *
 * Parameters
 *    hWnd
 *       Handle to the window to test.
 *
 * Return Value
 *    If the window handle identifies an existing window, the return value
 *    is TRUE. If the window handle does not identify an existing window,
 *    the return value is FALSE.
 */

BOOL FASTCALL
IntIsWindow(HWND hWnd)
{
   PWND Window;

   if (!(Window = UserGetWindowObject(hWnd)))
   {
      return FALSE;
   }

   return TRUE;
}

BOOL FASTCALL
IntIsWindowVisible(PWND Wnd)
{
   PWND Temp = Wnd;
   for (;;)
   {
      if (!Temp) return TRUE;
      if (!(Temp->style & WS_VISIBLE)) break;
      if (Temp->style & WS_MINIMIZE && Temp != Wnd) break;
      if (Temp->fnid == FNID_DESKTOP) return TRUE;
      Temp = Temp->spwndParent;
   }
   return FALSE;
}

PWND FASTCALL
IntGetParent(PWND Wnd)
{
   if (Wnd->style & WS_POPUP)
   {
      return Wnd->spwndOwner;
   }
   else if (Wnd->style & WS_CHILD)
   {
      return Wnd->spwndParent;
   }

   return NULL;
}

BOOL
FASTCALL
IntEnableWindow( HWND hWnd, BOOL bEnable )
{
   BOOL Update;
   PWND pWnd;
   UINT bIsDisabled;

   if(!(pWnd = UserGetWindowObject(hWnd)))
   {
      return FALSE;
   }

   /* check if updating is needed */
   bIsDisabled = !!(pWnd->style & WS_DISABLED);
   Update = bIsDisabled;

    if (bEnable)
    {
       IntSetStyle( pWnd, 0, WS_DISABLED );
    }
    else
    {
       Update = !bIsDisabled;

       co_IntSendMessage( hWnd, WM_CANCELMODE, 0, 0);

       /* Remove keyboard focus from that window if it had focus */
       if (hWnd == IntGetThreadFocusWindow())
       {
          TRACE("IntEnableWindow SF NULL\n");
          co_UserSetFocus(NULL);
       }
       IntSetStyle( pWnd, WS_DISABLED, 0 );
    }

    if (Update)
    {
        IntNotifyWinEvent(EVENT_OBJECT_STATECHANGE, pWnd, OBJID_WINDOW, CHILDID_SELF, 0);
        co_IntSendMessage(hWnd, WM_ENABLE, (LPARAM)bEnable, 0);
    }
    // Return nonzero if it was disabled, or zero if it wasn't:
    return bIsDisabled;
}

/*
 * IntWinListChildren
 *
 * Compile a list of all child window handles from given window.
 *
 * Remarks
 *    This function is similar to Wine WIN_ListChildren. The caller
 *    must free the returned list with ExFreePool.
 */

HWND* FASTCALL
IntWinListChildren(PWND Window)
{
    PWND Child;
    HWND *List;
    UINT Index, NumChildren = 0;

    if (!Window) return NULL;

    for (Child = Window->spwndChild; Child; Child = Child->spwndNext)
    {
        ++NumChildren;
    }

    List = ExAllocatePoolWithTag(PagedPool, (NumChildren + 1) * sizeof(HWND), USERTAG_WINDOWLIST);
    if(!List)
    {
        ERR("Failed to allocate memory for children array\n");
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }

    Index = 0;
    for (Child = Window->spwndChild; Child; Child = Child->spwndNext)
    {
        List[Index++] = UserHMGetHandle(Child);
    }
    List[Index] = NULL;

    return List;
}

static BOOL
IntWndIsDefaultIme(_In_ PWND Window)
{
    PTHREADINFO pti = Window->head.pti;

    return (IS_IMM_MODE() && !(pti->TIF_flags & TIF_INCLEANUP) &&
            Window == pti->spwndDefaultIme);
}

HWND* FASTCALL
IntWinListOwnedPopups(PWND Window)
{
    PWND Child, Desktop;
    HWND *List;
    UINT Index, NumOwned = 0;

    Desktop = co_GetDesktopWindow(Window);
    if (!Desktop)
        return NULL;

    for (Child = Desktop->spwndChild; Child; Child = Child->spwndNext)
    {
        if (Child->spwndOwner == Window && !IntWndIsDefaultIme(Child))
            ++NumOwned;
    }

    List = ExAllocatePoolWithTag(PagedPool, (NumOwned + 1) * sizeof(HWND), USERTAG_WINDOWLIST);
    if (!List)
    {
        ERR("Failed to allocate memory for children array\n");
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }

    Index = 0;
    for (Child = Desktop->spwndChild; Child; Child = Child->spwndNext)
    {
        if (Child->spwndOwner == Window && !IntWndIsDefaultIme(Child))
            List[Index++] = UserHMGetHandle(Child);
    }
    List[Index] = NULL;

    return List;
}

PWND FASTCALL
IntGetNonChildAncestor(PWND pWnd)
{
   while(pWnd && (pWnd->style & (WS_CHILD | WS_POPUP)) == WS_CHILD)
      pWnd = pWnd->spwndParent;
   return pWnd;
}

BOOL FASTCALL
IntIsTopLevelWindow(PWND pWnd)
{
   if ( pWnd->spwndParent &&
        pWnd->spwndParent == co_GetDesktopWindow(pWnd) ) return TRUE;
   return FALSE;
}

BOOL FASTCALL
IntValidateOwnerDepth(PWND Wnd, PWND Owner)
{
   INT Depth = 1;
   for (;;)
   {
      if ( !Owner ) return gNestedWindowLimit >= Depth;
      if (Owner == Wnd) break;
      Owner = Owner->spwndOwner;
      Depth++;
   }
   return FALSE;
}

HWND FASTCALL
IntGetWindow(HWND hWnd,
          UINT uCmd)
{
    PWND Wnd, FoundWnd;
    HWND Ret = NULL;

    Wnd = ValidateHwndNoErr(hWnd);
    if (!Wnd)
        return NULL;

    FoundWnd = NULL;
    switch (uCmd)
    {
            case GW_OWNER:
                if (Wnd->spwndOwner != NULL)
                    FoundWnd = Wnd->spwndOwner;
                break;

            case GW_HWNDFIRST:
                if(Wnd->spwndParent != NULL)
                {
                    FoundWnd = Wnd->spwndParent;
                    if (FoundWnd->spwndChild != NULL)
                        FoundWnd = FoundWnd->spwndChild;
                }
                break;
            case GW_HWNDNEXT:
                if (Wnd->spwndNext != NULL)
                    FoundWnd = Wnd->spwndNext;
                break;

            case GW_HWNDPREV:
                if (Wnd->spwndPrev != NULL)
                    FoundWnd = Wnd->spwndPrev;
                break;

            case GW_CHILD:
                if (Wnd->spwndChild != NULL)
                    FoundWnd = Wnd->spwndChild;
                break;

            case GW_HWNDLAST:
                FoundWnd = Wnd;
                while ( FoundWnd->spwndNext != NULL)
                    FoundWnd = FoundWnd->spwndNext;
                break;

            default:
                EngSetLastError(ERROR_INVALID_GW_COMMAND);
                break;
    }

    if (FoundWnd != NULL)
        Ret = UserHMGetHandle(FoundWnd);
    return Ret;
}

DWORD FASTCALL IntGetWindowContextHelpId( PWND pWnd )
{
   DWORD HelpId;

   do
   {
      HelpId = HandleToUlong(UserGetProp(pWnd, gpsi->atomContextHelpIdProp, TRUE));
      if (!HelpId) break;
      pWnd = IntGetParent(pWnd);
   }
   while (pWnd && pWnd->fnid != FNID_DESKTOP);
   return HelpId;
}


VOID
FASTCALL
IntRemoveTrackMouseEvent(
    PDESKTOP pDesk);

/***********************************************************************
 *           IntSendDestroyMsg
 */
static void IntSendDestroyMsg(HWND hWnd)
{
   PTHREADINFO ti;
   PWND Window;

   ti = PsGetCurrentThreadWin32Thread();
   Window = UserGetWindowObject(hWnd);

   if (Window)
   {
      /*
       * Look whether the focus is within the tree of windows
       * we will be destroying.
       */
      // Rule #1
      if ( ti->MessageQueue->spwndActive == Window || // Fixes CORE-106 RegSvr32 exit and return focus to CMD.
          (ti->MessageQueue->spwndActive == NULL && ti->MessageQueue == IntGetFocusMessageQueue()) )
      {
         co_WinPosActivateOtherWindow(Window);
      }

      /* Fixes CMD properties closing and returning focus to CMD */
      if (ti->MessageQueue->spwndFocus == Window)
      {
         if ((Window->style & (WS_CHILD | WS_POPUP)) == WS_CHILD)
         {
            co_UserSetFocus(Window->spwndParent);
         }
         else
         {
            co_UserSetFocus(NULL);
         }
      }

      if (ti->MessageQueue->CaretInfo.hWnd == UserHMGetHandle(Window))
      {
         co_IntDestroyCaret(ti);
      }

      /* If the window being destroyed is currently tracked... */
      if (ti->rpdesk && ti->rpdesk->spwndTrack == Window)
      {
          IntRemoveTrackMouseEvent(ti->rpdesk);
      }
   }

   /* If the window being destroyed is the current clipboard owner... */
   if (ti->ppi->prpwinsta != NULL && Window == ti->ppi->prpwinsta->spwndClipOwner)
   {
       /* ... make it release the clipboard */
       UserClipboardRelease(Window);
   }

   /* Send the WM_DESTROY to the window */
   co_IntSendMessage(hWnd, WM_DESTROY, 0, 0);

   /*
    * This WM_DESTROY message can trigger re-entrant calls to DestroyWindow
    * make sure that the window still exists when we come back.
    */
   if (IntIsWindow(hWnd))
   {
      HWND* pWndArray;
      int i;

      if (!(pWndArray = IntWinListChildren( Window ))) return;

      for (i = 0; pWndArray[i]; i++)
      {
         if (IntIsWindow( pWndArray[i] )) IntSendDestroyMsg( pWndArray[i] );
      }
      ExFreePoolWithTag(pWndArray, USERTAG_WINDOWLIST);
   }
   else
   {
      TRACE("destroyed itself while in WM_DESTROY!\n");
   }
}

static VOID
UserFreeWindowInfo(PTHREADINFO ti, PWND Wnd)
{
    PCLIENTINFO ClientInfo = GetWin32ClientInfo();

    if (!Wnd) return;

    if (ClientInfo->CallbackWnd.pWnd == DesktopHeapAddressToUser(Wnd))
    {
        ClientInfo->CallbackWnd.hWnd = NULL;
        ClientInfo->CallbackWnd.pWnd = NULL;
    }

   if (Wnd->strName.Buffer != NULL)
   {
       Wnd->strName.Length = 0;
       Wnd->strName.MaximumLength = 0;
       DesktopHeapFree(Wnd->head.rpdesk,
                       Wnd->strName.Buffer);
       Wnd->strName.Buffer = NULL;
   }

//    DesktopHeapFree(Wnd->head.rpdesk, Wnd);
//    WindowObject->hWnd = NULL;
}

/***********************************************************************
 *           co_UserFreeWindow
 *
 * Destroy storage associated to a window. "Internals" p.358
 *
 * This is the "functional" DestroyWindows function i.e. all stuff
 * done in CreateWindow is undone here and not in DestroyWindow :-P
 */
LRESULT co_UserFreeWindow(PWND Window,
                          PPROCESSINFO ProcessData,
                          PTHREADINFO ThreadData,
                          BOOLEAN SendMessages)
{
   HWND *Children;
   HWND *ChildHandle;
   PWND Child;
   PMENU Menu;
   BOOLEAN BelongsToThreadData;
   USER_REFERENCE_ENTRY Ref;

   ASSERT(Window);

   if(Window->state2 & WNDS2_INDESTROY)
   {
      TRACE("Tried to call co_UserFreeWindow() twice\n");
      return 0;
   }
   Window->state2 |= WNDS2_INDESTROY;
   Window->style &= ~WS_VISIBLE;
   Window->head.pti->cVisWindows--;

   /* remove the window already at this point from the thread window list so we
      don't get into trouble when destroying the thread windows while we're still
      in co_UserFreeWindow() */
   if (!IsListEmpty(&Window->ThreadListEntry))
       RemoveEntryList(&Window->ThreadListEntry);

   BelongsToThreadData = IntWndBelongsToThread(Window, ThreadData);

   IntDeRegisterShellHookWindow(UserHMGetHandle(Window));

   /* free child windows */
   Children = IntWinListChildren(Window);
   if (Children)
   {
      for (ChildHandle = Children; *ChildHandle; ++ChildHandle)
      {
         if ((Child = IntGetWindowObject(*ChildHandle)))
         {
            if (!IntWndBelongsToThread(Child, ThreadData))
            {
               /* send WM_DESTROY messages to windows not belonging to the same thread */
               co_IntSendMessage( UserHMGetHandle(Child), WM_ASYNC_DESTROYWINDOW, 0, 0 );
            }
            else
               co_UserFreeWindow(Child, ProcessData, ThreadData, SendMessages);

            UserDereferenceObject(Child);
         }
      }
      ExFreePoolWithTag(Children, USERTAG_WINDOWLIST);
   }

   if (SendMessages)
   {
      /*
       * Clear the update region to make sure no WM_PAINT messages will be
       * generated for this window while processing the WM_NCDESTROY.
       */
      co_UserRedrawWindow(Window, NULL, 0,
                          RDW_VALIDATE | RDW_NOFRAME | RDW_NOERASE |
                          RDW_NOINTERNALPAINT | RDW_NOCHILDREN);
      if (BelongsToThreadData)
         co_IntSendMessage(UserHMGetHandle(Window), WM_NCDESTROY, 0, 0);
   }

   UserClipboardFreeWindow(Window);

   DestroyTimersForWindow(ThreadData, Window);

   /* Unregister hot keys */
   UnregisterWindowHotKeys(Window);

   /* flush the message queue */
   MsqRemoveWindowMessagesFromQueue(Window);

   /* from now on no messages can be sent to this window anymore */
   Window->state |= WNDS_DESTROYED;
   Window->fnid |= FNID_FREED;

   /* don't remove the WINDOWSTATUS_DESTROYING bit */

   /* reset shell window handles */
   if (ThreadData->rpdesk)
   {
      if (UserHMGetHandle(Window) == ThreadData->rpdesk->rpwinstaParent->ShellWindow)
         ThreadData->rpdesk->rpwinstaParent->ShellWindow = NULL;

      if (UserHMGetHandle(Window) == ThreadData->rpdesk->rpwinstaParent->ShellListView)
         ThreadData->rpdesk->rpwinstaParent->ShellListView = NULL;
   }

   if (ThreadData->spwndDefaultIme &&
       ThreadData->spwndDefaultIme->spwndOwner == Window)
   {
      WndSetOwner(ThreadData->spwndDefaultIme, NULL);
   }

   if (IS_IMM_MODE() && Window == ThreadData->spwndDefaultIme)
   {
      UserAssignmentUnlock((PVOID*)&(ThreadData->spwndDefaultIme));
   }

   /* Fixes dialog test_focus breakage due to r66237. */
   if (ThreadData->MessageQueue->spwndFocus == Window)
      ThreadData->MessageQueue->spwndFocus = NULL;

   if (ThreadData->MessageQueue->spwndActive == Window)
      ThreadData->MessageQueue->spwndActive = NULL;

   if (ThreadData->MessageQueue->spwndCapture == Window)
   {
      IntReleaseCapture();
   }

   //// Now kill those remaining "PAINTING BUG: Thread marked as containing dirty windows" spam!!!
   if ( Window->hrgnUpdate != NULL || Window->state & WNDS_INTERNALPAINT )
   {
      MsqDecPaintCountQueue(Window->head.pti);
      if (Window->hrgnUpdate > HRGN_WINDOW && GreIsHandleValid(Window->hrgnUpdate))
      {
         IntGdiSetRegionOwner(Window->hrgnUpdate, GDI_OBJ_HMGR_POWNED);
         GreDeleteObject(Window->hrgnUpdate);
      }
      Window->hrgnUpdate = NULL;
      Window->state &= ~WNDS_INTERNALPAINT;
   }

   if (Window->state & (WNDS_SENDERASEBACKGROUND|WNDS_SENDNCPAINT))
   {
      Window->state &= ~(WNDS_SENDERASEBACKGROUND|WNDS_SENDNCPAINT);
   }

   if ( ((Window->style & (WS_CHILD|WS_POPUP)) != WS_CHILD) &&
        Window->IDMenu &&
        (Menu = UserGetMenuObject((HMENU)Window->IDMenu)))
   {
      TRACE("UFW: IDMenu %p\n",Window->IDMenu);
      IntDestroyMenuObject(Menu, TRUE);
      Window->IDMenu = 0;
   }

   if (Window->SystemMenu
        && (Menu = UserGetMenuObject(Window->SystemMenu)))
   {
      IntDestroyMenuObject(Menu, TRUE);
      Window->SystemMenu = (HMENU)0;
   }

   DceFreeWindowDCE(Window);    /* Always do this to catch orphaned DCs */

   IntUnlinkWindow(Window);

   if (Window->PropListItems)
   {
      UserRemoveWindowProps(Window);
      TRACE("Window->PropListItems %lu\n",Window->PropListItems);
      ASSERT(Window->PropListItems==0);
   }

   /* Kill any reference to linked windows. Prev & Next are taken care of in IntUnlinkWindow */
   WndSetOwner(Window, NULL);
   WndSetParent(Window, NULL);
   WndSetChild(Window, NULL);
   WndSetLastActive(Window, NULL);

   UserRefObjectCo(Window, &Ref);
   UserMarkObjectDestroy(Window);

   IntDestroyScrollBars(Window);

   if (Window->pcls->atomClassName == gaGuiConsoleWndClass)
   {
       /* Count only console windows manually */
       co_IntUserManualGuiCheck(FALSE);
   }

   /* dereference the class */
   NT_ASSERT(Window->head.pti != NULL);
   IntDereferenceClass(Window->pcls,
                       Window->head.pti->pDeskInfo,
                       Window->head.pti->ppi);
   Window->pcls = NULL;

   if (Window->hrgnClip)
   {
      IntGdiSetRegionOwner(Window->hrgnClip, GDI_OBJ_HMGR_POWNED);
      GreDeleteObject(Window->hrgnClip);
      Window->hrgnClip = NULL;
   }
   Window->head.pti->cWindows--;

//   ASSERT(Window != NULL);
   UserFreeWindowInfo(Window->head.pti, Window);

   UserDerefObjectCo(Window);
   UserDeleteObject(UserHMGetHandle(Window), TYPE_WINDOW);

   return 0;
}

//
// Same as User32:IntGetWndProc.
//
WNDPROC FASTCALL
IntGetWindowProc(PWND pWnd,
                 BOOL Ansi)
{
   INT i;
   PCLS Class;
   WNDPROC gcpd, Ret = 0;

   ASSERT(UserIsEnteredExclusive());

   Class = pWnd->pcls;

   if (pWnd->state & WNDS_SERVERSIDEWINDOWPROC)
   {
      for ( i = FNID_FIRST; i <= FNID_SWITCH; i++)
      {
         if (GETPFNSERVER(i) == pWnd->lpfnWndProc)
         {
            if (Ansi)
               Ret = GETPFNCLIENTA(i);
            else
               Ret = GETPFNCLIENTW(i);
         }
      }
      return Ret;
   }

   if (Class->fnid == FNID_EDIT)
      Ret = pWnd->lpfnWndProc;
   else
   {
      Ret = pWnd->lpfnWndProc;

      if (Class->fnid <= FNID_GHOST && Class->fnid >= FNID_BUTTON)
      {
         if (Ansi)
         {
            if (GETPFNCLIENTW(Class->fnid) == pWnd->lpfnWndProc)
               Ret = GETPFNCLIENTA(Class->fnid);
         }
         else
         {
            if (GETPFNCLIENTA(Class->fnid) == pWnd->lpfnWndProc)
               Ret = GETPFNCLIENTW(Class->fnid);
         }
      }
      if ( Ret != pWnd->lpfnWndProc)
         return Ret;
   }
   if ( Ansi == !!(pWnd->state & WNDS_ANSIWINDOWPROC) )
      return Ret;

   gcpd = (WNDPROC)UserGetCPD(
                       pWnd,
                      (Ansi ? UserGetCPDA2U : UserGetCPDU2A )|UserGetCPDWindow,
                      (ULONG_PTR)Ret);

   return (gcpd ? gcpd : Ret);
}

static WNDPROC
IntSetWindowProc(PWND pWnd,
                 WNDPROC NewWndProc,
                 BOOL Ansi)
{
   INT i;
   PCALLPROCDATA CallProc;
   PCLS Class;
   WNDPROC Ret, chWndProc = NULL;

   // Retrieve previous window proc.
   Ret = IntGetWindowProc(pWnd, Ansi);

   Class = pWnd->pcls;

   if (IsCallProcHandle(NewWndProc))
   {
      CallProc = UserGetObject(gHandleTable, NewWndProc, TYPE_CALLPROC);
      if (CallProc)
      {  // Reset new WndProc.
         NewWndProc = CallProc->pfnClientPrevious;
         // Reset Ansi from CallProc handle. This is expected with wine "deftest".
         Ansi = !!(CallProc->wType & UserGetCPDU2A);
      }
   }
   // Switch from Client Side call to Server Side call if match. Ref: "deftest".
   for ( i = FNID_FIRST; i <= FNID_SWITCH; i++)
   {
       if (GETPFNCLIENTW(i) == NewWndProc)
       {
          chWndProc = GETPFNSERVER(i);
          break;
       }
       if (GETPFNCLIENTA(i) == NewWndProc)
       {
          chWndProc = GETPFNSERVER(i);
          break;
       }
   }
   // If match, set/reset to Server Side and clear ansi.
   if (chWndProc)
   {
      pWnd->lpfnWndProc = chWndProc;
      pWnd->Unicode = TRUE;
      pWnd->state &= ~WNDS_ANSIWINDOWPROC;
      pWnd->state |= WNDS_SERVERSIDEWINDOWPROC;
   }
   else
   {
      pWnd->Unicode = !Ansi;
      // Handle the state change in here.
      if (Ansi)
         pWnd->state |= WNDS_ANSIWINDOWPROC;
      else
         pWnd->state &= ~WNDS_ANSIWINDOWPROC;

      if (pWnd->state & WNDS_SERVERSIDEWINDOWPROC)
         pWnd->state &= ~WNDS_SERVERSIDEWINDOWPROC;

      if (!NewWndProc) NewWndProc = pWnd->lpfnWndProc;

      if (Class->fnid <= FNID_GHOST && Class->fnid >= FNID_BUTTON)
      {
         if (Ansi)
         {
            if (GETPFNCLIENTW(Class->fnid) == NewWndProc)
               chWndProc = GETPFNCLIENTA(Class->fnid);
         }
         else
         {
            if (GETPFNCLIENTA(Class->fnid) == NewWndProc)
               chWndProc = GETPFNCLIENTW(Class->fnid);
         }
      }
      // Now set the new window proc.
      pWnd->lpfnWndProc = (chWndProc ? chWndProc : NewWndProc);
   }
   return Ret;
}


/* INTERNAL ******************************************************************/

////
//   This fixes a check for children messages that need paint while searching the parents messages!
//   Fixes wine msg:test_paint_messages:WmParentErasePaint ..
////
BOOL FASTCALL
IntIsChildWindow(PWND Parent, PWND BaseWindow)
{
   PWND Window = BaseWindow;
   do
   {
     if ( Window == NULL || (Window->style & (WS_POPUP|WS_CHILD)) != WS_CHILD )
        return FALSE;

     Window = Window->spwndParent;
   }
   while(Parent != Window);
   return TRUE;
}
////

/* Link the window into siblings list. Children and parent are kept in place. */
VOID FASTCALL
IntLinkWindow(
    PWND Wnd,
    PWND WndInsertAfter /* Set to NULL if top sibling */
)
{
    if (Wnd == WndInsertAfter)
    {
        ERR("Trying to link window 0x%p to itself\n", Wnd);
        ASSERT(WndInsertAfter != Wnd);
        return;
    }

    WndSetPrev(Wnd, WndInsertAfter);
    if (Wnd->spwndPrev)
    {
        /* Link after WndInsertAfter */
        ASSERT(Wnd != WndInsertAfter->spwndNext);
        WndSetNext(Wnd, WndInsertAfter->spwndNext);
        if (Wnd->spwndNext)
            WndSetPrev(Wnd->spwndNext, Wnd);

        ASSERT(Wnd != Wnd->spwndPrev);
        WndSetNext(Wnd->spwndPrev, Wnd);
    }
    else
    {
        /* Link at the top */
        ASSERT(Wnd != Wnd->spwndParent->spwndChild);
        WndSetNext(Wnd, Wnd->spwndParent->spwndChild);
        if (Wnd->spwndNext)
            WndSetPrev(Wnd->spwndNext, Wnd);

        WndSetChild(Wnd->spwndParent, Wnd);
    }
}

/*
 Note: Wnd->spwndParent can be null if it is the desktop.
*/
VOID FASTCALL IntLinkHwnd(PWND Wnd, HWND hWndPrev)
{
    if (hWndPrev == HWND_NOTOPMOST)
    {
        if (!(Wnd->ExStyle & WS_EX_TOPMOST) && (Wnd->ExStyle2 & WS_EX2_LINKED))
            return;  /* nothing to do */
        Wnd->ExStyle &= ~WS_EX_TOPMOST;
        hWndPrev = HWND_TOP;  /* fallback to the HWND_TOP case */
    }

    IntUnlinkWindow(Wnd);  /* unlink it from the previous location */

    if (hWndPrev == HWND_BOTTOM)
    {
        /* Link in the bottom of the list */
        PWND WndInsertAfter;

        WndInsertAfter = Wnd->spwndParent->spwndChild;
        while (WndInsertAfter && WndInsertAfter->spwndNext)
        {
            WndInsertAfter = WndInsertAfter->spwndNext;
        }

        IntLinkWindow(Wnd, WndInsertAfter);
        Wnd->ExStyle &= ~WS_EX_TOPMOST;
    }
    else if (hWndPrev == HWND_TOPMOST)
    {
        /* Link in the top of the list */
        IntLinkWindow(Wnd, NULL);
        Wnd->ExStyle |= WS_EX_TOPMOST;
    }
    else if (hWndPrev == HWND_TOP)
    {
        /* Link it after the last topmost window */
        PWND WndInsertBefore;

        WndInsertBefore = Wnd->spwndParent->spwndChild;

        if (!(Wnd->ExStyle & WS_EX_TOPMOST))  /* put it above the first non-topmost window */
        {
            while (WndInsertBefore != NULL && WndInsertBefore->spwndNext != NULL)
            {
                if (!(WndInsertBefore->ExStyle & WS_EX_TOPMOST))
                    break;

                if (WndInsertBefore == Wnd->spwndOwner)  /* keep it above owner */
                {
                    Wnd->ExStyle |= WS_EX_TOPMOST;
                    break;
                }
                WndInsertBefore = WndInsertBefore->spwndNext;
            }
        }

        IntLinkWindow(Wnd, WndInsertBefore ? WndInsertBefore->spwndPrev : NULL);
    }
    else
    {
        /* Link it after hWndPrev */
        PWND WndInsertAfter;

        WndInsertAfter = UserGetWindowObject(hWndPrev);
        /* Are we called with an erroneous handle */
        if (WndInsertAfter == NULL)
        {
            /* Link in a default position */
            IntLinkHwnd(Wnd, HWND_TOP);
            return;
        }

        if (Wnd == WndInsertAfter)
        {
            ERR("Trying to link window 0x%p to itself\n", Wnd);
            ASSERT(WndInsertAfter != Wnd);
            // FIXME: IntUnlinkWindow(Wnd) was already called. Continuing as is seems wrong!
        }
        else
        {
            IntLinkWindow(Wnd, WndInsertAfter);
        }

        /* Fix the WS_EX_TOPMOST flag */
        if (!(WndInsertAfter->ExStyle & WS_EX_TOPMOST))
        {
            Wnd->ExStyle &= ~WS_EX_TOPMOST;
        }
        else
        {
            if (WndInsertAfter->spwndNext &&
                (WndInsertAfter->spwndNext->ExStyle & WS_EX_TOPMOST))
            {
                Wnd->ExStyle |= WS_EX_TOPMOST;
            }
        }
    }
    Wnd->ExStyle2 |= WS_EX2_LINKED;
}
/* --- pretranslation split: the remaining function groups live in window.split2..5.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "window.split2.c"
#include "window.split3.c"
#include "window.split4.c"
#include "window.split5.c"
