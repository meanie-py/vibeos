/* window.split3.c — #include fragment of window.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original window.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


/*
 * @implemented
 */
PWND FASTCALL
co_UserCreateWindowEx(CREATESTRUCTW* Cs,
                     PUNICODE_STRING ClassName,
                     PLARGE_STRING WindowName,
                     PVOID acbiBuffer,
                     DWORD dwVer )
{
   ULONG style;
   PWND Window = NULL, ParentWindow = NULL, OwnerWindow;
   HWND hWnd, hWndParent, hWndOwner, hwndInsertAfter;
   PWINSTATION_OBJECT WinSta;
   PCLS Class = NULL;
   SIZE Size;
   POINT MaxSize, MaxPos, MinTrack, MaxTrack;
   CBT_CREATEWNDW * pCbtCreate;
   LRESULT Result;
   USER_REFERENCE_ENTRY ParentRef, Ref;
   PTHREADINFO pti;
   DWORD dwShowMode = SW_SHOW;
   CREATESTRUCTW *pCsw = NULL;
   PVOID pszClass = NULL, pszName = NULL;
   PWND ret = NULL;

   /* Get the current window station and reference it */
   pti = GetW32ThreadInfo();
   if (pti == NULL || pti->rpdesk == NULL)
   {
      ERR("Thread is not attached to a desktop! Cannot create window (%wZ)\n", ClassName);
      return NULL; // There is nothing to cleanup.
   }
   WinSta = pti->rpdesk->rpwinstaParent;
   ObReferenceObjectByPointer(WinSta, KernelMode, ExWindowStationObjectType, 0);

   pCsw = NULL;
   pCbtCreate = NULL;

   /* Get the class and reference it */
   Class = IntGetAndReferenceClass(ClassName, Cs->hInstance, FALSE);
   if(!Class)
   {
       EngSetLastError(ERROR_CANNOT_FIND_WND_CLASS);
       ERR("Failed to find class %wZ\n", ClassName);
       goto cleanup;
   }

   /* Now find the parent and the owner window */
   hWndParent = UserHMGetHandle(pti->rpdesk->pDeskInfo->spwnd);
   hWndOwner = NULL;

    if (Cs->hwndParent == HWND_MESSAGE)
    {
        Cs->hwndParent = hWndParent = UserHMGetHandle(pti->rpdesk->spwndMessage);
    }
    else if (Cs->hwndParent)
    {
        if ((Cs->style & (WS_CHILD|WS_POPUP)) != WS_CHILD)
            hWndOwner = Cs->hwndParent;
        else
            hWndParent = Cs->hwndParent;
    }
    else if ((Cs->style & (WS_CHILD|WS_POPUP)) == WS_CHILD)
    {
         ERR("Cannot create a child window (%wZ) without a parent\n", ClassName);
         EngSetLastError(ERROR_TLW_WITH_WSCHILD);
         goto cleanup;  /* WS_CHILD needs a parent, but WS_POPUP doesn't */
    }
    else if (Cs->lpszClass != (LPCWSTR)MAKEINTATOM(gpsi->atomSysClass[ICLS_DESKTOP]) &&
             (IS_INTRESOURCE(Cs->lpszClass) ||
              Cs->lpszClass != (LPCWSTR)MAKEINTATOM(gpsi->atomSysClass[ICLS_HWNDMESSAGE]) ||
              _wcsicmp(Cs->lpszClass, L"Message") != 0))
    {
        if (pti->ppi->dwLayout & LAYOUT_RTL)
        {
            Cs->dwExStyle |= WS_EX_LAYOUTRTL;
        }
    }

    ParentWindow = hWndParent ? UserGetWindowObject(hWndParent): NULL;
    OwnerWindow = hWndOwner ? UserGetWindowObject(hWndOwner): NULL;

    if (hWndParent && !ParentWindow)
    {
        ERR("Got invalid parent window handle for %wZ\n", ClassName);
        goto cleanup;
    }
    else if (hWndOwner && !OwnerWindow)
    {
        ERR("Got invalid owner window handle for %wZ\n", ClassName);
        ParentWindow = NULL;
        goto cleanup;
    }

    if(OwnerWindow)
    {
       if (IntIsDesktopWindow(OwnerWindow)) OwnerWindow = NULL;
       else if (ParentWindow && !IntIsDesktopWindow(ParentWindow))
       {
          ERR("an owned window must be created as top-level\n");
          EngSetLastError( STATUS_ACCESS_DENIED );
          goto cleanup;
       }
       else /* owner must be a top-level window */
       {
          while ((OwnerWindow->style & (WS_POPUP|WS_CHILD)) == WS_CHILD && !IntIsDesktopWindow(OwnerWindow->spwndParent))
                 OwnerWindow = OwnerWindow->spwndParent;
       }
    }

   /* Fix the position and the size of the window */
   if (ParentWindow)
   {
       UserRefObjectCo(ParentWindow, &ParentRef);
       IntFixWindowCoordinates(Cs, ParentWindow, &dwShowMode);
   }

   /* Allocate and initialize the new window */
   Window = IntCreateWindow(Cs,
                            WindowName,
                            Class,
                            ParentWindow,
                            OwnerWindow,
                            acbiBuffer,
                            NULL,
                            dwVer );
   if(!Window)
   {
       ERR("IntCreateWindow(%wZ) failed\n", ClassName);
       goto cleanup;
   }

   hWnd = UserHMGetHandle(Window);
   hwndInsertAfter = HWND_TOP;

   UserRefObjectCo(Window, &Ref);
   UserDereferenceObject(Window);
   ObDereferenceObject(WinSta);

   /* NCCREATE, WM_NCCALCSIZE and Hooks need the original values */
   Cs->lpszName = (LPCWSTR) WindowName;
   Cs->lpszClass = (LPCWSTR) ClassName;

   //// Check for a hook to eliminate overhead. ////
   if ( ISITHOOKED(WH_CBT) ||  (pti->rpdesk->pDeskInfo->fsHooks & HOOKID_TO_FLAG(WH_CBT)) )
   {
      // Allocate the calling structures Justin Case this goes Global.
      pCsw = ExAllocatePoolWithTag(NonPagedPool, sizeof(CREATESTRUCTW), TAG_HOOK);
      pCbtCreate = ExAllocatePoolWithTag(NonPagedPool, sizeof(CBT_CREATEWNDW), TAG_HOOK);
      if (!pCsw || !pCbtCreate)
      {
      	 ERR("UserHeapAlloc() failed!\n");
      	 goto cleanup;
      }

      if (!IntMsgCreateStructW( Window, pCsw, Cs, &pszClass, &pszName ) )
      {
          ERR("IntMsgCreateStructW() failed!\n");
          goto cleanup;
      }

      pCbtCreate->lpcs = pCsw;
      pCbtCreate->hwndInsertAfter = hwndInsertAfter;

      //// Call the WH_CBT hook ////
      Result = co_HOOK_CallHooks(WH_CBT, HCBT_CREATEWND, (WPARAM) hWnd, (LPARAM) pCbtCreate);
      if (Result != 0)
      {
         ERR("WH_CBT HCBT_CREATEWND hook failed! 0x%x\n", Result);
         goto cleanup;
      }
      // Write back changes.
      Cs->cx = pCsw->cx;
      Cs->cy = pCsw->cy;
      Cs->x = pCsw->x;
      Cs->y = pCsw->y;
      hwndInsertAfter = pCbtCreate->hwndInsertAfter;
   }

   if ((Cs->style & (WS_CHILD|WS_POPUP)) == WS_CHILD)
   {
      if (ParentWindow != co_GetDesktopWindow(Window))
      {
         Cs->x += ParentWindow->rcClient.left;
         Cs->y += ParentWindow->rcClient.top;
      }
   }

   /* Send the WM_GETMINMAXINFO message */
   Size.cx = Cs->cx;
   Size.cy = Cs->cy;

   if ((Cs->style & WS_THICKFRAME) || !(Cs->style & (WS_POPUP | WS_CHILD)))
   {
      co_WinPosGetMinMaxInfo(Window, &MaxSize, &MaxPos, &MinTrack, &MaxTrack);
      if (Size.cx > MaxTrack.x) Size.cx = MaxTrack.x;
      if (Size.cy > MaxTrack.y) Size.cy = MaxTrack.y;
      if (Size.cx < MinTrack.x) Size.cx = MinTrack.x;
      if (Size.cy < MinTrack.y) Size.cy = MinTrack.y;
   }

   Window->rcWindow.left = Cs->x;
   Window->rcWindow.top = Cs->y;
   Window->rcWindow.right = Cs->x + Size.cx;
   Window->rcWindow.bottom = Cs->y + Size.cy;
 /*
   if (0 != (Window->style & WS_CHILD) && ParentWindow)
   {
      ERR("co_UserCreateWindowEx(): Offset rcWindow\n");
      RECTL_vOffsetRect(&Window->rcWindow,
                        ParentWindow->rcClient.left,
                        ParentWindow->rcClient.top);
   }
 */
   /* correct child window coordinates if mirroring on parent is enabled */
   if (ParentWindow != NULL)
   {
      if ( ((Cs->style & WS_CHILD) == WS_CHILD) &&
          ((ParentWindow->ExStyle & WS_EX_LAYOUTRTL) ==  WS_EX_LAYOUTRTL))
      {
          Window->rcWindow.right = ParentWindow->rcClient.right - (Window->rcWindow.left - ParentWindow->rcClient.left);
          Window->rcWindow.left = Window->rcWindow.right - Size.cx;
      }
   }

   Window->rcClient = Window->rcWindow;

   if (Window->spwndNext || Window->spwndPrev)
   {
      ERR("Window 0x%p has been linked too early!\n", Window);
   }

   if (!(Window->state2 & WNDS2_WIN31COMPAT))
   {
      if (Class->style & CS_PARENTDC && !(ParentWindow->style & WS_CLIPCHILDREN))
         Window->style &= ~(WS_CLIPSIBLINGS | WS_CLIPCHILDREN);
   }

   if ((Window->style & (WS_CHILD | WS_POPUP)) == WS_CHILD)
   {
      if ( !IntIsTopLevelWindow(Window) )
      {
         if (pti != ParentWindow->head.pti)
         {
            //ERR("CreateWindow Parent in.\n");
            UserAttachThreadInput(pti, ParentWindow->head.pti, TRUE);
         }
      }
   }

   /* Send the NCCREATE message */
   Result = co_IntSendMessage(UserHMGetHandle(Window), WM_NCCREATE, 0, (LPARAM) Cs);
   if (!Result)
   {
      ERR("co_UserCreateWindowEx(%wZ): NCCREATE message failed\n", ClassName);
      goto cleanup;
   }

   /* Link the window */
   if (ParentWindow != NULL)
   {
      /* Link the window into the siblings list */
      if ((Cs->style & (WS_CHILD | WS_MAXIMIZE)) == WS_CHILD)
          IntLinkHwnd(Window, HWND_BOTTOM);
      else
          IntLinkHwnd(Window, hwndInsertAfter);
   }

   /* Create the IME window for pWnd */
   if (IS_IMM_MODE() && !pti->spwndDefaultIme && IntWantImeWindow(Window))
   {
      PWND pwndDefaultIme = co_IntCreateDefaultImeWindow(Window, Window->hModule);
      UserAssignmentLock((PVOID*)&pti->spwndDefaultIme, pwndDefaultIme);

      if (pwndDefaultIme)
      {
         HWND hImeWnd;
         USER_REFERENCE_ENTRY Ref;
         UserRefObjectCo(pwndDefaultIme, &Ref);

         hImeWnd = UserHMGetHandle(pwndDefaultIme);

         co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, IMS_LOADTHREADLAYOUT, 0);

         if (pti->pClientInfo->CI_flags & CI_IMMACTIVATE)
         {
            HKL hKL = pti->KeyboardLayout->hkl;
            co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, IMS_ACTIVATELAYOUT, (LPARAM)hKL);
            pti->pClientInfo->CI_flags &= ~CI_IMMACTIVATE;
         }

         UserDerefObjectCo(pwndDefaultIme);
      }
   }

   /* Send the WM_NCCALCSIZE message */
   {
  // RECT rc;
   MaxPos.x = Window->rcWindow.left;
   MaxPos.y = Window->rcWindow.top;

   Result = co_WinPosGetNonClientSize(Window, &Window->rcWindow, &Window->rcClient);
   //rc = Window->rcWindow;
   //Result = co_IntSendMessageNoWait(UserHMGetHandle(Window), WM_NCCALCSIZE, FALSE, (LPARAM)&rc);
   //Window->rcClient = rc;

   RECTL_vOffsetRect(&Window->rcWindow, MaxPos.x - Window->rcWindow.left,
                                     MaxPos.y - Window->rcWindow.top);
   }

   /* Send the WM_CREATE message. */
   Result = co_IntSendMessage(UserHMGetHandle(Window), WM_CREATE, 0, (LPARAM) Cs);
   if (Result == (LRESULT)-1)
   {
      ERR("co_UserCreateWindowEx(%wZ): WM_CREATE message failed\n", ClassName);
      goto cleanup;
   }

   /* Send the EVENT_OBJECT_CREATE event */
   IntNotifyWinEvent(EVENT_OBJECT_CREATE, Window, OBJID_WINDOW, CHILDID_SELF, 0);

   /* By setting the flag below it can be examined to determine if the window
      was created successfully and a valid pwnd was passed back to caller since
      from here the function has to succeed. */
   Window->state2 |= WNDS2_WMCREATEMSGPROCESSED;

   /* Send the WM_SIZE and WM_MOVE messages. */
   if (!(Window->state & WNDS_SENDSIZEMOVEMSGS))
   {
        co_WinPosSendSizeMove(Window);
   }

   /* Show or maybe minimize or maximize the window. */

   style = IntSetStyle( Window, 0, WS_MAXIMIZE | WS_MINIMIZE );
   if (style & (WS_MINIMIZE | WS_MAXIMIZE))
   {
      RECTL NewPos;
      UINT SwFlag = (style & WS_MINIMIZE) ? SW_MINIMIZE : SW_MAXIMIZE;

      SwFlag = co_WinPosMinMaximize(Window, SwFlag, &NewPos);
      SwFlag |= SWP_NOZORDER|SWP_FRAMECHANGED; /* Frame always gets changed */
      if (!(style & WS_VISIBLE) || (style & WS_CHILD) || UserGetActiveWindow() ||
          (Window->ExStyle & WS_EX_NOACTIVATE))
      {
         SwFlag |= SWP_NOACTIVATE;
      }
      co_WinPosSetWindowPos(Window, 0, NewPos.left, NewPos.top,
                            NewPos.right, NewPos.bottom, SwFlag);
   }

   /* Send the WM_PARENTNOTIFY message */
   IntSendParentNotify(Window, WM_CREATE);

   /* Notify the shell that a new window was created */
   if (Window->spwndOwner == NULL ||
       !(Window->spwndOwner->style & WS_VISIBLE) ||
       (Window->spwndOwner->ExStyle & WS_EX_TOOLWINDOW))
   {
      if (UserIsDesktopWindow(Window->spwndParent) &&
          (Window->style & WS_VISIBLE) &&
          (!(Window->ExStyle & WS_EX_TOOLWINDOW) ||
           (Window->ExStyle & WS_EX_APPWINDOW)))
      {
         co_IntShellHookNotify(HSHELL_WINDOWCREATED, (WPARAM)hWnd, 0);
      }
   }

   /* Initialize and show the window's scrollbars */
   if (Window->style & WS_VSCROLL)
   {
      co_UserShowScrollBar(Window, SB_VERT, FALSE, TRUE);
   }
   if (Window->style & WS_HSCROLL)
   {
      co_UserShowScrollBar(Window, SB_HORZ, TRUE, FALSE);
   }

   /* Show the new window */
   if (Cs->style & WS_VISIBLE)
   {
      if (Window->style & WS_MAXIMIZE)
         dwShowMode = SW_SHOW;
      else if (Window->style & WS_MINIMIZE)
         dwShowMode = SW_SHOWMINIMIZED;

      co_WinPosShowWindow(Window, dwShowMode);

      if (Window->ExStyle & WS_EX_MDICHILD)
      {
          ASSERT(ParentWindow);
          if(!ParentWindow)
              goto cleanup;
        co_IntSendMessage(UserHMGetHandle(ParentWindow), WM_MDIREFRESHMENU, 0, 0);
        /* ShowWindow won't activate child windows */
        co_WinPosSetWindowPos(Window, HWND_TOP, 0, 0, 0, 0, SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE);
      }
   }

   if (Class->atomClassName == gaGuiConsoleWndClass)
   {
       /* Count only console windows manually */
       co_IntUserManualGuiCheck(TRUE);
   }

   /* Set the hotkey */
   if (!(Window->style & (WS_POPUP | WS_CHILD)) || (Window->ExStyle & WS_EX_APPWINDOW))
   {
       if (pti->ppi->dwHotkey)
       {
          co_IntSendMessage(UserHMGetHandle(Window), WM_SETHOTKEY, pti->ppi->dwHotkey, 0);
          pti->ppi->dwHotkey = 0; /* Only the first suitable window gets the hotkey */
       }
   }

   TRACE("co_UserCreateWindowEx(%wZ): Created window %p\n", ClassName, hWnd);
   ret = Window;

cleanup:
   if (!ret)
   {
       TRACE("co_UserCreateWindowEx(): Error Created window!\n");
       /* If the window was created, the class will be dereferenced by co_UserDestroyWindow */
       if (Window)
            co_UserDestroyWindow(Window);
       else if (Class)
           IntDereferenceClass(Class, pti->pDeskInfo, pti->ppi);
   }

   if (pCsw) ExFreePoolWithTag(pCsw, TAG_HOOK);
   if (pCbtCreate) ExFreePoolWithTag(pCbtCreate, TAG_HOOK);
   if (pszName) UserHeapFree(pszName);
   if (pszClass) UserHeapFree(pszClass);

   if (Window)
   {
      UserDerefObjectCo(Window);
   }
   if (ParentWindow) UserDerefObjectCo(ParentWindow);

   // See CORE-13717, not setting error on success.
   if (ret)
      EngSetLastError(ERROR_SUCCESS);

   return ret;
}

NTSTATUS
NTAPI
ProbeAndCaptureLargeString(
    OUT PLARGE_STRING plstrSafe,
    IN PLARGE_STRING plstrUnsafe)
{
    LARGE_STRING lstrTemp;
    PVOID pvBuffer = NULL;

    {
        NTSTATUS __SehStatus = SafeProbeForRead(plstrUnsafe, sizeof(LARGE_STRING), sizeof(ULONG));
        if (!NT_SUCCESS(__SehStatus))
        {
            /* Fail */
            return __SehStatus;
        }
    }
    /* Probe and copy the string */

    lstrTemp = *plstrUnsafe;

    if (lstrTemp.Length != 0)
    {
        /* Allocate a buffer from paged pool */
        pvBuffer = ExAllocatePoolWithTag(PagedPool, lstrTemp.Length, TAG_STRING);
        if (!pvBuffer)
        {
            return STATUS_NO_MEMORY;
        }

        {
            NTSTATUS __SehStatus = SafeProbeForRead(lstrTemp.Buffer, lstrTemp.Length, sizeof(WCHAR));
            if (!NT_SUCCESS(__SehStatus))
            {
                /* Cleanup and fail */
                ExFreePoolWithTag(pvBuffer, TAG_STRING);
                return __SehStatus;
            }
        }
        /* Probe and copy the buffer */

        RtlCopyMemory(pvBuffer, lstrTemp.Buffer, lstrTemp.Length);
    }

    /* Set the output string */
    plstrSafe->Buffer = pvBuffer;
    plstrSafe->Length = lstrTemp.Length;
    plstrSafe->MaximumLength = lstrTemp.Length;

    return STATUS_SUCCESS;
}

/**
 * \todo Allow passing plstrClassName as ANSI.
 */
HWND
NTAPI
NtUserCreateWindowEx(
    DWORD dwExStyle,
    PLARGE_STRING plstrClassName,
    PLARGE_STRING plstrClsVersion,
    PLARGE_STRING plstrWindowName,
    DWORD dwStyle,
    int x,
    int y,
    int nWidth,
    int nHeight,
    HWND hWndParent,
    HMENU hMenu,
    HINSTANCE hInstance,
    LPVOID lpParam,
    DWORD dwFlags,
    PVOID acbiBuffer)
{
    NTSTATUS Status;
    LARGE_STRING lstrWindowName;
    LARGE_STRING lstrClassName;
    LARGE_STRING lstrClsVersion;
    UNICODE_STRING ustrClassName;
    UNICODE_STRING ustrClsVersion;
    CREATESTRUCTW Cs;
    HWND hwnd = NULL;
    PWND pwnd;

    lstrWindowName.Buffer = NULL;
    lstrClassName.Buffer = NULL;
    lstrClsVersion.Buffer = NULL;

    if ( (dwStyle & (WS_POPUP|WS_CHILD)) != WS_CHILD)
    {
        /* check hMenu is valid handle */
        if (hMenu && !UserGetMenuObject(hMenu))
        {
            ERR("NtUserCreateWindowEx: Got an invalid menu handle!\n");
            EngSetLastError(ERROR_INVALID_MENU_HANDLE);
            return NULL;
        }
    }

    /* Copy the window name to kernel mode */
    Status = ProbeAndCaptureLargeString(&lstrWindowName, plstrWindowName);
    if (!NT_SUCCESS(Status))
    {
        ERR("NtUserCreateWindowEx: failed to capture plstrWindowName\n");
        SetLastNtError(Status);
        return NULL;
    }

    plstrWindowName = &lstrWindowName;

    /* Check if the class is an atom */
    if (IS_ATOM(plstrClassName))
    {
        /* It is, pass the atom in the UNICODE_STRING */
        ustrClassName.Buffer = (PVOID)plstrClassName;
        ustrClassName.Length = 0;
        ustrClassName.MaximumLength = 0;
    }
    else
    {
        /* It's not, capture the class name */
        Status = ProbeAndCaptureLargeString(&lstrClassName, plstrClassName);
        if (!NT_SUCCESS(Status))
        {
            ERR("NtUserCreateWindowEx: failed to capture plstrClassName\n");
            /* Set last error, cleanup and return */
            SetLastNtError(Status);
            goto cleanup;
        }

        /* We pass it on as a UNICODE_STRING */
        ustrClassName.Buffer = lstrClassName.Buffer;
        ustrClassName.Length = (USHORT)min(lstrClassName.Length, MAXUSHORT); // FIXME: LARGE_STRING truncated
        ustrClassName.MaximumLength = (USHORT)min(lstrClassName.MaximumLength, MAXUSHORT);
    }

    /* Check if the class version is an atom */
    if (IS_ATOM(plstrClsVersion))
    {
        /* It is, pass the atom in the UNICODE_STRING */
        ustrClsVersion.Buffer = (PVOID)plstrClsVersion;
        ustrClsVersion.Length = 0;
        ustrClsVersion.MaximumLength = 0;
    }
    else
    {
        /* It's not, capture the class name */
        Status = ProbeAndCaptureLargeString(&lstrClsVersion, plstrClsVersion);
        if (!NT_SUCCESS(Status))
        {
            ERR("NtUserCreateWindowEx: failed to capture plstrClsVersion\n");
            /* Set last error, cleanup and return */
            SetLastNtError(Status);
            goto cleanup;
        }

        /* We pass it on as a UNICODE_STRING */
        ustrClsVersion.Buffer = lstrClsVersion.Buffer;
        ustrClsVersion.Length = (USHORT)min(lstrClsVersion.Length, MAXUSHORT); // FIXME: LARGE_STRING truncated
        ustrClsVersion.MaximumLength = (USHORT)min(lstrClsVersion.MaximumLength, MAXUSHORT);
    }

    /* Fill the CREATESTRUCTW */
    /* we will keep here the original parameters */
    Cs.style = dwStyle;
    Cs.lpCreateParams = lpParam;
    Cs.hInstance = hInstance;
    Cs.hMenu = hMenu;
    Cs.hwndParent = hWndParent;
    Cs.cx = nWidth;
    Cs.cy = nHeight;
    Cs.x = x;
    Cs.y = y;
    Cs.lpszName = (LPCWSTR) plstrWindowName->Buffer;
    Cs.lpszClass = ustrClassName.Buffer;
    Cs.dwExStyle = dwExStyle;

    UserEnterExclusive();

    /* Call the internal function */
    pwnd = co_UserCreateWindowEx(&Cs, &ustrClsVersion, plstrWindowName, acbiBuffer, dwFlags);

    if(!pwnd)
    {
        ERR("co_UserCreateWindowEx failed!\n");
    }
    hwnd = pwnd ? UserHMGetHandle(pwnd) : NULL;

    UserLeave();

cleanup:
    if (lstrWindowName.Buffer)
    {
        ExFreePoolWithTag(lstrWindowName.Buffer, TAG_STRING);
    }
    if (lstrClassName.Buffer)
    {
        ExFreePoolWithTag(lstrClassName.Buffer, TAG_STRING);
    }
    if (lstrClsVersion.Buffer)
    {
        ExFreePoolWithTag(lstrClsVersion.Buffer, TAG_STRING);
    }

   return hwnd;
}

// Win: xxxDW_DestroyOwnedWindows
VOID FASTCALL IntDestroyOwnedWindows(PWND Window)
{
    HWND* List;
    HWND* phWnd;
    PWND pWnd;
    USER_REFERENCE_ENTRY Ref;

    List = IntWinListOwnedPopups(Window);
    if (!List)
        return;

    for (phWnd = List; *phWnd; ++phWnd)
    {
        pWnd = ValidateHwndNoErr(*phWnd);
        if (pWnd == NULL)
            continue;
        ASSERT(pWnd->spwndOwner == Window);
        ASSERT(pWnd != Window);

        WndSetOwner(pWnd, NULL);
        if (IntWndBelongsToThread(pWnd, PsGetCurrentThreadWin32Thread()))
        {
            UserRefObjectCo(pWnd, &Ref); // Temp HACK?
            co_UserDestroyWindow(pWnd);
            UserDerefObjectCo(pWnd); // Temp HACK?
        }
        else
        {
            ERR("IntWndBelongsToThread(0x%p) is FALSE, ignoring.\n", pWnd);
        }
    }

    ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
}

// Win: xxxDestroyWindow
BOOLEAN co_UserDestroyWindow(PVOID Object)
{
   HWND hWnd;
   PWND pwndTemp;
   PTHREADINFO ti;
   MSG msg;
   PWND Window = Object;

   ASSERT_REFS_CO(Window); // FIXME: Temp HACK?

   /* NtUserDestroyWindow does check if the window has already been destroyed
      but co_UserDestroyWindow can be called from more paths which means
      that it can also be called for a window that has already been destroyed. */
   if (!IntIsWindow(UserHMGetHandle(Window)))
   {
      TRACE("Tried to destroy a window twice\n");
      return TRUE;
   }

   hWnd = UserHMGetHandle(Window);
   ti = PsGetCurrentThreadWin32Thread();

   TRACE("co_UserDestroyWindow(Window = 0x%p, hWnd = 0x%p)\n", Window, hWnd);

   /* Check for owner thread */
   if (Window->head.pti != ti)
   {
       /* Check if we are destroying the desktop window */
       if (! ((Window->head.rpdesk->dwDTFlags & DF_DESTROYED) && Window == Window->head.rpdesk->pDeskInfo->spwnd))
       {
           EngSetLastError(ERROR_ACCESS_DENIED);
           return FALSE;
       }
   }

   /* If window was created successfully and it is hooked */
   if ((Window->state2 & WNDS2_WMCREATEMSGPROCESSED))
   {
      if (co_HOOK_CallHooks(WH_CBT, HCBT_DESTROYWND, (WPARAM) hWnd, 0))
      {
         ERR("Destroy Window WH_CBT Call Hook return!\n");
         return FALSE;
      }
   }

   if (Window->pcls->atomClassName != gpsi->atomSysClass[ICLS_IME])
   {
      if ((Window->style & (WS_POPUP|WS_CHILD)) != WS_CHILD)
      {
         if (Window->spwndOwner)
         {
            //ERR("DestroyWindow Owner out.\n");
            UserAttachThreadInput(Window->head.pti, Window->spwndOwner->head.pti, FALSE);
         }
      }
   }

   /* Inform the parent */
   if (Window->style & WS_CHILD)
   {
      IntSendParentNotify(Window, WM_DESTROY);
   }

   if (!Window->spwndOwner && !IntGetParent(Window))
   {
      co_IntShellHookNotify(HSHELL_WINDOWDESTROYED, (WPARAM) hWnd, 0);
   }

   /* Hide the window */
   if (Window->style & WS_VISIBLE)
   {
      if (Window->style & WS_CHILD)
      {
         /* Only child windows receive WM_SHOWWINDOW in DestroyWindow() */
         co_WinPosShowWindow(Window, SW_HIDE);
      }
      else
      {
         co_WinPosSetWindowPos(Window, 0, 0, 0, 0, 0, SWP_NOMOVE|SWP_NOSIZE|SWP_NOZORDER|SWP_NOACTIVATE|SWP_HIDEWINDOW );
      }
   }

   /* Adjust last active */
   if ((pwndTemp = Window->spwndOwner))
   {
      while (pwndTemp->spwndOwner)
         pwndTemp = pwndTemp->spwndOwner;

      if (pwndTemp->spwndLastActive == Window)
         WndSetLastActive(pwndTemp, Window->spwndOwner);
   }

   if (Window->spwndParent && IntIsWindow(UserHMGetHandle(Window)))
   {
      if ((Window->style & (WS_POPUP | WS_CHILD)) == WS_CHILD)
      {
         if (!IntIsTopLevelWindow(Window))
         {
            //ERR("DestroyWindow Parent out.\n");
            UserAttachThreadInput(Window->head.pti, Window->spwndParent->head.pti, FALSE);
         }
      }
   }

   if (Window->head.pti->MessageQueue->spwndActive == Window)
      Window->head.pti->MessageQueue->spwndActive = NULL;
   if (Window->head.pti->MessageQueue->spwndFocus == Window)
      Window->head.pti->MessageQueue->spwndFocus = NULL;
   if (Window->head.pti->MessageQueue->spwndActivePrev == Window)
      Window->head.pti->MessageQueue->spwndActivePrev = NULL;
   if (Window->head.pti->MessageQueue->spwndCapture == Window)
      Window->head.pti->MessageQueue->spwndCapture = NULL;

   /*
    * Check if this window is the Shell's Desktop Window. If so set hShellWindow to NULL
    */

   if (ti->pDeskInfo != NULL)
   {
      if (ti->pDeskInfo->hShellWindow == hWnd)
      {
         ERR("Destroying the ShellWindow!\n");
         ti->pDeskInfo->hShellWindow = NULL;
      }
   }

   IntEngWindowChanged(Window, WOC_DELETE);

   if (!IntIsWindow(UserHMGetHandle(Window)))
   {
      return TRUE;
   }

    /* Recursively destroy owned windows */
    if (!(Window->style & WS_CHILD))
    {
        IntDestroyOwnedWindows(Window);
    }

    /* Generate mouse move message for the next window */
    msg.message = WM_MOUSEMOVE;
    msg.wParam = UserGetMouseButtonsState();
    msg.lParam = MAKELPARAM(gpsi->ptCursor.x, gpsi->ptCursor.y);
    msg.pt = gpsi->ptCursor;
    co_MsqInsertMouseMessage(&msg, 0, 0, TRUE);

   IntNotifyWinEvent(EVENT_OBJECT_DESTROY, Window, OBJID_WINDOW, CHILDID_SELF, 0);

   /* Send destroy messages */
   IntSendDestroyMsg(UserHMGetHandle(Window));

   /* Destroy the default IME window if necessary */
   if (IS_IMM_MODE() && !(ti->TIF_flags & TIF_INCLEANUP) &&
       ti->spwndDefaultIme && (ti->spwndDefaultIme != Window) &&
       !(Window->state & WNDS_DESTROYED) && !IS_WND_IMELIKE(Window))
   {
       if (IS_WND_CHILD(Window))
       {
           if (IntImeCanDestroyDefIMEforChild(ti->spwndDefaultIme, Window))
               co_UserDestroyWindow(ti->spwndDefaultIme);
       }
       else
       {
           if (IntImeCanDestroyDefIME(ti->spwndDefaultIme, Window))
               co_UserDestroyWindow(ti->spwndDefaultIme);
       }
   }

   if (!IntIsWindow(UserHMGetHandle(Window)))
   {
      return TRUE;
   }

   /* Destroy the window storage */
   co_UserFreeWindow(Window, PsGetCurrentProcessWin32Process(), PsGetCurrentThreadWin32Thread(), TRUE);

   return TRUE;
}


/*
 * @implemented
 */
BOOLEAN APIENTRY
NtUserDestroyWindow(HWND Wnd)
{
   PWND Window;
   BOOLEAN ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserDestroyWindow\n");
   UserEnterExclusive();

   Window = UserGetWindowObject(Wnd);
   if (Window)
   {
      UserRefObjectCo(Window, &Ref); // FIXME: Dunno if win should be reffed during destroy...
      ret = co_UserDestroyWindow(Window);
      UserDerefObjectCo(Window); // FIXME: Dunno if win should be reffed during destroy...
   }

   TRACE("Leave NtUserDestroyWindow, ret=%u\n", ret);
   UserLeave();
   return ret;
}


HWND FASTCALL
IntFindWindow(PWND Parent,
              PWND ChildAfter,
              RTL_ATOM ClassAtom,
              PUNICODE_STRING WindowName)
{
   BOOL CheckWindowName;
   HWND *List, *phWnd;
   HWND Ret = NULL;
   UNICODE_STRING CurrentWindowName;

   ASSERT(Parent);

   CheckWindowName = WindowName->Buffer != 0;

   if((List = IntWinListChildren(Parent)))
   {
      phWnd = List;
      if(ChildAfter)
      {
         /* skip handles before and including ChildAfter */
         while(*phWnd && (*(phWnd++) != UserHMGetHandle(ChildAfter)))
            ;
      }

      /* search children */
      while(*phWnd)
      {
         PWND Child;
         if(!(Child = UserGetWindowObject(*(phWnd++))))
         {
            continue;
         }

         /* Do not send WM_GETTEXT messages in the kernel mode version!
            The user mode version however calls GetWindowText() which will
            send WM_GETTEXT messages to windows belonging to its processes */
         if (!ClassAtom || Child->pcls->atomNVClassName == ClassAtom)
         {
             // FIXME: LARGE_STRING truncated
             CurrentWindowName.Buffer = Child->strName.Buffer;
             CurrentWindowName.Length = (USHORT)min(Child->strName.Length, MAXUSHORT);
             CurrentWindowName.MaximumLength = (USHORT)min(Child->strName.MaximumLength, MAXUSHORT);
             if(!CheckWindowName ||
                (Child->strName.Length < 0xFFFF &&
                 !RtlCompareUnicodeString(WindowName, &CurrentWindowName, TRUE)))
             {
                Ret = UserHMGetHandle(Child);
                break;
             }
         }
      }
      ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
   }

   return Ret;
}