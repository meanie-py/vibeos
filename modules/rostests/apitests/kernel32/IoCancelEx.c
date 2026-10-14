/*
 * PROJECT:     ReactOS api tests
 * LICENSE:     LGPL-2.0-or-later (https://spdx.org/licenses/LGPL-2.0-or-later)
 * PURPOSE:     Vista's cross-thread cancel and batched completion calls: CancelIoEx,
 *              CancelSynchronousIo, GetQueuedCompletionStatusEx, FlushProcessWriteBuffers.
 *              Looked up at run time so the same binary grades Windows 11 (winref).
 */
#include "precomp.h"

typedef struct _OVERLAPPED_ENTRY_T {
    ULONG_PTR lpCompletionKey;
    LPOVERLAPPED lpOverlapped;
    ULONG_PTR Internal;
    DWORD dwNumberOfBytesTransferred;
} OVERLAPPED_ENTRY_T;

static BOOL (WINAPI *pCancelIoEx)(HANDLE, LPOVERLAPPED);
static BOOL (WINAPI *pCancelSynchronousIo)(HANDLE);
static BOOL (WINAPI *pGetQueuedCompletionStatusEx)(HANDLE, OVERLAPPED_ENTRY_T *, ULONG, PULONG, DWORD, BOOL);
static VOID (WINAPI *pFlushProcessWriteBuffers)(VOID);

static LONG s_apc_ran;

static VOID CALLBACK CountApc(ULONG_PTR Param)
{
    InterlockedIncrement(&s_apc_ran);
}

static void Test_GetQueuedCompletionStatusEx(void)
{
    OVERLAPPED_ENTRY_T Entries[8];
    ULONG Removed, i;
    HANDLE Port;
    BOOL Ret;

    Port = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 0);
    ok(Port != NULL, "CreateIoCompletionPort failed: %lu\n", GetLastError());
    if (!Port)
        return;

    for (i = 1; i <= 3; i++)
        PostQueuedCompletionStatus(Port, i * 10, i, (LPOVERLAPPED)(ULONG_PTR)(i * 0x100));

    /* All three come back in one call, in order. */
    Removed = 0xdead;
    Ret = pGetQueuedCompletionStatusEx(Port, Entries, 8, &Removed, 0, FALSE);
    ok(Ret == TRUE, "batch: Ret %d, error %lu\n", Ret, GetLastError());
    ok(Removed == 3, "batch: Removed %lu\n", Removed);
    for (i = 0; i < 3 && i < Removed; i++)
    {
        ok(Entries[i].lpCompletionKey == i + 1, "entry %lu key %Iu\n", i, Entries[i].lpCompletionKey);
        ok(Entries[i].dwNumberOfBytesTransferred == (i + 1) * 10, "entry %lu bytes %lu\n",
           i, Entries[i].dwNumberOfBytesTransferred);
        ok(Entries[i].lpOverlapped == (LPOVERLAPPED)(ULONG_PTR)((i + 1) * 0x100),
           "entry %lu overlapped %p\n", i, Entries[i].lpOverlapped);
    }

    /* A count below what is queued takes only that many. */
    for (i = 1; i <= 3; i++)
        PostQueuedCompletionStatus(Port, 0, i, NULL);
    Ret = pGetQueuedCompletionStatusEx(Port, Entries, 2, &Removed, 0, FALSE);
    ok(Ret == TRUE && Removed == 2, "partial: Ret %d Removed %lu\n", Ret, Removed);
    Ret = pGetQueuedCompletionStatusEx(Port, Entries, 8, &Removed, 0, FALSE);
    ok(Ret == TRUE && Removed == 1, "rest: Ret %d Removed %lu\n", Ret, Removed);
    ok(Entries[0].lpCompletionKey == 3, "rest: key %Iu\n", Entries[0].lpCompletionKey);

    /* Empty: times out. */
    SetLastError(0xdeadbeef);
    Removed = 0xdead;
    Ret = pGetQueuedCompletionStatusEx(Port, Entries, 8, &Removed, 0, FALSE);
    ok(Ret == FALSE, "empty: Ret %d\n", Ret);
    ok(GetLastError() == WAIT_TIMEOUT, "empty: error %lu\n", GetLastError());
    ok(Removed == 0, "empty: Removed %lu\n", Removed);

    /* Alertable: a queued APC runs and the call reports it. */
    s_apc_ran = 0;
    QueueUserAPC(CountApc, GetCurrentThread(), 0);
    SetLastError(0xdeadbeef);
    Ret = pGetQueuedCompletionStatusEx(Port, Entries, 8, &Removed, 1000, TRUE);
    ok(Ret == FALSE, "alertable: Ret %d\n", Ret);
    ok(GetLastError() == WAIT_IO_COMPLETION, "alertable: error %lu\n", GetLastError());
    ok(s_apc_ran == 1, "alertable: APC ran %ld times\n", s_apc_ran);

    /* Not alertable: the APC waits. */
    s_apc_ran = 0;
    QueueUserAPC(CountApc, GetCurrentThread(), 0);
    SetLastError(0xdeadbeef);
    Ret = pGetQueuedCompletionStatusEx(Port, Entries, 8, &Removed, 50, FALSE);
    ok(Ret == FALSE && GetLastError() == WAIT_TIMEOUT, "non-alertable: Ret %d error %lu\n",
       Ret, GetLastError());
    ok(s_apc_ran == 0, "non-alertable: APC ran %ld times\n", s_apc_ran);
    SleepEx(0, TRUE);
    ok(s_apc_ran == 1, "drained: APC ran %ld times\n", s_apc_ran);

    /* A zero count is refused. */
    SetLastError(0xdeadbeef);
    Ret = pGetQueuedCompletionStatusEx(Port, Entries, 0, &Removed, 0, FALSE);
    ok(Ret == FALSE, "zero count: Ret %d\n", Ret);
    ok(GetLastError() == ERROR_INVALID_PARAMETER, "zero count: error %lu\n", GetLastError());

    CloseHandle(Port);
}

typedef struct _CANCEL_ARGS {
    HANDLE File;
    LPOVERLAPPED Overlapped;
    BOOL Ret;
    DWORD Error;
} CANCEL_ARGS;

static DWORD WINAPI CancelFromOtherThread(LPVOID Param)
{
    CANCEL_ARGS *Args = Param;
    SetLastError(0xdeadbeef);
    Args->Ret = pCancelIoEx(Args->File, Args->Overlapped);
    Args->Error = GetLastError();
    return 0;
}

static void Test_CancelIoEx(void)
{
    WCHAR Name[64];
    HANDLE Server, Client, Event, Thread;
    OVERLAPPED Ov = { 0 };
    CANCEL_ARGS Args;
    char Buffer[16];
    DWORD Read;
    BOOL Ret;

    _snwprintf(Name, _countof(Name), L"\\\\.\\pipe\\vibeos_cancelioex_%lu", GetCurrentProcessId());
    Server = CreateNamedPipeW(Name, PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
                              PIPE_TYPE_BYTE | PIPE_WAIT, 1, 64, 64, 0, NULL);
    ok(Server != INVALID_HANDLE_VALUE, "CreateNamedPipe failed: %lu\n", GetLastError());
    if (Server == INVALID_HANDLE_VALUE)
        return;
    Client = CreateFileW(Name, GENERIC_READ | GENERIC_WRITE, 0, NULL, OPEN_EXISTING, 0, NULL);
    ok(Client != INVALID_HANDLE_VALUE, "CreateFile failed: %lu\n", GetLastError());
    Event = CreateEventW(NULL, TRUE, FALSE, NULL);
    Ov.hEvent = Event;

    /* A read with no data pends; another thread cancels it by its OVERLAPPED. */
    Ret = ReadFile(Server, Buffer, sizeof(Buffer), NULL, &Ov);
    ok(Ret == FALSE && GetLastError() == ERROR_IO_PENDING, "ReadFile: Ret %d error %lu\n",
       Ret, GetLastError());
    Args.File = Server;
    Args.Overlapped = &Ov;
    Thread = CreateThread(NULL, 0, CancelFromOtherThread, &Args, 0, NULL);
    ok(WaitForSingleObject(Thread, 5000) == WAIT_OBJECT_0, "cancel thread hung\n");
    CloseHandle(Thread);
    ok(Args.Ret == TRUE, "CancelIoEx: Ret %d error %lu\n", Args.Ret, Args.Error);
    SetLastError(0xdeadbeef);
    Ret = GetOverlappedResult(Server, &Ov, &Read, TRUE);
    ok(Ret == FALSE && GetLastError() == ERROR_OPERATION_ABORTED,
       "cancelled read: Ret %d error %lu\n", Ret, GetLastError());

    /* Nothing pending: not found. */
    Thread = CreateThread(NULL, 0, CancelFromOtherThread, &Args, 0, NULL);
    WaitForSingleObject(Thread, 5000);
    CloseHandle(Thread);
    ok(Args.Ret == FALSE, "nothing pending: Ret %d\n", Args.Ret);
    ok(Args.Error == ERROR_NOT_FOUND, "nothing pending: error %lu\n", Args.Error);

    /* NULL: every request on the handle. */
    ResetEvent(Event);
    Ret = ReadFile(Server, Buffer, sizeof(Buffer), NULL, &Ov);
    ok(Ret == FALSE && GetLastError() == ERROR_IO_PENDING, "ReadFile 2: error %lu\n", GetLastError());
    Args.Overlapped = NULL;
    Thread = CreateThread(NULL, 0, CancelFromOtherThread, &Args, 0, NULL);
    WaitForSingleObject(Thread, 5000);
    CloseHandle(Thread);
    ok(Args.Ret == TRUE, "CancelIoEx(NULL): Ret %d error %lu\n", Args.Ret, Args.Error);
    Ret = GetOverlappedResult(Server, &Ov, &Read, TRUE);
    ok(Ret == FALSE && GetLastError() == ERROR_OPERATION_ABORTED,
       "cancelled read 2: Ret %d error %lu\n", Ret, GetLastError());

    CloseHandle(Event);
    CloseHandle(Client);
    CloseHandle(Server);
}

typedef struct _SYNC_READ_ARGS {
    HANDLE Pipe;
    BOOL Ret;
    DWORD Error;
} SYNC_READ_ARGS;

static DWORD WINAPI BlockingRead(LPVOID Param)
{
    SYNC_READ_ARGS *Args = Param;
    char Buffer[16];
    DWORD Read;
    SetLastError(0xdeadbeef);
    Args->Ret = ReadFile(Args->Pipe, Buffer, sizeof(Buffer), &Read, NULL);
    Args->Error = GetLastError();
    return 0;
}

static void Test_CancelSynchronousIo(void)
{
    HANDLE ReadEnd, WriteEnd, Thread;
    SYNC_READ_ARGS Args;
    DWORD Written;
    BOOL Ret;
    int Tries;

    ok(CreatePipe(&ReadEnd, &WriteEnd, NULL, 0), "CreatePipe failed: %lu\n", GetLastError());

    Args.Pipe = ReadEnd;
    Thread = CreateThread(NULL, 0, BlockingRead, &Args, 0, NULL);

    /* Retry until the read is actually in flight. */
    Ret = FALSE;
    for (Tries = 0; Tries < 50 && !Ret; Tries++)
    {
        Sleep(20);
        Ret = pCancelSynchronousIo(Thread);
    }
    ok(Ret == TRUE, "CancelSynchronousIo: error %lu after %d tries\n", GetLastError(), Tries);
    if (WaitForSingleObject(Thread, 5000) != WAIT_OBJECT_0)
    {
        ok(FALSE, "the blocked read was not cancelled\n");
        WriteFile(WriteEnd, "x", 1, &Written, NULL); /* let it go */
        WaitForSingleObject(Thread, 5000);
    }
    else
    {
        ok(Args.Ret == FALSE && Args.Error == ERROR_OPERATION_ABORTED,
           "cancelled sync read: Ret %d error %lu\n", Args.Ret, Args.Error);
    }

    /* A thread with nothing in flight: not found. */
    SetLastError(0xdeadbeef);
    Ret = pCancelSynchronousIo(Thread);
    ok(Ret == FALSE && GetLastError() == ERROR_NOT_FOUND, "idle thread: Ret %d error %lu\n",
       Ret, GetLastError());

    CloseHandle(Thread);
    CloseHandle(ReadEnd);
    CloseHandle(WriteEnd);
}

START_TEST(IoCancelEx)
{
    HMODULE Kernel32 = GetModuleHandleW(L"kernel32.dll");

    pCancelIoEx = (void *)GetProcAddress(Kernel32, "CancelIoEx");
    pCancelSynchronousIo = (void *)GetProcAddress(Kernel32, "CancelSynchronousIo");
    pGetQueuedCompletionStatusEx = (void *)GetProcAddress(Kernel32, "GetQueuedCompletionStatusEx");
    pFlushProcessWriteBuffers = (void *)GetProcAddress(Kernel32, "FlushProcessWriteBuffers");
    if (!pCancelIoEx || !pCancelSynchronousIo || !pGetQueuedCompletionStatusEx ||
        !pFlushProcessWriteBuffers)
    {
        ok(FALSE, "missing exports: %p %p %p %p\n", pCancelIoEx, pCancelSynchronousIo,
           pGetQueuedCompletionStatusEx, pFlushProcessWriteBuffers);
        return;
    }

    Test_GetQueuedCompletionStatusEx();
    Test_CancelIoEx();
    Test_CancelSynchronousIo();
    pFlushProcessWriteBuffers();
    ok(TRUE, "FlushProcessWriteBuffers returned\n");
}
