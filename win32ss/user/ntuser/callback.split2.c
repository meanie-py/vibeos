/* callback.split2.c — #include fragment of callback.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original callback.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


NTSTATUS
APIENTRY
co_IntClientThreadSetup(VOID)
{
   NTSTATUS Status;
   ULONG ArgumentLength, ResultLength;
   PVOID Argument, ResultPointer;

   /* Do not allow the desktop thread to do callback to user mode */
   ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);

   ArgumentLength = ResultLength = 0;
   Argument = ResultPointer = NULL;

   UserLeaveCo();

   Status = KeUserModeCallback(USER32_CALLBACK_CLIENTTHREADSTARTUP,
                               Argument,
                               ArgumentLength,
                               &ResultPointer,
                               &ResultLength);

   UserEnterCo();

   return Status;
}

HANDLE FASTCALL
co_IntCopyImage(HANDLE hnd, UINT type, INT desiredx, INT desiredy, UINT flags)
{
   HANDLE Handle;
   NTSTATUS Status;
   ULONG ArgumentLength, ResultLength;
   PVOID Argument, ResultPointer;
   PCOPYIMAGE_CALLBACK_ARGUMENTS Common;

   ArgumentLength = ResultLength = 0;
   Argument = ResultPointer = NULL;

   ArgumentLength = sizeof(COPYIMAGE_CALLBACK_ARGUMENTS);

   Argument = IntCbAllocateMemory(ArgumentLength);
   if (NULL == Argument)
   {
      ERR("CopyImage callback failed: out of memory\n");
      return 0;
   }
   Common = (PCOPYIMAGE_CALLBACK_ARGUMENTS) Argument;

   Common->hImage = hnd;
   Common->uType = type;
   Common->cxDesired = desiredx;
   Common->cyDesired = desiredy;
   Common->fuFlags = flags;

   UserLeaveCo();

   Status = KeUserModeCallback(USER32_CALLBACK_COPYIMAGE,
                               Argument,
                               ArgumentLength,
                               &ResultPointer,
                               &ResultLength);


   UserEnterCo();

   if (NT_SUCCESS(Status))
   {
      Handle = *(HANDLE*)ResultPointer;
   }
   else
   {
      ERR("CopyImage callback failed!\n");
      Handle = NULL;
   }

   IntCbFreeMemory(Argument);

   return Handle;
}

BOOL
APIENTRY
co_IntGetCharsetInfo(LCID Locale, PCHARSETINFO pCs)
{
   NTSTATUS Status;
   ULONG ArgumentLength, ResultLength;
   PVOID Argument, ResultPointer;
   PGET_CHARSET_INFO Common;

   ArgumentLength = sizeof(GET_CHARSET_INFO);

   Argument = IntCbAllocateMemory(ArgumentLength);
   if (NULL == Argument)
   {
      ERR("GetCharsetInfo callback failed: out of memory\n");
      return 0;
   }
   Common = (PGET_CHARSET_INFO) Argument;

   Common->Locale = Locale;

   ResultPointer = NULL;
   ResultLength = ArgumentLength;

   UserLeaveCo();

   Status = KeUserModeCallback(USER32_CALLBACK_GETCHARSETINFO,
                               Argument,
                               ArgumentLength,
                               &ResultPointer,
                               &ResultLength);

   if (NT_SUCCESS(Status))
   {
      _SEH2_TRY
      {
         /* Need to copy into our local buffer */
         RtlMoveMemory(Argument, ResultPointer, ArgumentLength);
      }
      _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
      {
         ERR("Failed to copy result from user mode!\n");
         Status = _SEH2_GetExceptionCode();
      }
      _SEH2_END;
   }

   UserEnterCo();

   RtlCopyMemory(pCs, &Common->Cs, sizeof(CHARSETINFO));

   IntCbFreeMemory(Argument);

   if (!NT_SUCCESS(Status))
   {
      ERR("GetCharsetInfo Failed!!\n");
      return FALSE;
   }

   return TRUE;
}

BOOL FASTCALL
co_IntSetWndIcons(VOID)
{
   NTSTATUS Status;
   ULONG ArgumentLength, ResultLength;
   PVOID Argument, ResultPointer;
   PSETWNDICONS_CALLBACK_ARGUMENTS Common;

   ResultPointer = NULL;
   ResultLength = ArgumentLength = sizeof(SETWNDICONS_CALLBACK_ARGUMENTS);

   Argument = IntCbAllocateMemory(ArgumentLength);
   if (NULL == Argument)
   {
      ERR("Set Window Icons callback failed: out of memory\n");
      return FALSE;
   }
   Common = (PSETWNDICONS_CALLBACK_ARGUMENTS) Argument;

   UserLeaveCo();

   Status = KeUserModeCallback(USER32_CALLBACK_SETWNDICONS,
                               Argument,
                               ArgumentLength,
                               &ResultPointer,
                               &ResultLength);


   UserEnterCo();

   if (!NT_SUCCESS(Status))
   {
      ERR("Set Window Icons callback failed!\n");
      IntCbFreeMemory(Argument);
      return FALSE;
   }

   RtlMoveMemory(Common, ResultPointer, ArgumentLength);
   gpsi->hIconSmWindows = Common->hIconSmWindows;
   gpsi->hIconWindows   = Common->hIconWindows;

   IntLoadSystenIcons(Common->hIconSample,   OIC_SAMPLE);
   IntLoadSystenIcons(Common->hIconHand,     OIC_HAND);
   IntLoadSystenIcons(Common->hIconQuestion, OIC_QUES);
   IntLoadSystenIcons(Common->hIconBang,     OIC_BANG);
   IntLoadSystenIcons(Common->hIconNote,     OIC_NOTE);
   IntLoadSystenIcons(gpsi->hIconWindows,    OIC_WINLOGO);
   IntLoadSystenIcons(gpsi->hIconSmWindows,  OIC_INTERNAL_WINSMALL);

   ERR("hIconSmWindows %p hIconWindows %p \n",gpsi->hIconSmWindows,gpsi->hIconWindows);

   IntCbFreeMemory(Argument);

   return TRUE;
}

VOID FASTCALL
co_IntDeliverUserAPC(VOID)
{
   ULONG ResultLength;
   PVOID ResultPointer;
   NTSTATUS Status;
   UserLeaveCo();

   Status = KeUserModeCallback(USER32_CALLBACK_DELIVERUSERAPC,
                               0,
                               0,
                               &ResultPointer,
                               &ResultLength);


   UserEnterCo();

   if (!NT_SUCCESS(Status))
   {
      ERR("Delivering User APC callback failed!\n");
   }
}

VOID FASTCALL
co_IntSetupOBM(VOID)
{
   NTSTATUS Status;
   ULONG ArgumentLength, ResultLength;
   PVOID Argument, ResultPointer;
   PSETOBM_CALLBACK_ARGUMENTS Common;

   ResultPointer = NULL;
   ResultLength = ArgumentLength = sizeof(SETOBM_CALLBACK_ARGUMENTS);

   Argument = IntCbAllocateMemory(ArgumentLength);
   if (NULL == Argument)
   {
      ERR("Set Window Icons callback failed: out of memory\n");
      return;
   }
   Common = (PSETOBM_CALLBACK_ARGUMENTS) Argument;

   UserLeaveCo();

   Status = KeUserModeCallback(USER32_CALLBACK_SETOBM,
                               Argument,
                               ArgumentLength,
                               &ResultPointer,
                               &ResultLength);


   UserEnterCo();

   if (!NT_SUCCESS(Status))
   {
      ERR("Set Window Icons callback failed!\n");
      IntCbFreeMemory(Argument);
      return;
   }

   RtlMoveMemory(Common, ResultPointer, ArgumentLength);
   RtlCopyMemory(gpsi->oembmi, Common->oembmi, sizeof(gpsi->oembmi));

   IntCbFreeMemory(Argument);
}

//
//  Called from Kernel GDI sides, no UserLeave/EnterCo required.
//
LRESULT
APIENTRY
co_UserCBClientPrinterThunk( PVOID pkt, INT InSize, PVOID pvOutData, INT OutSize )
{
   NTSTATUS Status;
   PVOID ResultPointer;

   Status = KeUserModeCallback( USER32_CALLBACK_UMPD,
                                pkt,
                                InSize,
                               &ResultPointer,
                               (PULONG)&OutSize );


   if (!NT_SUCCESS(Status))
   {
      ERR("User UMPD callback failed!\n");
      return 1;
   }

   if (OutSize) RtlMoveMemory( pvOutData, ResultPointer, OutSize );

   return 0;
}

// Win: ClientImmProcessKey
DWORD
APIENTRY
co_IntImmProcessKey(HWND hWnd, HKL hKL, UINT vKey, LPARAM lParam, DWORD dwHotKeyID)
{
    DWORD ret = 0;
    NTSTATUS Status;
    ULONG ResultLength = sizeof(DWORD);
    PVOID ResultPointer = NULL;
    IMMPROCESSKEY_CALLBACK_ARGUMENTS Common = { hWnd, hKL, vKey, lParam, dwHotKeyID };

    UserLeaveCo();
    Status = KeUserModeCallback(USER32_CALLBACK_IMMPROCESSKEY,
                                &Common,
                                sizeof(Common),
                                &ResultPointer,
                                &ResultLength);
    UserEnterCo();

    if (NT_SUCCESS(Status))
        ret = *(LPDWORD)ResultPointer;

    return ret;
}

/* Win: ClientImmLoadLayout */
BOOL
APIENTRY
co_ClientImmLoadLayout(
    _In_ HKL hKL,
    _Inout_ PIMEINFOEX pImeInfoEx)
{
    BOOL ret;
    NTSTATUS Status;
    IMMLOADLAYOUT_CALLBACK_ARGUMENTS Common = { hKL };
    ULONG ResultLength = sizeof(IMMLOADLAYOUT_CALLBACK_OUTPUT);
    PIMMLOADLAYOUT_CALLBACK_OUTPUT ResultPointer = NULL;

    RtlZeroMemory(pImeInfoEx, sizeof(IMEINFOEX));

    UserLeaveCo();
    Status = KeUserModeCallback(USER32_CALLBACK_IMMLOADLAYOUT,
                                &Common,
                                sizeof(Common),
                                (PVOID*)&ResultPointer,
                                &ResultLength);
    UserEnterCo();

    if (!NT_SUCCESS(Status) || !ResultPointer ||
        ResultLength != sizeof(IMMLOADLAYOUT_CALLBACK_OUTPUT))
    {
        ERR("0x%lX, %p, %lu\n", Status, ResultPointer, ResultLength);
        return FALSE;
    }

    Status = SafeProbeForRead(ResultPointer, ResultLength, 1);
    if (!NT_SUCCESS(Status))
    {
        ret = FALSE;
    }
    else
    {
        ret = ResultPointer->ret;
        if (ret)
            RtlCopyMemory(pImeInfoEx, &ResultPointer->iiex, sizeof(IMEINFOEX));
    }

    return ret;
}

/* EOF */
