/* iocpprobe: Chrome's processes talk over Mojo channels, which are overlapped named pipes bound to
 * an I/O completion port (mojo/core/channel_win.cc, base/message_loop/message_pump_win.cc). The
 * browser and its renderers start, then sit idle in NtRemoveIoCompletion forever: a channel
 * message was sent and its completion never arrived, or arrived in a shape the pump drops.
 * This makes the same calls, in the same order, and prints what each returned. Each line is
 * one fact, so it diffs against iocpprobe.win11.txt.
 *
 * What Mojo relies on, and what each case pins:
 *  - an overlapped ReadFile/WriteFile that completes AT ONCE (returns TRUE) still queues a
 *    packet on the port: channel_win.cc ignores the synchronous result and waits for it;
 *  - a pending read completes with the right key, OVERLAPPED pointer and byte count when the
 *    peer writes;
 *  - PostQueuedCompletionStatus (the pump's ScheduleWork) arrives with its key and pointer;
 *  - ConnectNamedPipe on an overlapped pipe: pending until the client opens it, then a packet;
 *    ERROR_PIPE_CONNECTED, with no packet, if the client was first;
 *  - GetQueuedCompletionStatusEx (Vista) returns the same entries;
 *  - all of it across a process boundary, with the pipe end inherited by a child (how the
 *    browser hands a renderer its end: --mojo-platform-channel-handle=N). */
#include <windows.h>
#include <stdio.h>
#include <string.h>

#define KEY_SRV 0x5151
#define KEY_CLI 0xC1C1
#define KEY_POST 0x7777

static void say(const char *fmt, ...)
{
    char buf[512];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof buf, fmt, ap);
    va_end(ap);
    printf("[IOCP] %s\n", buf);
    fflush(stdout);
}

/* One dequeue with a short wait, printed relative to the OVERLAPPEDs we know about. */
static void dequeue(const char *what, HANDLE port, OVERLAPPED *a, OVERLAPPED *b)
{
    DWORD bytes = 0xFFFFFFFF;
    ULONG_PTR key = 0;
    OVERLAPPED *ov = (OVERLAPPED *)1;
    BOOL ok = GetQueuedCompletionStatus(port, &bytes, &key, &ov, 2000);
    DWORD err = ok ? 0 : GetLastError();
    const char *which = ov == NULL ? "NULL" : ov == a ? "A" : ov == b ? "B" : "other";
    if (!ok && ov == NULL)
        say("%s: nothing (err %lu)", what, err);
    else
        say("%s: ok=%d err=%lu key=%#lx ov=%s bytes=%lu", what, ok, err, (unsigned long)key, which,
            (unsigned long)bytes);
}

static void nothing_more(const char *what, HANDLE port)
{
    DWORD bytes;
    ULONG_PTR key;
    OVERLAPPED *ov = NULL;
    BOOL ok = GetQueuedCompletionStatus(port, &bytes, &key, &ov, 200);
    if (!ok && ov == NULL)
        say("%s: queue empty (err %lu)", what, GetLastError());
    else
        say("%s: UNEXPECTED packet key=%#lx bytes=%lu", what, (unsigned long)key, (unsigned long)bytes);
}

static HANDLE make_server(const WCHAR *name)
{
    return CreateNamedPipeW(name,
                            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
                            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE, 1, 4096, 4096, 5000, NULL);
}

static HANDLE open_client(const WCHAR *name, BOOL inherit)
{
    SECURITY_ATTRIBUTES sa = { sizeof sa, NULL, inherit };
    return CreateFileW(name, GENERIC_READ | GENERIC_WRITE, 0, &sa, OPEN_EXISTING,
                       SECURITY_SQOS_PRESENT | SECURITY_ANONYMOUS | FILE_FLAG_OVERLAPPED, NULL);
}

static void in_process(void)
{
    const WCHAR *name = L"\\\\.\\pipe\\iocpprobe.1";
    char buf[64];
    DWORD n;
    OVERLAPPED a = { 0 }, b = { 0 };
    HANDLE srv = make_server(name);
    say("server pipe: %s (err %lu)", srv != INVALID_HANDLE_VALUE ? "ok" : "FAIL", GetLastError());
    if (srv == INVALID_HANDLE_VALUE)
        return;
    HANDLE port = CreateIoCompletionPort(srv, NULL, KEY_SRV, 1);
    say("port + server: %s", port ? "ok" : "FAIL");

    /* Connect before the client exists: pending, then a packet once it opens. */
    BOOL ok = ConnectNamedPipe(srv, &a);
    say("ConnectNamedPipe before client: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    HANDLE cli = open_client(name, FALSE);
    say("client open: %s (err %lu)", cli != INVALID_HANDLE_VALUE ? "ok" : "FAIL", GetLastError());
    if (cli == INVALID_HANDLE_VALUE)
        return;
    say("port + client: %s", CreateIoCompletionPort(cli, port, KEY_CLI, 0) == port ? "ok" : "FAIL");
    dequeue("connect completion", port, &a, &b);

    /* A write with room in the buffer completes at once; the packet must still be queued. */
    memset(&b, 0, sizeof b);
    ok = WriteFile(cli, "hello", 5, &n, &b);
    say("client WriteFile 5: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    dequeue("write completion", port, &a, &b);

    /* Data is waiting, so this read completes at once too; Mojo waits for the packet anyway. */
    memset(&a, 0, sizeof a);
    memset(buf, 0, sizeof buf);
    ok = ReadFile(srv, buf, sizeof buf, &n, &a);
    say("server ReadFile with data waiting: ret=%d err=%lu buf=%s", ok, ok ? 0 : GetLastError(), buf);
    dequeue("sync read completion", port, &a, &b);

    /* Nothing waiting: pending, then completes when the peer writes. */
    memset(&a, 0, sizeof a);
    memset(buf, 0, sizeof buf);
    ok = ReadFile(srv, buf, sizeof buf, &n, &a);
    say("server ReadFile, empty pipe: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    nothing_more("before the peer writes", port);
    memset(&b, 0, sizeof b);
    ok = WriteFile(cli, "world!", 6, &n, &b);
    say("client WriteFile 6: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    dequeue("completion 1 of 2", port, &a, &b);
    dequeue("completion 2 of 2", port, &a, &b);
    say("pending read got: %s", buf);

    /* The pump's ScheduleWork. */
    ok = PostQueuedCompletionStatus(port, 0, KEY_POST, (OVERLAPPED *)&b);
    say("PostQueuedCompletionStatus: ret=%d", ok);
    dequeue("posted", port, &a, &b);

    /* The Vista batch form, which message_pump_win also uses. */
    PostQueuedCompletionStatus(port, 3, KEY_POST, &a);
    PostQueuedCompletionStatus(port, 4, KEY_POST, &b);
    OVERLAPPED_ENTRY e[4];
    ULONG got = 0;
    memset(e, 0, sizeof e);
    ok = GetQueuedCompletionStatusEx(port, e, 4, &got, 2000, FALSE);
    say("GetQueuedCompletionStatusEx: ret=%d got=%lu", ok, got);
    for (ULONG i = 0; i < got && i < 4; i++)
        say("  entry %lu: key=%#lx ov=%s bytes=%lu", i, (unsigned long)e[i].lpCompletionKey,
            e[i].lpOverlapped == &a ? "A" : e[i].lpOverlapped == &b ? "B" : "other",
            (unsigned long)e[i].dwNumberOfBytesTransferred);

    /* A second server instance whose client connects BEFORE ConnectNamedPipe. */
    const WCHAR *name2 = L"\\\\.\\pipe\\iocpprobe.2";
    HANDLE srv2 = make_server(name2);
    HANDLE port2 = CreateIoCompletionPort(srv2, NULL, KEY_SRV, 1);
    HANDLE cli2 = open_client(name2, FALSE);
    say("pipe 2: server %s, client %s", srv2 != INVALID_HANDLE_VALUE ? "ok" : "FAIL",
        cli2 != INVALID_HANDLE_VALUE ? "ok" : "FAIL");
    memset(&a, 0, sizeof a);
    ok = ConnectNamedPipe(srv2, &a);
    say("ConnectNamedPipe after client: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    nothing_more("after ERROR_PIPE_CONNECTED", port2);

    /* The peer closing: a pending read fails with ERROR_BROKEN_PIPE through the port. */
    memset(&a, 0, sizeof a);
    ok = ReadFile(srv2, buf, sizeof buf, &n, &a);
    say("server 2 ReadFile, empty: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    CloseHandle(cli2);
    dequeue("after client 2 closed", port2, &a, &b);

    CloseHandle(srv2);
    CloseHandle(port2);
    CloseHandle(cli);
    CloseHandle(srv);
    CloseHandle(port);
}

/* Child: the inherited client end, numbered on its command line as Chrome numbers it. */
static int child(HANDLE h)
{
    char buf[64];
    DWORD n;
    OVERLAPPED o = { 0 };
    HANDLE port = CreateIoCompletionPort(h, NULL, KEY_CLI, 1);
    if (!port)
        return 10;
    memset(buf, 0, sizeof buf);
    BOOL ok = ReadFile(h, buf, sizeof buf, &n, &o);
    if (!ok && GetLastError() != ERROR_IO_PENDING)
        return 11;
    DWORD bytes;
    ULONG_PTR key;
    OVERLAPPED *ov;
    if (!GetQueuedCompletionStatus(port, &bytes, &key, &ov, 10000))
        return 12;
    if (ov != &o || key != KEY_CLI || bytes != 4 || memcmp(buf, "ping", 4))
        return 13;
    memset(&o, 0, sizeof o);
    ok = WriteFile(h, "pong", 4, &n, &o);
    if (!ok && GetLastError() != ERROR_IO_PENDING)
        return 14;
    if (!GetQueuedCompletionStatus(port, &bytes, &key, &ov, 10000) || ov != &o || bytes != 4)
        return 15;
    return 42;
}

static void cross_process(const char *self)
{
    const WCHAR *name = L"\\\\.\\pipe\\iocpprobe.3";
    char buf[64], cmd[512];
    DWORD n;
    OVERLAPPED a = { 0 }, b = { 0 };
    HANDLE srv = make_server(name);
    HANDLE port = CreateIoCompletionPort(srv, NULL, KEY_SRV, 1);
    ConnectNamedPipe(srv, &a);
    HANDLE cli = open_client(name, TRUE);
    say("cross: pipe %s, client %s", srv != INVALID_HANDLE_VALUE ? "ok" : "FAIL",
        cli != INVALID_HANDLE_VALUE ? "ok" : "FAIL");
    dequeue("cross: connect", port, &a, &b);

    snprintf(cmd, sizeof cmd, "\"%s\" --child=%llu", self, (unsigned long long)(ULONG_PTR)cli);
    STARTUPINFOA si = { sizeof si };
    PROCESS_INFORMATION pi;
    if (!CreateProcessA(NULL, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi))
    {
        say("cross: CreateProcess FAIL %lu", GetLastError());
        return;
    }
    CloseHandle(cli);

    memset(&b, 0, sizeof b);
    BOOL ok = WriteFile(srv, "ping", 4, &n, &b);
    say("cross: WriteFile ping: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    dequeue("cross: ping completion", port, &a, &b);
    memset(&a, 0, sizeof a);
    memset(buf, 0, sizeof buf);
    ok = ReadFile(srv, buf, sizeof buf, &n, &a);
    say("cross: ReadFile: ret=%d err=%lu", ok, ok ? 0 : GetLastError());
    dequeue("cross: pong completion", port, &a, &b);
    say("cross: got %s", buf);
    DWORD w = WaitForSingleObject(pi.hProcess, 15000), code = 0;
    GetExitCodeProcess(pi.hProcess, &code);
    say("cross: child wait=%lu exit=%lu (42 = every step ok)", w, code);
    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    CloseHandle(srv);
    CloseHandle(port);
}

int main(int argc, char **argv)
{
    if (argc > 1 && !strncmp(argv[1], "--child=", 8))
        return child((HANDLE)(ULONG_PTR)_strtoui64(argv[1] + 8, NULL, 10));
    in_process();
    cross_process(argv[0]);
    say("done");
    return 0;
}
