/* vibehttpd.c — tiny threaded HTTP server for vibeos netstack stress gates.
 *
 * Part of the httpd payload (tools/payloads/httpd/payload.yaml; design:
 * docs/engineering/payload-harness-design.md). Deliberately NOT a foreign
 * binary: it uses exactly the winsock surface vibeos has proven (blocking
 * sockets, thread-per-connection) so the payload gate measures the KERNEL
 * under load — afd handle churn, IRP cancellation, nonpaged pool, accept
 * backlog — not unimplemented winsock features. (Foreign-binary compat
 * pressure is PuTTY's job.)
 *
 * Endpoints:
 *   GET /            -> 200, fixed banner body
 *   GET /data?n=SIZE -> 200, SIZE bytes of a deterministic pattern (host
 *                       hammer checksums it; SIZE capped at 1 MiB)
 *   GET /status      -> 200, "requests=N threads=N" counters (the hammer's
 *                       end-of-run consistency oracle)
 *   anything else    -> 404
 *
 * One connection per request (Connection: close) — maximum socket churn per
 * unit of work, which is the point.
 *
 * Serial markers (OutputDebugStringA -> debug serial; the harness greps these):
 *   [HTTPD] listening on :8080
 *   [HTTPD] FATAL: <step> gle=N     (any setup failure — fail loud)
 */

#include <winsock2.h>
#include <windows.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#define PORT 8080
#define MAX_DATA (1024 * 1024)

static volatile LONG g_requests = 0;
static volatile LONG g_threads = 0;

static void fatal(const char *step)
{
    char buf[128];
    _snprintf(buf, sizeof(buf), "[HTTPD] FATAL: %s gle=%lu\n", step,
              (unsigned long)WSAGetLastError());
    buf[sizeof(buf) - 1] = 0;
    OutputDebugStringA(buf);
    ExitProcess(1);
}

static int send_all(SOCKET s, const char *p, int len)
{
    while (len > 0) {
        int n = send(s, p, len, 0);
        if (n <= 0)
            return -1;
        p += n;
        len -= n;
    }
    return 0;
}

static void send_response(SOCKET s, const char *status, const char *body, int body_len)
{
    char hdr[256];
    int n = _snprintf(hdr, sizeof(hdr),
                      "HTTP/1.0 %s\r\n"
                      "Content-Type: text/plain\r\n"
                      "Content-Length: %d\r\n"
                      "Connection: close\r\n"
                      "\r\n",
                      status, body_len);
    if (n < 0 || send_all(s, hdr, n) != 0)
        return;
    send_all(s, body, body_len);
}

static DWORD WINAPI handle_conn(LPVOID arg)
{
    SOCKET s = (SOCKET)(ULONG_PTR)arg;
    char req[2048];
    int got = 0;

    InterlockedIncrement(&g_threads);

    /* Read until the header terminator (or the buffer fills). Requests from
     * the hammer are single-packet GETs, but don't assume it. */
    while (got < (int)sizeof(req) - 1) {
        int n = recv(s, req + got, (int)sizeof(req) - 1 - got, 0);
        if (n <= 0)
            goto done;
        got += n;
        req[got] = 0;
        if (strstr(req, "\r\n\r\n"))
            break;
    }

    if (strncmp(req, "GET /status", 11) == 0) {
        char body[96];
        int n = _snprintf(body, sizeof(body), "requests=%ld threads=%ld",
                          (long)g_requests, (long)g_threads);
        InterlockedIncrement(&g_requests);
        send_response(s, "200 OK", body, n);
    } else if (strncmp(req, "GET /data?n=", 12) == 0) {
        long want = strtol(req + 12, NULL, 10);
        if (want < 0)
            want = 0;
        if (want > MAX_DATA)
            want = MAX_DATA;
        char *body = (char *)malloc(want ? want : 1);
        if (body) {
            /* Deterministic pattern: byte i = (i * 7 + 13) & 0xff — the host
             * hammer recomputes and compares. */
            long i;
            for (i = 0; i < want; i++)
                body[i] = (char)((i * 7 + 13) & 0xff);
            InterlockedIncrement(&g_requests);
            send_response(s, "200 OK", body, (int)want);
            free(body);
        } else {
            send_response(s, "500 Internal Server Error", "oom", 3);
        }
    } else if (strncmp(req, "GET / ", 6) == 0 || strncmp(req, "GET /\r", 6) == 0) {
        static const char banner[] = "vibehttpd on vibeos\n";
        InterlockedIncrement(&g_requests);
        send_response(s, "200 OK", banner, (int)sizeof(banner) - 1);
    } else {
        send_response(s, "404 Not Found", "nope\n", 5);
    }

done:
    /* Graceful-ish close: shutdown send side, drain briefly, close. Keeps
     * TIME_WAIT churn on the guest side — deliberate stress. */
    shutdown(s, SD_SEND);
    closesocket(s);
    InterlockedDecrement(&g_threads);
    return 0;
}

/* Entry/link recipe cloned from tools/doom/build.sh (learned there the hard
 * way, relearned here 2026-07-07 — see the build.sh guards):
 *   - GUI subsystem: a console app spawned from explorer's startup walk hangs
 *     in kernel32 PROCESS_ATTACH allocating a console from CSRSS.
 *   - classic msvcrt.dll via -lmsvcrt-os + -Wl,-e,mainCRTStartup: the UCRT
 *     llvm-mingw default imports api-ms-win-crt-* api-sets, WHICH REACTOS
 *     DOES NOT SHIP — the child then dies/hangs in the loader before main,
 *     with zero serial evidence (observed live: kernel32 attach OK, then
 *     silence; a hidden missing-component popup under runCmd's INFINITE
 *     wait). build.sh guards both properties post-link. */
int main(void)
{
    OutputDebugStringA("[HTTPD] main entered\n");
    WSADATA wsa;
    SOCKET lsock;
    struct sockaddr_in addr;

    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0)
        fatal("WSAStartup");
    OutputDebugStringA("[HTTPD] WSAStartup OK\n");

    lsock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (lsock == INVALID_SOCKET)
        fatal("socket");

    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(PORT);
    if (bind(lsock, (struct sockaddr *)&addr, sizeof(addr)) == SOCKET_ERROR)
        fatal("bind");
    if (listen(lsock, 64) == SOCKET_ERROR)
        fatal("listen");

    OutputDebugStringA("[HTTPD] listening on :8080\n");

    for (;;) {
        SOCKET c = accept(lsock, NULL, NULL);
        if (c == INVALID_SOCKET) {
            /* Transient accept failures under load are survivable; a broken
             * listener is not. WSAECONNRESET on a queued-then-aborted
             * connection is the transient case. */
            if (WSAGetLastError() == WSAECONNRESET)
                continue;
            fatal("accept");
        }
        HANDLE h = CreateThread(NULL, 0, handle_conn, (LPVOID)(ULONG_PTR)c, 0, NULL);
        if (h == NULL) {
            /* Thread exhaustion: shed load, keep serving. */
            closesocket(c);
            continue;
        }
        CloseHandle(h);
    }
}
