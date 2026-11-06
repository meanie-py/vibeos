/* dupprobe: what Mojo's broker does that single-process Chrome never does. Chrome dumps the page
 * with --single-process but hangs multi-process, every thread blocked; between processes Mojo
 * moves handles with DuplicateHandle (mojo/core/platform_handle_in_transit.cc): the browser
 * duplicates sections and pipe ends INTO a child, and takes a child's handles OUT of it with
 * DUPLICATE_CLOSE_SOURCE. To join two children (renderer <-> GPU) it makes an overlapped pipe
 * pair and duplicates one end into each. Each case is one line, to diff with dupprobe.win11.txt.
 *
 * The parent drives two copies of itself over their stdin/stdout ("<cmd> <args>" in, one result
 * line out), so the handle values each side prints are its own. */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void say(const char *fmt, ...)
{
    char b[512];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("[DUP] %s\n", b);
    fflush(stdout);
}

/* ---- child ---- */

static void reply(const char *fmt, ...)
{
    char b[400];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("%s\n", b);
    fflush(stdout);
}

static void child_pipe(HANDLE h, int server)
{
    char buf[16] = { 0 };
    DWORD n, bytes;
    ULONG_PTR key;
    OVERLAPPED o = { 0 }, *ov;
    HANDLE port = CreateIoCompletionPort(h, NULL, 7, 1);
    if (!port) {
        reply("pipe: port err %lu", GetLastError());
        return;
    }
    if (server) {
        BOOL ok = WriteFile(h, "hi", 2, &n, &o);
        if (!ok && GetLastError() != ERROR_IO_PENDING) {
            reply("pipe s: write err %lu", GetLastError());
            return;
        }
        if (!GetQueuedCompletionStatus(port, &bytes, &key, &ov, 10000)) {
            reply("pipe s: write completion err %lu", GetLastError());
            return;
        }
        memset(&o, 0, sizeof o);
        ok = ReadFile(h, buf, sizeof buf - 1, &n, &o);
        if (!ok && GetLastError() != ERROR_IO_PENDING) {
            reply("pipe s: read err %lu", GetLastError());
            return;
        }
        if (!GetQueuedCompletionStatus(port, &bytes, &key, &ov, 10000)) {
            reply("pipe s: read completion err %lu", GetLastError());
            return;
        }
        reply("pipe s: sent hi, got %.*s (%lu bytes)", (int)bytes, buf, bytes);
    } else {
        BOOL ok = ReadFile(h, buf, sizeof buf - 1, &n, &o);
        if (!ok && GetLastError() != ERROR_IO_PENDING) {
            reply("pipe c: read err %lu", GetLastError());
            return;
        }
        if (!GetQueuedCompletionStatus(port, &bytes, &key, &ov, 10000)) {
            reply("pipe c: read completion err %lu", GetLastError());
            return;
        }
        DWORD got = bytes;
        memset(&o, 0, sizeof o);
        ok = WriteFile(h, "yo", 2, &n, &o);
        if ((!ok && GetLastError() != ERROR_IO_PENDING) ||
            !GetQueuedCompletionStatus(port, &bytes, &key, &ov, 10000)) {
            reply("pipe c: write err %lu", GetLastError());
            return;
        }
        reply("pipe c: got %.*s (%lu bytes), sent yo", (int)got, buf, got);
    }
}

static int child(void)
{
    char line[256];
    while (fgets(line, sizeof line, stdin)) {
        char cmd[16] = { 0 };
        unsigned long long a = 0, b = 0;
        sscanf(line, "%15s %llu %llu", cmd, &a, &b);
        HANDLE h = (HANDLE)(ULONG_PTR)a;
        if (!strcmp(cmd, "map")) {
            char *v = MapViewOfFile(h, b ? FILE_MAP_WRITE : FILE_MAP_READ, 0, 0, 4096);
            if (!v) {
                reply("map %s: err %lu", b ? "rw" : "ro", GetLastError());
                continue;
            }
            char seen[16];
            memcpy(seen, v, 8);
            seen[8] = 0;
            if (b)
                memcpy(v, "child!!", 8);
            reply("map %s: ok, read \"%s\"%s", b ? "rw" : "ro", seen, b ? ", wrote child!!" : "");
            UnmapViewOfFile(v);
        } else if (!strcmp(cmd, "mkev")) {
            HANDLE e = CreateEventA(NULL, TRUE, FALSE, NULL);
            reply("%llu", (unsigned long long)(ULONG_PTR)e);
        } else if (!strcmp(cmd, "wait")) {
            DWORD w = WaitForSingleObject(h, 5000);
            reply("wait: %lu (err %lu)", w, w == WAIT_FAILED ? GetLastError() : 0);
        } else if (!strcmp(cmd, "pipe")) {
            child_pipe(h, (int)b);
        } else if (!strcmp(cmd, "quit")) {
            return 0;
        }
    }
    return 0;
}

/* ---- parent ---- */

typedef struct {
    PROCESS_INFORMATION pi;
    HANDLE in, out;
} Kid;

static int spawn(const char *self, Kid *k)
{
    SECURITY_ATTRIBUTES sa = { sizeof sa, NULL, TRUE };
    HANDLE in_r, in_w, out_r, out_w;
    char cmd[512];
    STARTUPINFOA si = { sizeof si };
    if (!CreatePipe(&in_r, &in_w, &sa, 0) || !CreatePipe(&out_r, &out_w, &sa, 0))
        return 0;
    SetHandleInformation(in_w, HANDLE_FLAG_INHERIT, 0);
    SetHandleInformation(out_r, HANDLE_FLAG_INHERIT, 0);
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdInput = in_r;
    si.hStdOutput = out_w;
    si.hStdError = GetStdHandle(STD_ERROR_HANDLE);
    snprintf(cmd, sizeof cmd, "\"%s\" --child", self);
    if (!CreateProcessA(NULL, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &k->pi))
        return 0;
    CloseHandle(in_r);
    CloseHandle(out_w);
    k->in = in_w;
    k->out = out_r;
    return 1;
}

static void tell(Kid *k, const char *fmt, ...)
{
    char b[128];
    DWORD n;
    va_list ap;
    va_start(ap, fmt);
    int len = vsnprintf(b, sizeof b - 1, fmt, ap);
    va_end(ap);
    b[len++] = '\n';
    WriteFile(k->in, b, (DWORD)len, &n, NULL);
}

static void recv_line(Kid *k, char *b, int cap)
{
    int i = 0;
    DWORD n;
    char c;
    while (i < cap - 1 && ReadFile(k->out, &c, 1, &n, NULL) && n == 1 && c != '\n')
        if (c != '\r')
            b[i++] = c;
    b[i] = 0;
}

static void ask(Kid *k, const char *label, char *b, int cap)
{
    recv_line(k, b, cap);
    say("%s: %s", label, b);
}

static HANDLE into(Kid *k, HANDLE h, DWORD access, DWORD opts)
{
    HANDLE r = NULL;
    BOOL ok = DuplicateHandle(GetCurrentProcess(), h, k->pi.hProcess, &r, access, FALSE, opts);
    if (!ok)
        say("  DuplicateHandle into child failed: %lu", GetLastError());
    return ok ? r : NULL;
}

static void parent(const char *self)
{
    char b[400];
    Kid x, y;
    if (!spawn(self, &x) || !spawn(self, &y)) {
        say("spawn failed: %lu", GetLastError());
        return;
    }

    /* A: a section duplicated into the child, read-write: the child sees our bytes and we see its. */
    HANDLE sec = CreateFileMappingA(INVALID_HANDLE_VALUE, NULL, PAGE_READWRITE, 0, 65536, NULL);
    char *v = MapViewOfFile(sec, FILE_MAP_ALL_ACCESS, 0, 0, 0);
    memcpy(v, "parent!", 8);
    HANDLE hx = into(&x, sec, FILE_MAP_READ | FILE_MAP_WRITE, 0);
    tell(&x, "map %llu 1", (unsigned long long)(ULONG_PTR)hx);
    ask(&x, "A section rw into child", b, sizeof b);
    say("A parent now reads \"%.8s\"", v);

    /* B: read-only into the child: its write mapping must be refused. */
    hx = into(&x, sec, FILE_MAP_READ, 0);
    tell(&x, "map %llu 1", (unsigned long long)(ULONG_PTR)hx);
    ask(&x, "B section ro into child, mapped for write", b, sizeof b);
    tell(&x, "map %llu 0", (unsigned long long)(ULONG_PTR)hx);
    ask(&x, "B same handle mapped for read", b, sizeof b);

    /* C: take the child's event out of it with DUPLICATE_CLOSE_SOURCE. Ours must work, the
     * child's must be gone, and none of OUR handles may be closed in its place. */
    HANDLE mine[32];
    for (int i = 0; i < 32; i++)
        mine[i] = CreateEventA(NULL, FALSE, FALSE, NULL);
    tell(&x, "mkev");
    recv_line(&x, b, sizeof b);
    unsigned long long remote = _strtoui64(b, NULL, 10);
    HANDLE got = NULL;
    BOOL ok = DuplicateHandle(x.pi.hProcess, (HANDLE)(ULONG_PTR)remote, GetCurrentProcess(), &got, 0, FALSE,
                              DUPLICATE_SAME_ACCESS | DUPLICATE_CLOSE_SOURCE);
    say("C DuplicateHandle out of child with CLOSE_SOURCE: ok=%d err=%lu", ok, ok ? 0 : GetLastError());
    say("C SetEvent on ours: %d", got ? SetEvent(got) : 0);
    tell(&x, "wait %llu", remote);
    ask(&x, "C child waits on its now-closed handle", b, sizeof b);
    int alive = 0;
    for (int i = 0; i < 32; i++)
        alive += SetEvent(mine[i]) != 0;
    say("C our own 32 handles still valid: %d", alive);

    /* D: an event made by us, duplicated into the child, signalled by us. */
    HANDLE ev = CreateEventA(NULL, TRUE, FALSE, NULL);
    hx = into(&x, ev, 0, DUPLICATE_SAME_ACCESS);
    SetEvent(ev);
    tell(&x, "wait %llu", (unsigned long long)(ULONG_PTR)hx);
    ask(&x, "D child waits on our signalled event", b, sizeof b);

    /* E: Mojo's introduction. An overlapped pipe pair made here, one end duplicated into each
     * child and closed here (DUPLICATE_CLOSE_SOURCE), then the children talk directly. */
    HANDLE ps = CreateNamedPipeA("\\\\.\\pipe\\dupprobe.e",
                                 PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
                                 PIPE_TYPE_BYTE | PIPE_READMODE_BYTE, 1, 4096, 4096, 5000, NULL);
    HANDLE pc = CreateFileA("\\\\.\\pipe\\dupprobe.e", GENERIC_READ | GENERIC_WRITE, 0, NULL, OPEN_EXISTING,
                            SECURITY_SQOS_PRESENT | SECURITY_ANONYMOUS | FILE_FLAG_OVERLAPPED, NULL);
    say("E pipe pair: server %s, client %s", ps != INVALID_HANDLE_VALUE ? "ok" : "FAIL",
        pc != INVALID_HANDLE_VALUE ? "ok" : "FAIL");
    HANDLE sx = into(&x, ps, 0, DUPLICATE_SAME_ACCESS | DUPLICATE_CLOSE_SOURCE);
    HANDLE cy = into(&y, pc, 0, DUPLICATE_SAME_ACCESS | DUPLICATE_CLOSE_SOURCE);
    tell(&y, "pipe %llu 0", (unsigned long long)(ULONG_PTR)cy);
    tell(&x, "pipe %llu 1", (unsigned long long)(ULONG_PTR)sx);
    ask(&x, "E child X (server end)", b, sizeof b);
    ask(&y, "E child Y (client end)", b, sizeof b);

    tell(&x, "quit");
    tell(&y, "quit");
    DWORD w1 = WaitForSingleObject(x.pi.hProcess, 10000), w2 = WaitForSingleObject(y.pi.hProcess, 10000);
    say("children exited: %lu %lu", w1, w2);
}

int main(int argc, char **argv)
{
    if (argc > 1 && !strcmp(argv[1], "--child"))
        return child();
    parent(argv[0]);
    say("done");
    return 0;
}
