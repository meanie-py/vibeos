/*
 * COPYRIGHT:        See COPYING in the top level directory
 * PROJECT:          ReactOS Win32k subsystem
 * PURPOSE:          Messages
 * PROGRAMER:        Casper S. Hornstrup (chorns@users.sourceforge.net)
 */

#include <win32k.h>
#include <safe_user_io.h>

#include <dde.h>

DBG_DEFAULT_CHANNEL(UserMsg);

#define PM_BADMSGFLAGS ~((QS_RAWINPUT << 16)|PM_QS_SENDMESSAGE|PM_QS_PAINT|PM_QS_POSTMESSAGE|PM_QS_INPUT|PM_NOYIELD|PM_REMOVE)

/* Strings that are OK to pass between user and kernel mode.
 * There may be other strings needed that can easily be added here. */
WCHAR StrUserKernel[3][20] = {{L"intl"}, {L"Environment"}, {L"Policy"}};

/* FUNCTIONS *****************************************************************/

/* PosInArray checks for strings that can pass between user and kernel mode.
 * See: https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-settingchange
 * It mentions 'Environment', 'intl', and 'Policy'.
 * These strings are enumerated in the StrUserKernel definition.
 * Returns: A positive integer indicating its position in the array if the
 * string is found, or returns a minus one (-1) if the string is not found. */
static INT PosInArray(_In_ PCWSTR String)
{
    INT i;

    for (i = 0; i < ARRAYSIZE(StrUserKernel); ++i)
    {
        if (wcsncmp(String, StrUserKernel[i], _countof(StrUserKernel[0])) == 0)
            return i;
    }
    return -1;
}

NTSTATUS FASTCALL
IntInitMessageImpl(VOID)
{
    return STATUS_SUCCESS;
}

NTSTATUS FASTCALL
IntCleanupMessageImpl(VOID)
{
    return STATUS_SUCCESS;
}

/* From wine: */
/* flag for messages that contain pointers */
/* 32 messages per entry, messages 0..31 map to bits 0..31 */

#define SET(msg) (1 << ((msg) & 31))

static const unsigned int message_pointer_flags[] =
{
    /* 0x00 - 0x1f */
    SET(WM_CREATE) | SET(WM_SETTEXT) | SET(WM_GETTEXT) |
    SET(WM_WININICHANGE) | SET(WM_DEVMODECHANGE),
    /* 0x20 - 0x3f */
    SET(WM_GETMINMAXINFO) | SET(WM_DRAWITEM) | SET(WM_MEASUREITEM) | SET(WM_DELETEITEM) |
    SET(WM_COMPAREITEM),
    /* 0x40 - 0x5f */
    SET(WM_WINDOWPOSCHANGING) | SET(WM_WINDOWPOSCHANGED) | SET(WM_COPYDATA) | SET(WM_COPYGLOBALDATA) | SET(WM_HELP),
    /* 0x60 - 0x7f */
    SET(WM_STYLECHANGING) | SET(WM_STYLECHANGED),
    /* 0x80 - 0x9f */
    SET(WM_NCCREATE) | SET(WM_NCCALCSIZE) | SET(WM_GETDLGCODE),
    /* 0xa0 - 0xbf */
    SET(EM_GETSEL) | SET(EM_GETRECT) | SET(EM_SETRECT) | SET(EM_SETRECTNP),
    /* 0xc0 - 0xdf */
    SET(EM_REPLACESEL) | SET(EM_GETLINE) | SET(EM_SETTABSTOPS),
    /* 0xe0 - 0xff */
    SET(SBM_GETRANGE) | SET(SBM_SETSCROLLINFO) | SET(SBM_GETSCROLLINFO) | SET(SBM_GETSCROLLBARINFO),
    /* 0x100 - 0x11f */
    0,
    /* 0x120 - 0x13f */
    0,
    /* 0x140 - 0x15f */
    SET(CB_GETEDITSEL) | SET(CB_ADDSTRING) | SET(CB_DIR) | SET(CB_GETLBTEXT) |
    SET(CB_INSERTSTRING) | SET(CB_FINDSTRING) | SET(CB_SELECTSTRING) |
    SET(CB_GETDROPPEDCONTROLRECT) | SET(CB_FINDSTRINGEXACT),
    /* 0x160 - 0x17f */
    0,
    /* 0x180 - 0x19f */
    SET(LB_ADDSTRING) | SET(LB_INSERTSTRING) | SET(LB_GETTEXT) | SET(LB_SELECTSTRING) |
    SET(LB_DIR) | SET(LB_FINDSTRING) |
    SET(LB_GETSELITEMS) | SET(LB_SETTABSTOPS) | SET(LB_ADDFILE) | SET(LB_GETITEMRECT),
    /* 0x1a0 - 0x1bf */
    SET(LB_FINDSTRINGEXACT),
    /* 0x1c0 - 0x1df */
    0,
    /* 0x1e0 - 0x1ff */
    0,
    /* 0x200 - 0x21f */
    SET(WM_NEXTMENU) | SET(WM_SIZING) | SET(WM_MOVING) | SET(WM_DEVICECHANGE),
    /* 0x220 - 0x23f */
    SET(WM_MDICREATE) | SET(WM_MDIGETACTIVE) | SET(WM_DROPOBJECT) |
    SET(WM_QUERYDROPOBJECT) | SET(WM_DRAGLOOP) | SET(WM_DRAGSELECT) | SET(WM_DRAGMOVE),
    /* 0x240 - 0x25f */
    0,
    /* 0x260 - 0x27f */
    0,
    /* 0x280 - 0x29f */
    0,
    /* 0x2a0 - 0x2bf */
    0,
    /* 0x2c0 - 0x2df */
    0,
    /* 0x2e0 - 0x2ff */
    0,
    /* 0x300 - 0x31f */
    SET(WM_ASKCBFORMATNAME)
};

/* check whether a given message type includes pointers */
static inline int is_pointer_message( UINT message, WPARAM wparam )
{
    if (message >= 8*sizeof(message_pointer_flags)) return FALSE;
    if (message == WM_DEVICECHANGE && !(wparam & 0x8000)) return FALSE;
    return (message_pointer_flags[message / 32] & SET(message)) != 0;
}
#undef SET

#define MMS_SIZE_WPARAM      -1
#define MMS_SIZE_WPARAMWCHAR -2
#define MMS_SIZE_LPARAMSZ    -3
#define MMS_SIZE_SPECIAL     -4
#define MMS_FLAG_READ        0x01
#define MMS_FLAG_WRITE       0x02
#define MMS_FLAG_READWRITE   (MMS_FLAG_READ | MMS_FLAG_WRITE)
typedef struct tagMSGMEMORY
{
    UINT Message;
    UINT Size;
    INT Flags;
}
MSGMEMORY, *PMSGMEMORY;

static MSGMEMORY g_MsgMemory[] =
{
    { WM_CREATE, MMS_SIZE_SPECIAL, MMS_FLAG_READWRITE },
    { WM_GETMINMAXINFO, sizeof(MINMAXINFO), MMS_FLAG_READWRITE },
    { WM_GETTEXT, MMS_SIZE_WPARAMWCHAR, MMS_FLAG_WRITE },
    { WM_NCCALCSIZE, MMS_SIZE_SPECIAL, MMS_FLAG_READWRITE },
    { WM_NCCREATE, MMS_SIZE_SPECIAL, MMS_FLAG_READWRITE },
    { WM_SETTEXT, MMS_SIZE_LPARAMSZ, MMS_FLAG_READ },
    { WM_STYLECHANGED, sizeof(STYLESTRUCT), MMS_FLAG_READ },
    { WM_STYLECHANGING, sizeof(STYLESTRUCT), MMS_FLAG_READWRITE },
    { WM_SETTINGCHANGE, MMS_SIZE_LPARAMSZ, MMS_FLAG_READ },
    { WM_COPYDATA, MMS_SIZE_SPECIAL, MMS_FLAG_READ },
    { WM_COPYGLOBALDATA, MMS_SIZE_WPARAM, MMS_FLAG_READ },
    { WM_WINDOWPOSCHANGED, sizeof(WINDOWPOS), MMS_FLAG_READWRITE },
    { WM_WINDOWPOSCHANGING, sizeof(WINDOWPOS), MMS_FLAG_READWRITE },
    { WM_SIZING, sizeof(RECT), MMS_FLAG_READWRITE },
    { WM_MOVING, sizeof(RECT), MMS_FLAG_READWRITE },
    { WM_MEASUREITEM, sizeof(MEASUREITEMSTRUCT), MMS_FLAG_READWRITE },
    { WM_DRAWITEM, sizeof(DRAWITEMSTRUCT), MMS_FLAG_READWRITE },
    { WM_HELP, sizeof(HELPINFO), MMS_FLAG_READWRITE },
    { WM_NEXTMENU, sizeof(MDINEXTMENU), MMS_FLAG_READWRITE },
    { WM_DEVICECHANGE, MMS_SIZE_SPECIAL, MMS_FLAG_READ },
};

static PMSGMEMORY FASTCALL
FindMsgMemory(UINT Msg)
{
    PMSGMEMORY MsgMemoryEntry;

    /* See if this message type is present in the table */
    for (MsgMemoryEntry = g_MsgMemory;
    MsgMemoryEntry < g_MsgMemory + sizeof(g_MsgMemory) / sizeof(MSGMEMORY);
    MsgMemoryEntry++)
    {
        if (Msg == MsgMemoryEntry->Message)
        {
            return MsgMemoryEntry;
        }
    }

    return NULL;
}

static UINT FASTCALL
MsgMemorySize(PMSGMEMORY MsgMemoryEntry, WPARAM wParam, LPARAM lParam)
{
    CREATESTRUCTW *Cs;
    PLARGE_STRING WindowName;
    PUNICODE_STRING ClassName;
    UINT Size = 0;

    _SEH2_TRY
    {
        if (MMS_SIZE_WPARAM == MsgMemoryEntry->Size)
        {
            Size = (UINT)wParam;
        }
        else if (MMS_SIZE_WPARAMWCHAR == MsgMemoryEntry->Size)
        {
            Size = (UINT) (wParam * sizeof(WCHAR));
        }
        else if (MMS_SIZE_LPARAMSZ == MsgMemoryEntry->Size)
        {
            // WM_SETTEXT and WM_SETTINGCHANGE can be null!
            if (!lParam)
            {
               TRACE("lParam is NULL!\n");
               Size = 0;
            }
            else
               Size = (UINT) ((wcslen((PWSTR) lParam) + 1) * sizeof(WCHAR));
        }
        else if (MMS_SIZE_SPECIAL == MsgMemoryEntry->Size)
        {
            switch(MsgMemoryEntry->Message)
            {
            case WM_CREATE:
            case WM_NCCREATE:
                Cs = (CREATESTRUCTW *) lParam;
                WindowName = (PLARGE_STRING) Cs->lpszName;
                ClassName = (PUNICODE_STRING) Cs->lpszClass;
                Size = sizeof(CREATESTRUCTW) + WindowName->Length + sizeof(WCHAR);
                if (IS_ATOM(ClassName->Buffer))
                {
                    Size += sizeof(WCHAR) + sizeof(ATOM);
                }
                else
                {
                    Size += sizeof(WCHAR) + ClassName->Length + sizeof(WCHAR);
                }
                break;

            case WM_NCCALCSIZE:
                Size = wParam ? sizeof(NCCALCSIZE_PARAMS) + sizeof(WINDOWPOS) : sizeof(RECT);
                break;

            case WM_COPYDATA:
                {
                COPYDATASTRUCT *cds = (COPYDATASTRUCT *)lParam;
                Size = sizeof(COPYDATASTRUCT) + cds->cbData;
                }
                break;

            case WM_DEVICECHANGE:
                {
                    if ( lParam && (wParam & 0x8000) )
                    {
                        DEV_BROADCAST_HDR *header = (DEV_BROADCAST_HDR *)lParam;
                        Size = header->dbch_size;
                    }
                }
                break;

            default:
                ASSERT(FALSE);
                Size = 0;
                break;
            }
        }
        else
        {
            Size = MsgMemoryEntry->Size;
        }
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        ERR("Exception caught in MsgMemorySize()! Status: 0x%x\n", _SEH2_GetExceptionCode());
        Size = 0;
    }
    _SEH2_END;
    return Size;
}

UINT lParamMemorySize(UINT Msg, WPARAM wParam, LPARAM lParam)
{
    PMSGMEMORY MsgMemoryEntry = FindMsgMemory(Msg);
    if(MsgMemoryEntry == NULL) return 0;
    return MsgMemorySize(MsgMemoryEntry, wParam, lParam);
}

static NTSTATUS
PackParam(LPARAM *lParamPacked, UINT Msg, WPARAM wParam, LPARAM lParam, BOOL NonPagedPoolNeeded)
{
    NCCALCSIZE_PARAMS *UnpackedNcCalcsize;
    NCCALCSIZE_PARAMS *PackedNcCalcsize;
    CREATESTRUCTW *UnpackedCs;
    CREATESTRUCTW *PackedCs;
    PLARGE_STRING WindowName;
    PUNICODE_STRING ClassName;
    POOL_TYPE PoolType;
    UINT Size;
    PCHAR CsData;

    *lParamPacked = lParam;

    if (NonPagedPoolNeeded)
        PoolType = NonPagedPool;
    else
        PoolType = PagedPool;

    if (WM_NCCALCSIZE == Msg && wParam)
    {

        UnpackedNcCalcsize = (NCCALCSIZE_PARAMS *) lParam;
        PackedNcCalcsize = ExAllocatePoolWithTag(PoolType,
        sizeof(NCCALCSIZE_PARAMS) + sizeof(WINDOWPOS),
        TAG_MSG);

        if (NULL == PackedNcCalcsize)
        {
            ERR("Not enough memory to pack lParam\n");
            return STATUS_NO_MEMORY;
        }
        RtlCopyMemory(PackedNcCalcsize, UnpackedNcCalcsize, sizeof(NCCALCSIZE_PARAMS));
        PackedNcCalcsize->lppos = (PWINDOWPOS) (PackedNcCalcsize + 1);
        RtlCopyMemory(PackedNcCalcsize->lppos, UnpackedNcCalcsize->lppos, sizeof(WINDOWPOS));
        *lParamPacked = (LPARAM) PackedNcCalcsize;
    }
    else if (WM_CREATE == Msg || WM_NCCREATE == Msg)
    {
        UnpackedCs = (CREATESTRUCTW *) lParam;
        WindowName = (PLARGE_STRING) UnpackedCs->lpszName;
        ClassName = (PUNICODE_STRING) UnpackedCs->lpszClass;
        Size = sizeof(CREATESTRUCTW) + WindowName->Length + sizeof(WCHAR);
        if (IS_ATOM(ClassName->Buffer))
        {
            Size += sizeof(WCHAR) + sizeof(ATOM);
        }
        else
        {
            Size += sizeof(WCHAR) + ClassName->Length + sizeof(WCHAR);
        }
        PackedCs = ExAllocatePoolWithTag(PoolType, Size, TAG_MSG);
        if (NULL == PackedCs)
        {
            ERR("Not enough memory to pack lParam\n");
            return STATUS_NO_MEMORY;
        }
        RtlCopyMemory(PackedCs, UnpackedCs, sizeof(CREATESTRUCTW));
        CsData = (PCHAR) (PackedCs + 1);
        PackedCs->lpszName = (LPCWSTR) (CsData - (PCHAR) PackedCs);
        RtlCopyMemory(CsData, WindowName->Buffer, WindowName->Length);
        CsData += WindowName->Length;
        *((WCHAR *) CsData) = L'\0';
        CsData += sizeof(WCHAR);
        PackedCs->lpszClass = (LPCWSTR) (CsData - (PCHAR) PackedCs);
        if (IS_ATOM(ClassName->Buffer))
        {
            *((WCHAR *) CsData) = L'A';
            CsData += sizeof(WCHAR);
            *((ATOM *) CsData) = (ATOM)(DWORD_PTR) ClassName->Buffer;
            CsData += sizeof(ATOM);
        }
        else
        {
            NT_ASSERT(ClassName->Buffer != NULL);
            *((WCHAR *) CsData) = L'S';
            CsData += sizeof(WCHAR);
            RtlCopyMemory(CsData, ClassName->Buffer, ClassName->Length);
            CsData += ClassName->Length;
            *((WCHAR *) CsData) = L'\0';
            CsData += sizeof(WCHAR);
        }
        ASSERT(CsData == (PCHAR) PackedCs + Size);
        *lParamPacked = (LPARAM) PackedCs;
    }
    else if (PoolType == NonPagedPool)
    {
        PMSGMEMORY MsgMemoryEntry;
        PVOID PackedData;
        SIZE_T size;

        MsgMemoryEntry = FindMsgMemory(Msg);

        if (!MsgMemoryEntry)
        {
            /* Keep previous behavior */
            return STATUS_SUCCESS;
        }
        size = MsgMemorySize(MsgMemoryEntry, wParam, lParam);
        if (!size)
        {
           ERR("No size for lParamPacked\n");
           return STATUS_SUCCESS;
        }
        PackedData = ExAllocatePoolWithTag(NonPagedPool, size, TAG_MSG);
        if (PackedData == NULL)
        {
            ERR("Not enough memory to pack lParam\n");
            return STATUS_NO_MEMORY;
        }
        RtlCopyMemory(PackedData, (PVOID)lParam, MsgMemorySize(MsgMemoryEntry, wParam, lParam));
        *lParamPacked = (LPARAM)PackedData;
    }

    return STATUS_SUCCESS;
}

static NTSTATUS
UnpackParam(LPARAM lParamPacked, UINT Msg, WPARAM wParam, LPARAM lParam, BOOL NonPagedPoolUsed)
{
    NCCALCSIZE_PARAMS *UnpackedParams;
    NCCALCSIZE_PARAMS *PackedParams;
    PWINDOWPOS UnpackedWindowPos;

    if (lParamPacked == lParam)
    {
        return STATUS_SUCCESS;
    }

    if (WM_NCCALCSIZE == Msg && wParam)
    {
        PackedParams = (NCCALCSIZE_PARAMS *) lParamPacked;
        UnpackedParams = (NCCALCSIZE_PARAMS *) lParam;
        UnpackedWindowPos = UnpackedParams->lppos;
        RtlCopyMemory(UnpackedParams, PackedParams, sizeof(NCCALCSIZE_PARAMS));
        UnpackedParams->lppos = UnpackedWindowPos;
        RtlCopyMemory(UnpackedWindowPos, PackedParams + 1, sizeof(WINDOWPOS));
        ExFreePool((PVOID) lParamPacked);

        return STATUS_SUCCESS;
    }
    else if (WM_CREATE == Msg || WM_NCCREATE == Msg)
    {
        ExFreePool((PVOID) lParamPacked);

        return STATUS_SUCCESS;
    }
    else if (NonPagedPoolUsed)
    {
        PMSGMEMORY MsgMemoryEntry;
        MsgMemoryEntry = FindMsgMemory(Msg);
        ASSERT(MsgMemoryEntry);

        if (MsgMemoryEntry->Flags == MMS_FLAG_READWRITE)
        {
            //RtlCopyMemory((PVOID)lParam, (PVOID)lParamPacked, MsgMemoryEntry->Size);
        }
        ExFreePool((PVOID) lParamPacked);
        return STATUS_SUCCESS;
    }

    ASSERT(FALSE);

    return STATUS_INVALID_PARAMETER;
}

static NTSTATUS FASTCALL
CopyMsgToKernelMem(MSG *KernelModeMsg, MSG *UserModeMsg, PMSGMEMORY MsgMemoryEntry)
{
    NTSTATUS Status;

    PVOID KernelMem;
    UINT Size;

    *KernelModeMsg = *UserModeMsg;

    /* See if this message type is present in the table */
    if (NULL == MsgMemoryEntry)
    {
        /* Not present, no copying needed */
        return STATUS_SUCCESS;
    }

    /* Determine required size */
    Size = MsgMemorySize(MsgMemoryEntry, UserModeMsg->wParam, UserModeMsg->lParam);

    if (0 != Size)
    {
        /* Allocate kernel mem */
        KernelMem = ExAllocatePoolWithTag(PagedPool, Size, TAG_MSG);
        if (NULL == KernelMem)
        {
            ERR("Not enough memory to copy message to kernel mem\n");
            return STATUS_NO_MEMORY;
        }
        KernelModeMsg->lParam = (LPARAM) KernelMem;

        /* Copy data if required */
        if (0 != (MsgMemoryEntry->Flags & MMS_FLAG_READ))
        {
            TRACE("Copy Message %u from usermode buffer\n", KernelModeMsg->message);
            /* Don't do extra testing for 1 word messages. For examples see
             * https://wiki.winehq.org/List_Of_Windows_Messages and
             * we are just handling WM_WININICHANGE here. */
            if (Size > 1 && UserModeMsg->lParam &&
                KernelModeMsg->message == WM_WININICHANGE)
            {
                WCHAR lParamMsg[_countof(StrUserKernel[0]) + 1] = { 0 };
                _SEH2_TRY
                {
                    RtlCopyMemory(lParamMsg, (WCHAR*)UserModeMsg->lParam, sizeof(lParamMsg));
                }
                _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                {
                    _SEH2_YIELD(return STATUS_ACCESS_VIOLATION);
                }
                _SEH2_END;

                /* Make sure that we have a UNICODE_NULL within lParamMsg */
                lParamMsg[ARRAYSIZE(lParamMsg) - 1] = UNICODE_NULL;

                if (!UserModeMsg->wParam && PosInArray(lParamMsg) >= 0)
                {
                    TRACE("Copy String '%S' from usermode buffer\n", lParamMsg);
                    wcscpy(KernelMem, lParamMsg);
                    return STATUS_SUCCESS;
                }
            }

            Status = MmCopyFromCaller(KernelMem, (PVOID)UserModeMsg->lParam, Size);
            if (!NT_SUCCESS(Status))
            {
                ERR("Failed to copy message to kernel: invalid usermode lParam buffer\n");
                ExFreePoolWithTag(KernelMem, TAG_MSG);
                return Status;
            }
        }
        else
        {
            /* Make sure we don't pass any secrets to usermode */
            RtlZeroMemory(KernelMem, Size);
        }
    }
    else
    {
        KernelModeMsg->lParam = 0;
    }

    return STATUS_SUCCESS;
}

static NTSTATUS FASTCALL
CopyMsgToUserMem(MSG *UserModeMsg, MSG *KernelModeMsg)
{
    NTSTATUS Status;
    PMSGMEMORY MsgMemoryEntry;
    UINT Size;

    /* See if this message type is present in the table */
    MsgMemoryEntry = FindMsgMemory(UserModeMsg->message);
    if (NULL == MsgMemoryEntry)
    {
        /* Not present, no copying needed */
        return STATUS_SUCCESS;
    }

    /* Determine required size */
    Size = MsgMemorySize(MsgMemoryEntry, UserModeMsg->wParam, UserModeMsg->lParam);

    if (0 != Size)
    {
        /* Copy data if required */
        if (0 != (MsgMemoryEntry->Flags & MMS_FLAG_WRITE))
        {
            Status = MmCopyToCaller((PVOID) UserModeMsg->lParam, (PVOID) KernelModeMsg->lParam, Size);
            if (! NT_SUCCESS(Status))
            {
                ERR("Failed to copy message from kernel: invalid usermode lParam buffer\n");
                ExFreePool((PVOID) KernelModeMsg->lParam);
                return Status;
            }
        }
        ExFreePool((PVOID) KernelModeMsg->lParam);
    }

    return STATUS_SUCCESS;
}

//
// Wakeup any thread/process waiting on idle input.
//
VOID FASTCALL
IdlePing(VOID)
{
   PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();
   PTHREADINFO pti;

   pti = PsGetCurrentThreadWin32Thread();

   if ( pti )
   {
      pti->pClientInfo->cSpins = 0; // Reset spins.

      if ( pti->pDeskInfo && pti == gptiForeground )
      {
         if ( pti->fsHooks & HOOKID_TO_FLAG(WH_FOREGROUNDIDLE) ||
              pti->pDeskInfo->fsHooks & HOOKID_TO_FLAG(WH_FOREGROUNDIDLE) )
         {
            co_HOOK_CallHooks(WH_FOREGROUNDIDLE,HC_ACTION,0,0);
         }
      }
   }

   TRACE("IdlePing ppi %p\n", ppi);
   if ( ppi && ppi->InputIdleEvent )
   {
      TRACE("InputIdleEvent\n");
      KeSetEvent( ppi->InputIdleEvent, IO_NO_INCREMENT, FALSE);
   }
}

VOID FASTCALL
IdlePong(VOID)
{
   PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();

   TRACE("IdlePong ppi %p\n", ppi);
   if ( ppi && ppi->InputIdleEvent )
   {
      KeClearEvent(ppi->InputIdleEvent);
   }
}

static BOOL is_message_broadcastable(UINT msg)
{
    return msg < WM_USER || msg >= 0xc000;
}

UINT FASTCALL
GetWakeMask(UINT first, UINT last )
{
    UINT mask = QS_POSTMESSAGE | QS_SENDMESSAGE;  /* Always selected */

    if (first || last)
    {
        if ((first <= WM_KEYLAST) && (last >= WM_KEYFIRST)) mask |= QS_KEY;
        if ( ((first <= WM_MOUSELAST) && (last >= WM_MOUSEFIRST)) ||
             ((first <= WM_NCMOUSELAST) && (last >= WM_NCMOUSEFIRST)) ) mask |= QS_MOUSE;
        if ((first <= WM_TIMER) && (last >= WM_TIMER)) mask |= QS_TIMER;
        if ((first <= WM_SYSTIMER) && (last >= WM_SYSTIMER)) mask |= QS_TIMER;
        if ((first <= WM_PAINT) && (last >= WM_PAINT)) mask |= QS_PAINT;
    }
    else mask = QS_ALLINPUT;

    return mask;
}

//
// Pass Strings to User Heap Space for Message Hook Callbacks.
//
BOOL
FASTCALL
IntMsgCreateStructW(
    PWND Window,
    CREATESTRUCTW *pCsw,
    CREATESTRUCTW *Cs,
    PVOID *ppszClass,
    PVOID *ppszName )
{
    PLARGE_STRING WindowName;
    PUNICODE_STRING ClassName;
    PVOID pszClass = NULL, pszName = NULL;

    /* Fill the new CREATESTRUCTW */
    RtlCopyMemory(pCsw, Cs, sizeof(CREATESTRUCTW));
    pCsw->style = Window->style; /* HCBT_CREATEWND needs the real window style */

    WindowName = (PLARGE_STRING)   Cs->lpszName;
    ClassName  = (PUNICODE_STRING) Cs->lpszClass;

    // Based on the assumption this is from "unicode source" user32, ReactOS, answer is yes.
    if (!IS_ATOM(ClassName->Buffer))
    {
        if (ClassName->Length)
        {
            if (Window->state & WNDS_ANSICREATOR)
            {
                ANSI_STRING AnsiString;
                AnsiString.MaximumLength = (USHORT)RtlUnicodeStringToAnsiSize(ClassName)+sizeof(CHAR);
                pszClass = UserHeapAlloc(AnsiString.MaximumLength);
                if (!pszClass)
                {
                    ERR("UserHeapAlloc() failed!\n");
                    return FALSE;
                }
                RtlZeroMemory(pszClass, AnsiString.MaximumLength);
                AnsiString.Buffer = (PCHAR)pszClass;
                RtlUnicodeStringToAnsiString(&AnsiString, ClassName, FALSE);
            }
            else
            {
                UNICODE_STRING UnicodeString;
                UnicodeString.MaximumLength = ClassName->Length + sizeof(UNICODE_NULL);
                pszClass = UserHeapAlloc(UnicodeString.MaximumLength);
                if (!pszClass)
                {
                    ERR("UserHeapAlloc() failed!\n");
                    return FALSE;
                }
                RtlZeroMemory(pszClass, UnicodeString.MaximumLength);
                UnicodeString.Buffer = (PWSTR)pszClass;
                RtlCopyUnicodeString(&UnicodeString, ClassName);
            }
            *ppszClass = pszClass;
            pCsw->lpszClass = UserHeapAddressToUser(pszClass);
        }
        else
        {
            pCsw->lpszClass = NULL;
        }
    }
    else
    {
        pCsw->lpszClass = ClassName->Buffer;
    }
    if (WindowName->Length)
    {
        UNICODE_STRING Name;
        Name.Buffer = WindowName->Buffer;
        Name.Length = (USHORT)min(WindowName->Length, MAXUSHORT); // FIXME: LARGE_STRING truncated
        Name.MaximumLength = (USHORT)min(WindowName->MaximumLength, MAXUSHORT);

        if (Window->state & WNDS_ANSICREATOR)
        {
            ANSI_STRING AnsiString;
            AnsiString.MaximumLength = (USHORT)RtlUnicodeStringToAnsiSize(&Name) + sizeof(CHAR);
            pszName = UserHeapAlloc(AnsiString.MaximumLength);
            if (!pszName)
            {
               ERR("UserHeapAlloc() failed!\n");
               return FALSE;
            }
            RtlZeroMemory(pszName, AnsiString.MaximumLength);
            AnsiString.Buffer = (PCHAR)pszName;
            RtlUnicodeStringToAnsiString(&AnsiString, &Name, FALSE);
        }
        else
        {
            UNICODE_STRING UnicodeString;
            UnicodeString.MaximumLength = Name.Length + sizeof(UNICODE_NULL);
            pszName = UserHeapAlloc(UnicodeString.MaximumLength);
            if (!pszName)
            {
               ERR("UserHeapAlloc() failed!\n");
               return FALSE;
            }
            RtlZeroMemory(pszName, UnicodeString.MaximumLength);
            UnicodeString.Buffer = (PWSTR)pszName;
            RtlCopyUnicodeString(&UnicodeString, &Name);
        }
        *ppszName = pszName;
        pCsw->lpszName = UserHeapAddressToUser(pszName);
    }
    else
    {
        pCsw->lpszName = NULL;
    }

    return TRUE;
}

static VOID FASTCALL
IntCallWndProc( PWND Window, HWND hWnd, UINT Msg, WPARAM wParam, LPARAM lParam )
{
    BOOL SameThread = FALSE;
    CWPSTRUCT CWP;
    PVOID pszClass = NULL, pszName = NULL;
    CREATESTRUCTW Csw;

    //// Check for a hook to eliminate overhead. ////
    if ( !ISITHOOKED(WH_CALLWNDPROC) && !(Window->head.rpdesk->pDeskInfo->fsHooks & HOOKID_TO_FLAG(WH_CALLWNDPROC)) )
        return;

    if (Window->head.pti == ((PTHREADINFO)PsGetCurrentThreadWin32Thread()))
        SameThread = TRUE;

    if ( Msg == WM_CREATE || Msg == WM_NCCREATE )
    {   //
        // String pointers are in user heap space, like WH_CBT HCBT_CREATEWND.
        //
        if (!IntMsgCreateStructW( Window, &Csw, (CREATESTRUCTW *)lParam, &pszClass, &pszName ))
            return;
        lParam = (LPARAM)&Csw;
    }

    CWP.hwnd    = hWnd;
    CWP.message = Msg;
    CWP.wParam  = wParam;
    CWP.lParam  = lParam;
    co_HOOK_CallHooks( WH_CALLWNDPROC, HC_ACTION, SameThread, (LPARAM)&CWP );

    if (pszName)  UserHeapFree(pszName);
    if (pszClass) UserHeapFree(pszClass);
}

static VOID FASTCALL
IntCallWndProcRet( PWND Window, HWND hWnd, UINT Msg, WPARAM wParam, LPARAM lParam, LRESULT *uResult )
{
    BOOL SameThread = FALSE;
    CWPRETSTRUCT CWPR;
    PVOID pszClass = NULL, pszName = NULL;
    CREATESTRUCTW Csw;

    if ( !ISITHOOKED(WH_CALLWNDPROCRET) && !(Window->head.rpdesk->pDeskInfo->fsHooks & HOOKID_TO_FLAG(WH_CALLWNDPROCRET)) )
        return;

    if (Window->head.pti == ((PTHREADINFO)PsGetCurrentThreadWin32Thread()))
        SameThread = TRUE;

    if ( Msg == WM_CREATE || Msg == WM_NCCREATE )
    {
        if (!IntMsgCreateStructW( Window, &Csw, (CREATESTRUCTW *)lParam, &pszClass, &pszName ))
            return;
        lParam = (LPARAM)&Csw;
    }

    CWPR.hwnd    = hWnd;
    CWPR.message = Msg;
    CWPR.wParam  = wParam;
    CWPR.lParam  = lParam;
    CWPR.lResult = uResult ? (*uResult) : 0;
    co_HOOK_CallHooks( WH_CALLWNDPROCRET, HC_ACTION, SameThread, (LPARAM)&CWPR );

    if (pszName)  UserHeapFree(pszName);
    if (pszClass) UserHeapFree(pszClass);
}

static LRESULT handle_internal_message( PWND pWnd, UINT msg, WPARAM wparam, LPARAM lparam )
{
    LRESULT lRes;
//    USER_REFERENCE_ENTRY Ref;
//    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

    if (!pWnd || UserIsDesktopWindow(pWnd) || UserIsMessageWindow(pWnd))
       return 0;

    TRACE("Internal Event Msg 0x%x hWnd 0x%p\n", msg, UserHMGetHandle(pWnd));

    switch(msg)
    {
       case WM_ASYNC_SHOWWINDOW:
          return co_WinPosShowWindow( pWnd, wparam );
       case WM_ASYNC_SETWINDOWPOS:
       {
          PWINDOWPOS winpos = (PWINDOWPOS)lparam;
          if (!winpos) return 0;
          lRes = co_WinPosSetWindowPos( pWnd,
                                        winpos->hwndInsertAfter,
                                        winpos->x,
                                        winpos->y,
                                        winpos->cx,
                                        winpos->cy,
                                        winpos->flags);
          ExFreePoolWithTag(winpos, USERTAG_SWP);
          return lRes;
       }
       case WM_ASYNC_DESTROYWINDOW:
       {
          TRACE("WM_ASYNC_DESTROYWINDOW\n");
          if (pWnd->style & WS_CHILD)
             return co_UserFreeWindow(pWnd, PsGetCurrentProcessWin32Process(), PsGetCurrentThreadWin32Thread(), TRUE);
          else
             co_UserDestroyWindow(pWnd);
       }
    }
    return 0;
}

static LRESULT handle_internal_events( PTHREADINFO pti, PWND pWnd, DWORD dwQEvent, LONG_PTR ExtraInfo, PMSG pMsg)
{
    LRESULT Result = 0;

    switch(dwQEvent)
    {
       case POSTEVENT_NWE:
       {
          co_EVENT_CallEvents( pMsg->message, pMsg->hwnd, pMsg->wParam, ExtraInfo);
       }
       break;
       case POSTEVENT_SAW:
       {
          //ERR("HIE : SAW : pti 0x%p hWnd 0x%p\n",pti,pMsg->hwnd);
          IntActivateWindow((PWND)pMsg->wParam, pti, (HANDLE)pMsg->lParam, (DWORD)ExtraInfo);
       }
       break;
       case POSTEVENT_DAW:
       {
          //ERR("HIE : DAW : pti 0x%p tid 0x%p hWndPrev 0x%p\n",pti,ExtraInfo,pMsg->hwnd);
          IntDeactivateWindow(pti, (HANDLE)ExtraInfo);
       }
       break;
    }
    return Result;
}

LRESULT FASTCALL
IntDispatchMessage(PMSG pMsg)
{
    LONG Time;
    LRESULT retval = 0;
    PTHREADINFO pti;
    PWND Window = NULL;
    BOOL DoCallBack = TRUE;

    if (pMsg->hwnd)
    {
        Window = UserGetWindowObject(pMsg->hwnd);
        if (!Window) return 0;
    }

    pti = PsGetCurrentThreadWin32Thread();

    if ( Window && Window->head.pti != pti)
    {
       EngSetLastError( ERROR_MESSAGE_SYNC_ONLY );
       return 0;
    }

    if (((pMsg->message == WM_SYSTIMER) ||
         (pMsg->message == WM_TIMER)) &&
         (pMsg->lParam) )
    {
        if (pMsg->message == WM_TIMER)
        {
            if (ValidateTimerCallback(pti,pMsg->lParam))
            {
                Time = EngGetTickCount32();
                retval = co_IntCallWindowProc((WNDPROC)pMsg->lParam,
                                              TRUE,
                                              pMsg->hwnd,
                                              WM_TIMER,
                                              pMsg->wParam,
                                              (LPARAM)Time,
                                              -1);
            }
            return retval;
        }
        else
        {
            PTIMER pTimer = FindSystemTimer(pMsg);
            if (pTimer && pTimer->pfn)
            {
                Time = EngGetTickCount32();
                pTimer->pfn(pMsg->hwnd, WM_SYSTIMER, (UINT)pMsg->wParam, Time);
            }
            return 0;
        }
    }
    // Need a window!
    if ( !Window ) return 0;

    if (pMsg->message == WM_PAINT) Window->state |= WNDS_PAINTNOTPROCESSED;

    if ( Window->state & WNDS_SERVERSIDEWINDOWPROC )
    {
       TRACE("Dispatch: Server Side Window Procedure\n");
       switch(Window->fnid)
       {
          case FNID_DESKTOP:
            DoCallBack = !DesktopWindowProc( Window,
                                             pMsg->message,
                                             pMsg->wParam,
                                             pMsg->lParam,
                                            &retval);
            break;
          case FNID_MESSAGEWND:
            DoCallBack = !UserMessageWindowProc( Window,
                                                 pMsg->message,
                                                 pMsg->wParam,
                                                 pMsg->lParam,
                                                &retval);
            break;
          case FNID_MENU:
            DoCallBack = !PopupMenuWndProc( Window,
                                            pMsg->message,
                                            pMsg->wParam,
                                            pMsg->lParam,
                                           &retval);
            break;
       }
    }

    /* Since we are doing a callback on the same thread right away, there is
       no need to copy the lparam to kernel mode and then back to usermode.
       We just pretend it isn't a pointer */

    if (DoCallBack)
    retval = co_IntCallWindowProc( Window->lpfnWndProc,
                                   !Window->Unicode,
                                   pMsg->hwnd,
                                   pMsg->message,
                                   pMsg->wParam,
                                   pMsg->lParam,
                                   -1);

    if ( pMsg->message == WM_PAINT &&
         VerifyWnd(Window) &&
         Window->state & WNDS_PAINTNOTPROCESSED ) // <--- Cleared, paint was already processed!
    {
        Window->state2 &= ~WNDS2_WMPAINTSENT;
        /* send a WM_ERASEBKGND if the non-client area is still invalid */
        ERR("Message WM_PAINT count %d Internal Paint Set? %s\n",Window->head.pti->cPaintsReady, Window->state & WNDS_INTERNALPAINT ? "TRUE" : "FALSE");
        IntPaintWindow( Window );
    }

    return retval;
}

/*
 * Internal version of PeekMessage() doing all the work
 *
 * MSDN:
 *   Sent messages
 *   Posted messages
 *   Input (hardware) messages and system internal events
 *   Sent messages (again)
 *   WM_PAINT messages
 *   WM_TIMER messages
 */
BOOL APIENTRY
co_IntPeekMessage( PMSG Msg,
                   PWND Window,
                   UINT MsgFilterMin,
                   UINT MsgFilterMax,
                   UINT RemoveMsg,
                   LONG_PTR *ExtraInfo,
                   BOOL bGMSG )
{
    PTHREADINFO pti;
    BOOL RemoveMessages;
    UINT ProcessMask;
    BOOL Hit = FALSE;

    pti = PsGetCurrentThreadWin32Thread();

    RemoveMessages = RemoveMsg & PM_REMOVE;
    ProcessMask = HIWORD(RemoveMsg);

 /* Hint, "If wMsgFilterMin and wMsgFilterMax are both zero, PeekMessage returns
    all available messages (that is, no range filtering is performed)".        */
    if (!ProcessMask) ProcessMask = (QS_ALLPOSTMESSAGE|QS_ALLINPUT);

    IdlePong();

    do
    {
        /* Update the last message-queue access time */
        pti->pcti->timeLastRead = EngGetTickCount32();

        // Post mouse moves while looping through peek messages.
        if (pti->MessageQueue->QF_flags & QF_MOUSEMOVED)
        {
           IntCoalesceMouseMove(pti);
        }

        /* Dispatch sent messages here. */
        while ( co_MsqDispatchOneSentMessage(pti) )
        {
           /* if some PM_QS* flags were specified, only handle sent messages from now on */
           if (HIWORD(RemoveMsg) && !bGMSG) Hit = TRUE; // wine does this; ProcessMask = QS_SENDMESSAGE;
        }
        if (Hit) return FALSE;

        /* Clear changed bits so we can wait on them if we don't find a message */
        if (ProcessMask & QS_POSTMESSAGE)
        {
           pti->pcti->fsChangeBits &= ~(QS_POSTMESSAGE | QS_HOTKEY | QS_TIMER);
           if (MsgFilterMin == 0 && MsgFilterMax == 0) // Wine hack does this; ~0U)
           {
              pti->pcti->fsChangeBits &= ~QS_ALLPOSTMESSAGE;
           }
        }

        if (ProcessMask & QS_INPUT)
        {
           pti->pcti->fsChangeBits &= ~QS_INPUT;
        }

        /* Now check for normal messages. */
        if (( (ProcessMask & QS_POSTMESSAGE) ||
              (ProcessMask & QS_HOTKEY) ) &&
            MsqPeekMessage( pti,
                            RemoveMessages,
                            Window,
                            MsgFilterMin,
                            MsgFilterMax,
                            ProcessMask,
                            ExtraInfo,
                            0,
                            Msg ))
        {
            goto GotMessage;
        }

        /* Only check for quit messages if not posted messages pending. */
        if (ProcessMask & QS_POSTMESSAGE && pti->QuitPosted)
        {
            /* According to the PSDK, WM_QUIT messages are always returned, regardless
               of the filter specified */
            Msg->hwnd = NULL;
            Msg->message = WM_QUIT;
            Msg->wParam = pti->exitCode;
            Msg->lParam = 0;
            if (RemoveMessages)
            {
                pti->QuitPosted = FALSE;
                ClearMsgBitsMask(pti, QS_POSTMESSAGE);
                pti->pcti->fsWakeBits &= ~QS_ALLPOSTMESSAGE;
                pti->pcti->fsChangeBits &= ~QS_ALLPOSTMESSAGE;
            }
            goto GotMessage;
        }

        /* Check for hardware events. */
        if ((ProcessMask & QS_INPUT) &&
            co_MsqPeekHardwareMessage( pti,
                                       RemoveMessages,
                                       Window,
                                       MsgFilterMin,
                                       MsgFilterMax,
                                       ProcessMask,
                                       Msg))
        {
            goto GotMessage;
        }

        /* Now check for System Event messages. */
        {
           LONG_PTR eExtraInfo;
           MSG eMsg;
           DWORD dwQEvent;
           if (MsqPeekMessage( pti,
                               TRUE,
                               Window,
                               0,
                               0,
                               QS_EVENT,
                              &eExtraInfo,
                              &dwQEvent,
                              &eMsg ))
           {
              handle_internal_events( pti, Window, dwQEvent, eExtraInfo, &eMsg);
              continue;
           }
        }

        /* Check for sent messages again. */
        while ( co_MsqDispatchOneSentMessage(pti) )
        {
           if (HIWORD(RemoveMsg) && !bGMSG) Hit = TRUE;
        }
        if (Hit) return FALSE;

        /* Check for paint messages. */
        if ((ProcessMask & QS_PAINT) &&
            pti->cPaintsReady &&
            IntGetPaintMessage( Window,
                                MsgFilterMin,
                                MsgFilterMax,
                                pti,
                                Msg,
                                RemoveMessages))
        {
            goto GotMessage;
        }

       /* This is correct, check for the current threads timers waiting to be
          posted to this threads message queue. If any we loop again.
        */
        if ((ProcessMask & QS_TIMER) &&
            PostTimerMessages(Window))
        {
            continue;
        }

        return FALSE;
    }
    while (TRUE);

GotMessage:
    /* Update the last message-queue access time */
    pti->pcti->timeLastRead = EngGetTickCount32();
    return TRUE;
}
/* --- pretranslation split: the remaining function groups live in message.split2..3.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "message.split2.c"
#include "message.split3.c"
