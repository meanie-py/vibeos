/* sockprobe: a TCP connect the way Chrome's net/socket/tcp_socket_win.cc makes one, against a
 * listener in this process. Chrome's fetch of http://127.0.0.1:8000/ timed out with the server
 * never seeing a connection, and its network service logged "closesocket: not a socket".
 * Chrome's sequence: WSASocket(WSA_FLAG_OVERLAPPED | WSA_FLAG_NO_HANDLE_INHERIT), FIONBIO,
 * WSAEventSelect(FD_CONNECT), connect() -> WSAEWOULDBLOCK, wait on the event,
 * WSAEnumNetworkEvents; then FD_READ|FD_CLOSE for the response. One line per fact, to diff with
 * sockprobe.win11.txt. */
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <stdio.h>
#include <string.h>

#ifndef WSA_FLAG_NO_HANDLE_INHERIT
#define WSA_FLAG_NO_HANDLE_INHERIT 0x80
#endif
#define PORT 8123

static void say(const char *fmt, ...)
{
    char b[400];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("[SOCK] %s\n", b);
    fflush(stdout);
}

static volatile LONG g_accepted;

static DWORD WINAPI server(LPVOID arg)
{
    SOCKET s = (SOCKET)(ULONG_PTR)arg;
    for (;;) {
        SOCKET c = accept(s, NULL, NULL);
        if (c == INVALID_SOCKET)
            return 1;
        InterlockedIncrement(&g_accepted);
        char buf[512];
        int n = recv(c, buf, sizeof buf, 0);
        if (n > 0)
            send(c, "pong", 4, 0);
        shutdown(c, SD_SEND);
        closesocket(c);
    }
}

static struct sockaddr_in addr(unsigned long ip)
{
    struct sockaddr_in a = { 0 };
    a.sin_family = AF_INET;
    a.sin_port = htons(PORT);
    a.sin_addr.s_addr = ip;
    return a;
}

static void blocking(const char *tag, unsigned long ip)
{
    struct sockaddr_in a = addr(ip);
    SOCKET c = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    int r = connect(c, (struct sockaddr *)&a, sizeof a);
    say("%s blocking connect: %s (err %d)", tag, r ? "FAIL" : "ok", r ? WSAGetLastError() : 0);
    if (!r) {
        char buf[16] = { 0 };
        send(c, "ping", 4, 0);
        int n = recv(c, buf, sizeof buf - 1, 0);
        say("%s blocking reply: %d '%s'", tag, n, n > 0 ? buf : "");
    }
    say("%s closesocket: %d", tag, closesocket(c));
}

static void chrome(const char *tag, unsigned long ip)
{
    struct sockaddr_in a = addr(ip);
    WSANETWORKEVENTS ne;
    SOCKET c = WSASocketW(AF_INET, SOCK_STREAM, IPPROTO_TCP, NULL, 0,
                          WSA_FLAG_OVERLAPPED | WSA_FLAG_NO_HANDLE_INHERIT);
    if (c == INVALID_SOCKET) {
        say("%s WSASocket(OVERLAPPED|NO_HANDLE_INHERIT) FAIL %d", tag, WSAGetLastError());
        return;
    }
    u_long nb = 1;
    say("%s FIONBIO: %d", tag, ioctlsocket(c, FIONBIO, &nb));
    BOOL nd = TRUE;
    say("%s TCP_NODELAY: %d", tag, setsockopt(c, IPPROTO_TCP, TCP_NODELAY, (char *)&nd, sizeof nd));
    WSAEVENT ev = WSACreateEvent();
    say("%s WSAEventSelect(FD_CONNECT): %d", tag, WSAEventSelect(c, ev, FD_CONNECT));
    int r = connect(c, (struct sockaddr *)&a, sizeof a);
    int e = r ? WSAGetLastError() : 0;
    say("%s connect: %d err %d%s", tag, r, e, e == WSAEWOULDBLOCK ? " (WOULDBLOCK)" : "");
    DWORD w = WaitForSingleObject(ev, 5000);
    say("%s connect event: %s", tag, w == WAIT_OBJECT_0 ? "signaled" : w == WAIT_TIMEOUT ? "TIMEOUT" : "failed");
    memset(&ne, 0, sizeof ne);
    r = WSAEnumNetworkEvents(c, ev, &ne);
    say("%s WSAEnumNetworkEvents: %d events %#lx connect-err %d", tag, r, ne.lNetworkEvents,
        ne.iErrorCode[FD_CONNECT_BIT]);
    if (ne.lNetworkEvents & FD_CONNECT && !ne.iErrorCode[FD_CONNECT_BIT]) {
        say("%s WSAEventSelect(FD_READ|FD_CLOSE): %d", tag, WSAEventSelect(c, ev, FD_READ | FD_CLOSE));
        say("%s send: %d", tag, send(c, "ping", 4, 0));
        w = WaitForSingleObject(ev, 5000);
        memset(&ne, 0, sizeof ne);
        WSAEnumNetworkEvents(c, ev, &ne);
        char buf[16] = { 0 };
        int n = recv(c, buf, sizeof buf - 1, 0);
        say("%s read event %s events %#lx, recv %d '%s'", tag, w == WAIT_OBJECT_0 ? "signaled" : "TIMEOUT",
            ne.lNetworkEvents & FD_READ, n, n > 0 ? buf : "");
    }
    WSAEventSelect(c, NULL, 0);
    WSACloseEvent(ev);
    r = closesocket(c);
    say("%s closesocket: %d err %d", tag, r, r ? WSAGetLastError() : 0);
}

int main(void)
{
    WSADATA wd;
    WSAStartup(MAKEWORD(2, 2), &wd);
    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    struct sockaddr_in a = addr(htonl(INADDR_ANY));
    int r = bind(s, (struct sockaddr *)&a, sizeof a);
    say("listen on :%d: %s", PORT, r || listen(s, 16) ? "FAIL" : "ok");
    CloseHandle(CreateThread(NULL, 0, server, (LPVOID)(ULONG_PTR)s, 0, NULL));

    /* The machine's own (NIC) address, as a second target that is not loopback. */
    unsigned long nic = 0;
    char host[256];
    struct addrinfo hints = { 0 }, *ai = NULL;
    hints.ai_family = AF_INET;
    if (!gethostname(host, sizeof host) && !getaddrinfo(host, NULL, &hints, &ai) && ai) {
        nic = ((struct sockaddr_in *)ai->ai_addr)->sin_addr.s_addr;
        freeaddrinfo(ai);
    }
    say("own address: %s", nic && nic != htonl(INADDR_LOOPBACK) ? "found" : "none");

    blocking("loopback", htonl(INADDR_LOOPBACK));
    chrome("loopback", htonl(INADDR_LOOPBACK));
    if (nic && nic != htonl(INADDR_LOOPBACK)) {
        blocking("nic", nic);
        chrome("nic", nic);
    }
    Sleep(200);
    say("connections accepted: %ld", g_accepted);
    say("done");
    return 0;
}
