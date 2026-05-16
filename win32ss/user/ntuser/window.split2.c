/* window.split2.c — #include fragment of window.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original window.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


VOID FASTCALL
IntProcessOwnerSwap(PWND Wnd, PWND WndNewOwner, PWND WndOldOwner)
{
   if (WndOldOwner)
   {
      if (Wnd->head.pti != WndOldOwner->head.pti)
      {
         if (!WndNewOwner ||
              Wnd->head.pti == WndNewOwner->head.pti ||
              WndOldOwner->head.pti != WndNewOwner->head.pti )
         {
            //ERR("ProcessOwnerSwap Old out.\n");
            UserAttachThreadInput(Wnd->head.pti, WndOldOwner->head.pti, FALSE);
         }
      }
   }
   if (WndNewOwner)
   {
      if (Wnd->head.pti != WndNewOwner->head.pti)
      {
         if (!WndOldOwner ||
              WndOldOwner->head.pti != WndNewOwner->head.pti )
         {
            //ERR("ProcessOwnerSwap New in.\n");
            UserAttachThreadInput(Wnd->head.pti, WndNewOwner->head.pti, TRUE);
         }
      }
   }
   // FIXME: System Tray checks.
}

static
HWND FASTCALL
IntSetOwner(HWND hWnd, HWND hWndNewOwner)
{
   PWND Wnd, WndOldOwner, WndNewOwner;
   HWND ret;

   Wnd = IntGetWindowObject(hWnd);
   if(!Wnd)
      return NULL;

   WndOldOwner = Wnd->spwndOwner;

   ret = WndOldOwner ? UserHMGetHandle(WndOldOwner) : 0;
   WndNewOwner = UserGetWindowObject(hWndNewOwner);

   if (!WndNewOwner && hWndNewOwner)
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      ret = NULL;
      goto Error;
   }

   /* if parent belongs to a different thread and the window isn't */
   /* top-level, attach the two threads */
   IntProcessOwnerSwap(Wnd, WndNewOwner, WndOldOwner);

   if (IntValidateOwnerDepth(Wnd, WndNewOwner))
   {
      WndSetOwner(Wnd, WndNewOwner);
   }
   else
   {
      IntProcessOwnerSwap(Wnd, WndOldOwner, WndNewOwner);
      EngSetLastError(ERROR_INVALID_PARAMETER);
      ret = NULL;
   }
Error:
   UserDereferenceObject(Wnd);
   return ret;
}

PWND FASTCALL
co_IntSetParent(PWND Wnd, PWND WndNewParent)
{
   PWND WndOldParent, pWndExam;
   BOOL WasVisible;
   POINT pt;
   int swFlags = SWP_NOSIZE|SWP_NOZORDER;

   ASSERT(Wnd);
   ASSERT(WndNewParent);
   ASSERT_REFS_CO(Wnd);
   ASSERT_REFS_CO(WndNewParent);

   if (Wnd == Wnd->head.rpdesk->spwndMessage)
   {
      EngSetLastError(ERROR_ACCESS_DENIED);
      return NULL;
   }

   /* Some applications try to set a child as a parent */
   if (IntIsChildWindow(Wnd, WndNewParent))
   {
      TRACE("IntSetParent try to set a child as a parent.\n");
      EngSetLastError( ERROR_INVALID_PARAMETER );
      return NULL;
   }

   pWndExam = WndNewParent; // Load parent Window to examine.
   // Now test for set parent to parent hit.
   while (pWndExam)
   {
      if (Wnd == pWndExam)
      {
         TRACE("IntSetParent Failed Test for set parent to parent!\n");
         EngSetLastError(ERROR_INVALID_PARAMETER);
         return NULL;
      }
      pWndExam = pWndExam->spwndParent;
   }

   /*
    * Windows hides the window first, then shows it again
    * including the WM_SHOWWINDOW messages and all
    */
   WasVisible = co_WinPosShowWindow(Wnd, SW_HIDE);

   /* Window must belong to current process */
   if (Wnd->head.pti->ppi != PsGetCurrentProcessWin32Process())
   {
      ERR("IntSetParent Window must belong to current process!\n");
      return NULL;
   }

   WndOldParent = Wnd->spwndParent;

   if ( WndOldParent &&
        WndOldParent->ExStyle & WS_EX_LAYOUTRTL)
      pt.x = Wnd->rcWindow.right;
   else
      pt.x = Wnd->rcWindow.left;
   pt.y = Wnd->rcWindow.top;

   IntScreenToClient(WndOldParent, &pt);

   if (WndOldParent) UserReferenceObject(WndOldParent); /* Caller must deref */

   /* Even if WndNewParent == WndOldParent continue because the
    * child window (Wnd) should be moved to the top of the z-order */

   /* Unlink the window from the siblings list */
   IntUnlinkWindow(Wnd);
   Wnd->ExStyle2 &= ~WS_EX2_LINKED;

   /* Set the new parent */
   WndSetParent(Wnd, WndNewParent);

   if (Wnd->style & WS_CHILD &&
       Wnd->spwndOwner &&
       Wnd->spwndOwner->ExStyle & WS_EX_TOPMOST)
   {
      ERR("SetParent Top Most from Pop up\n");
      Wnd->ExStyle |= WS_EX_TOPMOST;
   }

   /* Link the window with its new siblings */
   IntLinkHwnd(Wnd,
               ((0 == (Wnd->ExStyle & WS_EX_TOPMOST) &&
               UserIsDesktopWindow(WndNewParent)) ? HWND_TOP : HWND_TOPMOST));

   if ( WndNewParent == co_GetDesktopWindow(Wnd) &&
       !(Wnd->style & WS_CLIPSIBLINGS) )
   {
      Wnd->style |= WS_CLIPSIBLINGS;
      DceResetActiveDCEs(Wnd);
   }

   /* if parent belongs to a different thread and the window isn't */
   /* top-level, attach the two threads */
   if ((Wnd->style & (WS_CHILD|WS_POPUP)) == WS_CHILD)
   {
      if ( Wnd->spwndParent != co_GetDesktopWindow(Wnd))
      {
         if (WndOldParent && (Wnd->head.pti != WndOldParent->head.pti))
         {
            //ERR("SetParent Old out.\n");
            UserAttachThreadInput(Wnd->head.pti, WndOldParent->head.pti, FALSE);
         }
      }
      if ( WndNewParent != co_GetDesktopWindow(Wnd))
      {
         if (Wnd->head.pti != WndNewParent->head.pti)
         {
            //ERR("SetParent New in.\n");
            UserAttachThreadInput(Wnd->head.pti, WndNewParent->head.pti, TRUE);
         }
      }
   }

   if (UserIsMessageWindow(WndOldParent) || UserIsMessageWindow(WndNewParent))
      swFlags |= SWP_NOACTIVATE;

   IntNotifyWinEvent(EVENT_OBJECT_PARENTCHANGE, Wnd ,OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
   /*
    * SetParent additionally needs to make hwnd the top window
    * in the z-order and send the expected WM_WINDOWPOSCHANGING and
    * WM_WINDOWPOSCHANGED notification messages.
    */
   //ERR("IntSetParent SetWindowPos 1\n");
   co_WinPosSetWindowPos( Wnd,
                         (0 == (Wnd->ExStyle & WS_EX_TOPMOST) ? HWND_TOP : HWND_TOPMOST),
                          pt.x, pt.y, 0, 0, swFlags);
   //ERR("IntSetParent SetWindowPos 2 X %d Y %d\n",pt.x, pt.y);
   if (WasVisible) co_WinPosShowWindow(Wnd, SW_SHOWNORMAL);

   return WndOldParent;
}

// Win: xxxSetParent
HWND FASTCALL
co_UserSetParent(HWND hWndChild, HWND hWndNewParent)
{
   PWND Wnd = NULL, WndParent = NULL, WndOldParent;
   HWND hWndOldParent = NULL;
   USER_REFERENCE_ENTRY Ref, ParentRef;

   if (IntIsBroadcastHwnd(hWndChild) || IntIsBroadcastHwnd(hWndNewParent))
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      return NULL;
   }

   if (hWndChild == IntGetDesktopWindow())
   {
      ERR("UserSetParent Access Denied!\n");
      EngSetLastError(ERROR_ACCESS_DENIED);
      return NULL;
   }

   if (hWndNewParent)
   {
      if (!(WndParent = UserGetWindowObject(hWndNewParent)))
      {
         ERR("UserSetParent Bad New Parent!\n");
         return NULL;
      }
   }
   else
   {
      if (!(WndParent = UserGetWindowObject(IntGetDesktopWindow())))
      {
         return NULL;
      }
   }

   if (!(Wnd = UserGetWindowObject(hWndChild)))
   {
      ERR("UserSetParent Bad Child!\n");
      return NULL;
   }

   UserRefObjectCo(Wnd, &Ref);
   UserRefObjectCo(WndParent, &ParentRef);
   //ERR("Enter co_IntSetParent\n");
   WndOldParent = co_IntSetParent(Wnd, WndParent);
   //ERR("Leave co_IntSetParent\n");
   UserDerefObjectCo(WndParent);
   UserDerefObjectCo(Wnd);

   if (WndOldParent)
   {
      hWndOldParent = UserHMGetHandle(WndOldParent);
      UserDereferenceObject(WndOldParent);
   }

   return hWndOldParent;
}

/* Unlink the window from siblings. Children and parent are kept in place. */
VOID FASTCALL
IntUnlinkWindow(PWND Wnd)
{
    ASSERT(Wnd != Wnd->spwndNext);
    ASSERT(Wnd != Wnd->spwndPrev);

    if (Wnd->spwndNext)
        WndSetPrev(Wnd->spwndNext, Wnd->spwndPrev);

    if (Wnd->spwndPrev)
        WndSetNext(Wnd->spwndPrev, Wnd->spwndNext);

    if (Wnd->spwndParent && Wnd->spwndParent->spwndChild == Wnd)
        WndSetChild(Wnd->spwndParent, Wnd->spwndNext);

    WndSetPrev(Wnd, NULL);
    WndSetNext(Wnd, NULL);
}

// Win: ExpandWindowList
BOOL FASTCALL IntGrowHwndList(PWINDOWLIST *ppwl)
{
    PWINDOWLIST pwlOld, pwlNew;
    SIZE_T ibOld, ibNew;

#define GROW_COUNT 8
    pwlOld = *ppwl;
    ibOld = (LPBYTE)pwlOld->phwndLast - (LPBYTE)pwlOld;
    ibNew = ibOld + GROW_COUNT * sizeof(HWND);
#undef GROW_COUNT
    pwlNew = IntReAllocatePoolWithTag(PagedPool, pwlOld, ibOld, ibNew, USERTAG_WINDOWLIST);
    if (!pwlNew)
        return FALSE;

    pwlNew->phwndLast = (HWND *)((LPBYTE)pwlNew + ibOld);
    pwlNew->phwndEnd = (HWND *)((LPBYTE)pwlNew + ibNew);
    *ppwl = pwlNew;
    return TRUE;
}

// Win: InternalBuildHwndList
PWINDOWLIST FASTCALL IntPopulateHwndList(PWINDOWLIST pwl, PWND pwnd, DWORD dwFlags)
{
    ASSERT(!WL_IS_BAD(pwl));

    for (; pwnd; pwnd = pwnd->spwndNext)
    {
        if (!pwl->pti || pwl->pti == pwnd->head.pti)
        {
            *(pwl->phwndLast) = UserHMGetHandle(pwnd);
            ++(pwl->phwndLast);

            if (pwl->phwndLast == pwl->phwndEnd && !IntGrowHwndList(&pwl))
                break;
        }

        if ((dwFlags & IACE_CHILDREN) && pwnd->spwndChild)
        {
            pwl = IntPopulateHwndList(pwl, pwnd->spwndChild, IACE_CHILDREN | IACE_LIST);
            if (WL_IS_BAD(pwl))
                break;
        }

        if (!(dwFlags & IACE_LIST))
            break;
    }

    return pwl;
}

// Win: BuildHwndList
PWINDOWLIST FASTCALL IntBuildHwndList(PWND pwnd, DWORD dwFlags, PTHREADINFO pti)
{
    PWINDOWLIST pwl;
    DWORD cbWL;

    if (gpwlCache)
    {
        pwl = gpwlCache;
        gpwlCache = NULL;
    }
    else
    {
#define INITIAL_COUNT 32
        cbWL = sizeof(WINDOWLIST) + (INITIAL_COUNT - 1) * sizeof(HWND);
        pwl = ExAllocatePoolWithTag(PagedPool, cbWL, USERTAG_WINDOWLIST);
        if (!pwl)
            return NULL;

        pwl->phwndEnd = &pwl->ahwnd[INITIAL_COUNT];
#undef INITIAL_COUNT
    }

    pwl->pti = pti;
    pwl->phwndLast = pwl->ahwnd;
    pwl = IntPopulateHwndList(pwl, pwnd, dwFlags);
    if (WL_IS_BAD(pwl))
    {
        ExFreePoolWithTag(pwl, USERTAG_WINDOWLIST);
        return NULL;
    }

    *(pwl->phwndLast) = HWND_TERMINATOR;

    if (dwFlags & 0x8)
    {
        // TODO:
    }

    pwl->pti = GetW32ThreadInfo();
    pwl->pNextList = gpwlList;
    gpwlList = pwl;

    return pwl;
}

// Win: FreeHwndList
VOID FASTCALL IntFreeHwndList(PWINDOWLIST pwlTarget)
{
    PWINDOWLIST pwl, *ppwl;

    for (ppwl = &gpwlList; *ppwl; ppwl = &(*ppwl)->pNextList)
    {
        if (*ppwl != pwlTarget)
            continue;

        *ppwl = pwlTarget->pNextList;

        if (gpwlCache)
        {
            if (WL_CAPACITY(pwlTarget) > WL_CAPACITY(gpwlCache))
            {
                pwl = gpwlCache;
                gpwlCache = pwlTarget;
                ExFreePoolWithTag(pwl, USERTAG_WINDOWLIST);
            }
            else
            {
                ExFreePoolWithTag(pwlTarget, USERTAG_WINDOWLIST);
            }
        }
        else
        {
            gpwlCache = pwlTarget;
        }

        break;
    }
}

/* FUNCTIONS *****************************************************************/

/*
 * As best as I can figure, this function is used by EnumWindows,
 * EnumChildWindows, EnumDesktopWindows, & EnumThreadWindows.
 *
 * It's supposed to build a list of HWNDs to return to the caller.
 * We can figure out what kind of list by what parameters are
 * passed to us.
 */
/*
 * @implemented
 */
NTSTATUS
NTAPI
NtUserBuildHwndList(
   HDESK hDesktop,
   HWND hwndParent,
   BOOLEAN bChildren,
   ULONG dwThreadId,
   ULONG cHwnd,
   HWND* phwndList,
   ULONG* pcHwndNeeded)
{
   NTSTATUS Status;
   ULONG dwCount = 0;

   if (pcHwndNeeded == NULL)
       return STATUS_INVALID_PARAMETER;

   UserEnterExclusive();

   if (hwndParent || !dwThreadId)
   {
      PDESKTOP Desktop;
      PWND Parent, Window;

      if(!hwndParent)
      {
         if(hDesktop == NULL && !(Desktop = IntGetActiveDesktop()))
         {
            Status = STATUS_INVALID_HANDLE;
            goto Quit;
         }

         if(hDesktop)
         {
            Status = IntValidateDesktopHandle(hDesktop,
                                              UserMode,
                                              0,
                                              &Desktop);
            if(!NT_SUCCESS(Status))
            {
                Status = STATUS_INVALID_HANDLE;
                goto Quit;
            }
         }
         hwndParent = Desktop->DesktopWindow;
      }
      else
      {
         hDesktop = 0;
      }

      if((Parent = UserGetWindowObject(hwndParent)) &&
         (Window = Parent->spwndChild))
      {
         BOOL bGoDown = TRUE;

         Status = STATUS_SUCCESS;
         while(TRUE)
         {
            if (bGoDown)
            {
               if (dwCount++ < cHwnd && phwndList)
               {
                  _SEH2_TRY
                  {
                     ProbeForWrite(phwndList, sizeof(HWND), 1);
                     *phwndList = UserHMGetHandle(Window);
                     phwndList++;
                  }
                  _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                  {
                     Status = _SEH2_GetExceptionCode();
                  }
                  _SEH2_END
                  if(!NT_SUCCESS(Status))
                  {
                     break;
                  }
               }
               if (Window->spwndChild && bChildren)
               {
                  Window = Window->spwndChild;
                  continue;
               }
               bGoDown = FALSE;
            }
            if (Window->spwndNext)
            {
               Window = Window->spwndNext;
               bGoDown = TRUE;
               continue;
            }
            Window = Window->spwndParent;
            if (Window == Parent)
            {
               break;
            }
         }
      }

      if(hDesktop)
      {
         ObDereferenceObject(Desktop);
      }
   }
   else // Build EnumThreadWindows list!
   {
      PETHREAD Thread;
      PTHREADINFO W32Thread;
      PWND Window;
      HWND *List = NULL;

      Status = PsLookupThreadByThreadId(UlongToHandle(dwThreadId), &Thread);
      if (!NT_SUCCESS(Status))
      {
         ERR("Thread Id is not valid!\n");
         Status = STATUS_INVALID_PARAMETER;
         goto Quit;
      }
      if (!(W32Thread = (PTHREADINFO)Thread->Tcb.Win32Thread))
      {
         ObDereferenceObject(Thread);
         TRACE("Tried to enumerate windows of a non gui thread\n");
         Status = STATUS_INVALID_PARAMETER;
         goto Quit;
      }

     // Do not use Thread link list due to co_UserFreeWindow!!!
     // Current = W32Thread->WindowListHead.Flink;
     // Fixes Api:CreateWindowEx tests!!!
      List = IntWinListChildren(UserGetDesktopWindow());
      if (List)
      {
         int i;
         for (i = 0; List[i]; i++)
         {
            Window = ValidateHwndNoErr(List[i]);
            if (Window && Window->head.pti == W32Thread)
            {
               if (dwCount < cHwnd && phwndList)
               {
                  _SEH2_TRY
                  {
                     ProbeForWrite(phwndList, sizeof(HWND), 1);
                     *phwndList = UserHMGetHandle(Window);
                     phwndList++;
                  }
                  _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                  {
                     Status = _SEH2_GetExceptionCode();
                  }
                  _SEH2_END
                  if (!NT_SUCCESS(Status))
                  {
                     ERR("Failure to build window list!\n");
                     break;
                  }
               }
               dwCount++;
            }
         }
         ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
      }

      ObDereferenceObject(Thread);
   }

   *pcHwndNeeded = dwCount;
   Status = STATUS_SUCCESS;

Quit:
   SetLastNtError(Status);
   UserLeave();
   return Status;
}

static void IntSendParentNotify( PWND pWindow, UINT msg )
{
    PWND Parent;

    if ( (pWindow->style & (WS_CHILD | WS_POPUP)) == WS_CHILD &&
         !(pWindow->ExStyle & WS_EX_NOPARENTNOTIFY))
    {
        Parent = pWindow->spwndParent;
        if (VerifyWnd(Parent) && !UserIsDesktopWindow(Parent))
        {
            USER_REFERENCE_ENTRY Ref;
            UserRefObjectCo(Parent, &Ref);
            co_IntSendMessage( UserHMGetHandle(Parent),
                               WM_PARENTNOTIFY,
                               MAKEWPARAM( msg, pWindow->IDMenu),
                               (LPARAM)UserHMGetHandle(pWindow) );
            UserDerefObjectCo(Parent);
        }
    }
}

void FASTCALL
IntFixWindowCoordinates(CREATESTRUCTW* Cs, PWND ParentWindow, DWORD* dwShowMode)
{
#define IS_DEFAULT(x)  ((x) == CW_USEDEFAULT || (x) == (SHORT)0x8000)

   /* default positioning for overlapped windows */
    if(!(Cs->style & (WS_POPUP | WS_CHILD)))
   {
      PMONITOR pMonitor = NULL;
      PRTL_USER_PROCESS_PARAMETERS ProcessParams;
      PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();

      if (ppi && ppi->hMonitor)
         pMonitor = UserGetMonitorObject(ppi->hMonitor);
      if (!pMonitor)
         pMonitor = UserGetPrimaryMonitor();

      /* Check if we don't have a monitor attached yet */
      if (pMonitor == NULL)
      {
          Cs->x = Cs->y = 0;
          Cs->cx = 800;
          Cs->cy = 600;
          return;
      }

      ProcessParams = PsGetCurrentProcess()->Peb->ProcessParameters;

      if (IS_DEFAULT(Cs->x))
      {
          if (!IS_DEFAULT(Cs->y)) *dwShowMode = Cs->y;

          if(ProcessParams->WindowFlags & STARTF_USEPOSITION)
          {
              Cs->x = ProcessParams->StartingX;
              Cs->y = ProcessParams->StartingY;
          }
          else
          {
               Cs->x = pMonitor->cWndStack * (UserGetSystemMetrics(SM_CXSIZE) + UserGetSystemMetrics(SM_CXFRAME));
               Cs->y = pMonitor->cWndStack * (UserGetSystemMetrics(SM_CYSIZE) + UserGetSystemMetrics(SM_CYFRAME));
               if (Cs->x > ((pMonitor->rcWork.right - pMonitor->rcWork.left) / 4) ||
                   Cs->y > ((pMonitor->rcWork.bottom - pMonitor->rcWork.top) / 4))
               {
                  /* reset counter and position */
                  Cs->x = 0;
                  Cs->y = 0;
                  pMonitor->cWndStack = 0;
               }
               pMonitor->cWndStack++;
          }
      }

      if (IS_DEFAULT(Cs->cx))
      {
          if (ProcessParams->WindowFlags & STARTF_USEPOSITION)
          {
              Cs->cx = ProcessParams->CountX;
              Cs->cy = ProcessParams->CountY;
          }
          else
          {
              Cs->cx = (pMonitor->rcWork.right - pMonitor->rcWork.left) * 3 / 4;
              Cs->cy = (pMonitor->rcWork.bottom - pMonitor->rcWork.top) * 3 / 4;
          }
      }
      /* neither x nor cx are default. Check the y values .
       * In the trace we see Outlook and Outlook Express using
       * cy set to CW_USEDEFAULT when opening the address book.
       */
      else if (IS_DEFAULT(Cs->cy))
      {
          TRACE("Strange use of CW_USEDEFAULT in nHeight\n");
          Cs->cy = (pMonitor->rcWork.bottom - pMonitor->rcWork.top) * 3 / 4;
      }
   }
   else
   {
      /* if CW_USEDEFAULT is set for non-overlapped windows, both values are set to zero */
      if(IS_DEFAULT(Cs->x))
      {
         Cs->x = 0;
         Cs->y = 0;
      }
      if(IS_DEFAULT(Cs->cx))
      {
         Cs->cx = 0;
         Cs->cy = 0;
      }
   }

#undef IS_DEFAULT
}

/* Allocates and initializes a window */
PWND FASTCALL IntCreateWindow(CREATESTRUCTW* Cs,
                              PLARGE_STRING WindowName,
                              PCLS Class,
                              PWND ParentWindow,
                              PWND OwnerWindow,
                              PVOID acbiBuffer,
                              PDESKTOP pdeskCreated,
                              DWORD dwVer )
{
   PWND pWnd = NULL;
   HWND hWnd;
   PTHREADINFO pti;
   BOOL MenuChanged;
   BOOL bUnicodeWindow;
   PCALLPROCDATA pcpd;

   pti = pdeskCreated ? gptiDesktopThread : GetW32ThreadInfo();

   if (!(Cs->dwExStyle & WS_EX_LAYOUTRTL))
   {      // Need both here for wine win.c test_CreateWindow.
      //if (Cs->hwndParent && ParentWindow)
      if (ParentWindow) // It breaks more tests..... WIP.
      {
         if ( (Cs->style & (WS_CHILD|WS_POPUP)) == WS_CHILD &&
              ParentWindow->ExStyle & WS_EX_LAYOUTRTL &&
             !(ParentWindow->ExStyle & WS_EX_NOINHERITLAYOUT) )
            Cs->dwExStyle |= WS_EX_LAYOUTRTL;
      }
      else
      { /*
         * Note from MSDN <https://learn.microsoft.com/en-us/previous-versions/aa913269(v=msdn.10)>:
         *
         * Dialog boxes and message boxes do not inherit layout, so you must
         * set the layout explicitly.
         */
         if ( Class->fnid != FNID_DIALOG )
         {
            if (pti->ppi->dwLayout & LAYOUT_RTL)
            {
               Cs->dwExStyle |= WS_EX_LAYOUTRTL;
            }
         }
      }
   }

   /* Automatically add WS_EX_WINDOWEDGE */
   if ((Cs->dwExStyle & WS_EX_DLGMODALFRAME) ||
         ((!(Cs->dwExStyle & WS_EX_STATICEDGE)) &&
         (Cs->style & (WS_DLGFRAME | WS_THICKFRAME))))
      Cs->dwExStyle |= WS_EX_WINDOWEDGE;
   else
      Cs->dwExStyle &= ~WS_EX_WINDOWEDGE;

   /* Is it a unicode window? */
   bUnicodeWindow =!(Cs->dwExStyle & WS_EX_SETANSICREATOR);
   Cs->dwExStyle &= ~WS_EX_SETANSICREATOR;

   /* Allocate the new window */
   pWnd = (PWND) UserCreateObject( gHandleTable,
                                   pdeskCreated ? pdeskCreated : pti->rpdesk,
                                   pti,
                                  (PHANDLE)&hWnd,
                                   TYPE_WINDOW,
                                   sizeof(WND) + Class->cbwndExtra);

   if (!pWnd)
   {
      goto AllocError;
   }

   TRACE("Created window object with handle %p\n", hWnd);

   if (pdeskCreated && pdeskCreated->DesktopWindow == NULL )
   {  /* HACK: Helper for win32csr/desktopbg.c */
      /* If there is no desktop window yet, we must be creating it */
      TRACE("CreateWindow setting desktop.\n");
      pdeskCreated->DesktopWindow = hWnd;
      pdeskCreated->pDeskInfo->spwnd = pWnd;
   }

   /*
    * Fill out the structure describing it.
    */
   /* Remember, pWnd->head is setup in object.c ... */
   WndSetParent(pWnd, ParentWindow);
   WndSetOwner(pWnd, OwnerWindow);
   pWnd->fnid = 0;
   WndSetLastActive(pWnd, pWnd);
   // Ramp up compatible version sets.
   if ( dwVer >= WINVER_WIN31 )
   {
       pWnd->state2 |= WNDS2_WIN31COMPAT;
       if ( dwVer >= WINVER_WINNT4 )
       {
           pWnd->state2 |= WNDS2_WIN40COMPAT;
           if ( dwVer >= WINVER_WIN2K )
           {
               pWnd->state2 |= WNDS2_WIN50COMPAT;
           }
       }
   }
   pWnd->pcls = Class;
   pWnd->hModule = Cs->hInstance;
   pWnd->style = Cs->style & ~WS_VISIBLE;
   pWnd->ExStyle = Cs->dwExStyle;
   pWnd->cbwndExtra = pWnd->pcls->cbwndExtra;
   pWnd->pActCtx = acbiBuffer;

   if (pti->spDefaultImc && Class->atomClassName != gpsi->atomSysClass[ICLS_BUTTON])
      pWnd->hImc = UserHMGetHandle(pti->spDefaultImc);

   pWnd->InternalPos.MaxPos.x  = pWnd->InternalPos.MaxPos.y  = -1;
   pWnd->InternalPos.IconPos.x = pWnd->InternalPos.IconPos.y = -1;

   if (pWnd->spwndParent != NULL && Cs->hwndParent != 0)
   {
       pWnd->HideFocus = pWnd->spwndParent->HideFocus;
       pWnd->HideAccel = pWnd->spwndParent->HideAccel;
   }

   InitializeListHead(&pWnd->ThreadListEntry);
   pWnd->head.pti->cWindows++;

   if (Class->spicn && !Class->spicnSm)
   {
       HICON IconSmHandle = NULL;
       if((Class->spicn->CURSORF_flags & (CURSORF_LRSHARED | CURSORF_FROMRESOURCE))
               == (CURSORF_LRSHARED | CURSORF_FROMRESOURCE))
       {
           IconSmHandle = co_IntCopyImage(
               UserHMGetHandle(Class->spicn),
               IMAGE_ICON,
               UserGetSystemMetrics( SM_CXSMICON ),
               UserGetSystemMetrics( SM_CYSMICON ),
               LR_COPYFROMRESOURCE);
       }
       if (!IconSmHandle)
       {
           /* Retry without copying from resource */
           IconSmHandle = co_IntCopyImage(
               UserHMGetHandle(Class->spicn),
               IMAGE_ICON,
               UserGetSystemMetrics( SM_CXSMICON ),
               UserGetSystemMetrics( SM_CYSMICON ),
               0);
       }

       if (IconSmHandle)
       {
           Class->spicnSm = UserGetCurIconObject(IconSmHandle);
           Class->CSF_flags |= CSF_CACHEDSMICON;
       }
   }

   if (pWnd->pcls->CSF_flags & CSF_SERVERSIDEPROC)
      pWnd->state |= WNDS_SERVERSIDEWINDOWPROC;

 /* BugBoy Comments: Comment below say that System classes are always created
    as UNICODE. In windows, creating a window with the ANSI version of CreateWindow
    sets the window to ansi as verified by testing with IsUnicodeWindow API.

    No where can I see in code or through testing does the window change back
    to ANSI after being created as UNICODE in ROS. I didnt do more testing to
    see what problems this would cause. */

   // Set WndProc from Class.
   if (IsCallProcHandle(pWnd->pcls->lpfnWndProc))
   {
      pcpd = UserGetObject(gHandleTable, pWnd->pcls->lpfnWndProc, TYPE_CALLPROC);
      if (pcpd)
         pWnd->lpfnWndProc = pcpd->pfnClientPrevious;
   }
   else
   {
      pWnd->lpfnWndProc = pWnd->pcls->lpfnWndProc;
   }

   // GetWindowProc, test for non server side default classes and set WndProc.
    if ( pWnd->pcls->fnid <= FNID_GHOST && pWnd->pcls->fnid >= FNID_BUTTON )
    {
      if (bUnicodeWindow)
      {
         if (GETPFNCLIENTA(pWnd->pcls->fnid) == pWnd->lpfnWndProc)
            pWnd->lpfnWndProc = GETPFNCLIENTW(pWnd->pcls->fnid);
      }
      else
      {
         if (GETPFNCLIENTW(pWnd->pcls->fnid) == pWnd->lpfnWndProc)
            pWnd->lpfnWndProc = GETPFNCLIENTA(pWnd->pcls->fnid);
      }
    }


   // If not an Unicode caller, set Ansi creator bit.
   if (!bUnicodeWindow) pWnd->state |= WNDS_ANSICREATOR;

   // Clone Class Ansi/Unicode proc type.
   if (pWnd->pcls->CSF_flags & CSF_ANSIPROC)
   {
      pWnd->state |= WNDS_ANSIWINDOWPROC;
      pWnd->Unicode = FALSE;
   }
   else
   { /*
      * It seems there can be both an Ansi creator and Unicode Class Window
      * WndProc, unless the following overriding conditions occur:
      */
      if ( !bUnicodeWindow &&
          ( Class->atomClassName == gpsi->atomSysClass[ICLS_BUTTON]    ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_COMBOBOX]  ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_COMBOLBOX] ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_DIALOG]    ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_EDIT]      ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_IME]       ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_LISTBOX]   ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_MDICLIENT] ||
            Class->atomClassName == gpsi->atomSysClass[ICLS_STATIC] ) )
      { // Override Class and set the window Ansi WndProc.
         pWnd->state |= WNDS_ANSIWINDOWPROC;
         pWnd->Unicode = FALSE;
      }
      else
      { // Set the window Unicode WndProc.
         pWnd->state &= ~WNDS_ANSIWINDOWPROC;
         pWnd->Unicode = TRUE;
      }
   }

   /* BugBoy Comments: if the window being created is a edit control, ATOM 0xCxxx,
      then my testing shows that windows (2k and XP) creates a CallProc for it immediately
      Dont understand why it does this. */
   if (Class->atomClassName == gpsi->atomSysClass[ICLS_EDIT])
   {
      PCALLPROCDATA CallProc;
      CallProc = CreateCallProc(pWnd->head.rpdesk, pWnd->lpfnWndProc, pWnd->Unicode , pWnd->head.pti->ppi);

      if (!CallProc)
      {
         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
         ERR("Warning: Unable to create CallProc for edit control. Control may not operate correctly! hwnd %p\n", hWnd);
      }
      else
      {
         UserAddCallProcToClass(pWnd->pcls, CallProc);
      }
   }

   InitializeListHead(&pWnd->PropListHead);
   pWnd->PropListItems = 0;

   if ( WindowName->Buffer != NULL && WindowName->Length > 0 )
   {
      pWnd->strName.Buffer = DesktopHeapAlloc(pWnd->head.rpdesk,
                                             WindowName->Length + sizeof(UNICODE_NULL));
      if (pWnd->strName.Buffer == NULL)
      {
          goto AllocError;
      }

      RtlCopyMemory(pWnd->strName.Buffer, WindowName->Buffer, WindowName->Length);
      pWnd->strName.Buffer[WindowName->Length / sizeof(WCHAR)] = L'\0';
      pWnd->strName.Length = WindowName->Length;
      pWnd->strName.MaximumLength = WindowName->Length + sizeof(UNICODE_NULL);
   }

   /* Correct the window style. */
   if ((pWnd->style & (WS_CHILD | WS_POPUP)) != WS_CHILD)
   {
      pWnd->style |= WS_CLIPSIBLINGS;
      if (!(pWnd->style & WS_POPUP))
      {
         pWnd->style |= WS_CAPTION;
      }
   }

   /* WS_EX_WINDOWEDGE depends on some other styles */
   if (pWnd->ExStyle & WS_EX_DLGMODALFRAME)
       pWnd->ExStyle |= WS_EX_WINDOWEDGE;
   else if (pWnd->style & (WS_DLGFRAME | WS_THICKFRAME))
   {
       if (!((pWnd->ExStyle & WS_EX_STATICEDGE) &&
            (pWnd->style & (WS_CHILD | WS_POPUP))))
           pWnd->ExStyle |= WS_EX_WINDOWEDGE;
   }
    else
        pWnd->ExStyle &= ~WS_EX_WINDOWEDGE;

   if (!(pWnd->style & (WS_CHILD | WS_POPUP)))
      pWnd->state |= WNDS_SENDSIZEMOVEMSGS;

   /* Set the window menu */
   if ((Cs->style & (WS_CHILD | WS_POPUP)) != WS_CHILD)
   {
      if (Cs->hMenu)
      {
         IntSetMenu(pWnd, Cs->hMenu, &MenuChanged);
      }
      else if (pWnd->pcls->lpszMenuName) // Take it from the parent.
      {
          UNICODE_STRING MenuName;
          HMENU hMenu;

          if (IS_INTRESOURCE(pWnd->pcls->lpszMenuName))
          {
             MenuName.Length = 0;
             MenuName.MaximumLength = 0;
             MenuName.Buffer = pWnd->pcls->lpszMenuName;
          }
          else
          {
             RtlInitUnicodeString( &MenuName, pWnd->pcls->lpszMenuName);
          }
          hMenu = co_IntCallLoadMenu( pWnd->pcls->hModule, &MenuName);
          if (hMenu) IntSetMenu(pWnd, hMenu, &MenuChanged);
      }
   }
   else // Not a child
      pWnd->IDMenu = (UINT_PTR)Cs->hMenu;


   if ( ParentWindow &&
        ParentWindow != ParentWindow->head.rpdesk->spwndMessage &&
        ParentWindow != ParentWindow->head.rpdesk->pDeskInfo->spwnd )
   {
       PWND Owner = IntGetNonChildAncestor(ParentWindow);

       if (!IntValidateOwnerDepth(pWnd, Owner))
       {
          EngSetLastError(ERROR_INVALID_PARAMETER);
          goto Error;
       }
       if ( pWnd->spwndOwner &&
            pWnd->spwndOwner->ExStyle & WS_EX_TOPMOST )
       {
          pWnd->ExStyle |= WS_EX_TOPMOST;
       }
       if ( pWnd->spwndOwner &&
            Class->atomClassName != gpsi->atomSysClass[ICLS_IME] &&
            pti != pWnd->spwndOwner->head.pti)
       {
          //ERR("CreateWindow Owner in.\n");
          UserAttachThreadInput(pti, pWnd->spwndOwner->head.pti, TRUE);
       }
   }

   /* Insert the window into the thread's window list. */
   InsertTailList (&pti->WindowListHead, &pWnd->ThreadListEntry);

   /* Handle "CS_CLASSDC", it is tested first. */
   if ( (pWnd->pcls->style & CS_CLASSDC) && !(pWnd->pcls->pdce) )
   {  /* One DCE per class to have CLASS. */
      pWnd->pcls->pdce = DceAllocDCE( pWnd, DCE_CLASS_DC );
   }
   else if ( pWnd->pcls->style & CS_OWNDC)
   {  /* Allocate a DCE for this window. */
      DceAllocDCE(pWnd, DCE_WINDOW_DC);
   }

   return pWnd;

AllocError:
   ERR("IntCreateWindow Allocation Error.\n");
   SetLastNtError(STATUS_INSUFFICIENT_RESOURCES);
Error:
   if(pWnd)
      UserDereferenceObject(pWnd);
   return NULL;
}