/* servepage: a two-resource HTTP server, then a command run against it.
 * Usage: servepage <port> <command line...>
 *
 * Rung 2 of the Chrome ladder: the page comes over a socket instead of file://, so Chrome's
 * network service (its own process, winsock through AFD) has to fetch it. The page pulls its
 * script from a second URL, so "net js ran: 42" in the dump means two requests were served AND
 * V8 ran what came back. The command inherits our stdout, so Chrome's --dump-dom lands in the
 * sweep's pipe; we exit with its exit code. Listens on every address, so the command may use
 * 127.0.0.1 or the guest's NIC address. One line per request served. */
#include <winsock2.h>
#include <windows.h>
#include <stdio.h>
#include <string.h>

static const char PAGE[] =
    "<!DOCTYPE html>\n<html><head><title>vibeos net</title></head>\n"
    "<body><p id=\"m\">hello over http</p>\n"
    "<script src=\"/app.js\"></script>\n</body></html>\n";
static const char SCRIPT[] =
    "document.getElementById('m').textContent += ', net js ran: ' + (6 * 7);\n";

static void say(const char *fmt, ...)
{
    char b[400];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("[SERVE] %s\n", b);
    fflush(stdout);
}

static void send_all(SOCKET c, const char *p, int n)
{
    while (n > 0) {
        int k = send(c, p, n, 0);
        if (k <= 0)
            return;
        p += k;
        n -= k;
    }
}

static DWORD WINAPI conn(LPVOID arg)
{
    SOCKET c = (SOCKET)(ULONG_PTR)arg;
    char req[4096], path[256] = "?", hdr[256];
    int got = 0, last = 0, err = 0;
    while (got < (int)sizeof req - 1) {
        int k = recv(c, req + got, sizeof req - 1 - got, 0);
        last = k;
        if (k <= 0) {
            err = k ? WSAGetLastError() : 0;
            break;
        }
        got += k;
        req[got] = 0;
        if (strstr(req, "\r\n\r\n"))
            break;
    }
    req[got] = 0;
    sscanf(req, "GET %255s", path);
    const char *body = NULL, *type = NULL;
    if (!strcmp(path, "/")) {
        body = PAGE;
        type = "text/html";
    } else if (!strcmp(path, "/app.js")) {
        body = SCRIPT;
        type = "text/javascript";
    }
    if (body) {
        int n = (int)strlen(body);
        int h = snprintf(hdr, sizeof hdr,
                         "HTTP/1.1 200 OK\r\nContent-Type: %s\r\nContent-Length: %d\r\n"
                         "Cache-Control: no-store\r\nConnection: close\r\n\r\n", type, n);
        send_all(c, hdr, h);
        send_all(c, body, n);
    } else {
        static const char nf[] =
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        send_all(c, nf, sizeof nf - 1);
    }
    if (!strcmp(path, "?"))
        say("no request: %d bytes, then recv %d (error %d)", got, last, err);
    say("GET %s -> %d", path, body ? 200 : 404);
    shutdown(c, SD_SEND);
    closesocket(c);
    return 0;
}

static DWORD WINAPI server(LPVOID arg)
{
    SOCKET s = (SOCKET)(ULONG_PTR)arg;
    for (;;) {
        SOCKET c = accept(s, NULL, NULL);
        if (c == INVALID_SOCKET) {
            say("accept failed %d", WSAGetLastError());
            return 1;
        }
        HANDLE t = CreateThread(NULL, 0, conn, (LPVOID)(ULONG_PTR)c, 0, NULL);
        if (t)
            CloseHandle(t);
        else
            closesocket(c);
    }
}

int main(int argc, char **argv)
{
    WSADATA wd;
    struct sockaddr_in a = { 0 };
    if (argc < 3) {
        say("usage: servepage <port> <command line>");
        return 2;
    }
    if (WSAStartup(MAKEWORD(2, 2), &wd)) {
        say("WSAStartup failed");
        return 1;
    }
    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    a.sin_family = AF_INET;
    a.sin_port = htons((u_short)atoi(argv[1]));
    a.sin_addr.s_addr = htonl(INADDR_ANY);
    if (s == INVALID_SOCKET || bind(s, (struct sockaddr *)&a, sizeof a) || listen(s, 16)) {
        say("listen on :%s failed %d", argv[1], WSAGetLastError());
        return 1;
    }
    say("listening on :%s", argv[1]);
    CloseHandle(CreateThread(NULL, 0, server, (LPVOID)(ULONG_PTR)s, 0, NULL));

    /* The command is everything after the port, taken verbatim from our own command line. */
    char *cmd = GetCommandLineA();
    for (int field = 0; field < 2 && *cmd; field++) {
        if (*cmd == '"') {
            cmd++;
            while (*cmd && *cmd != '"')
                cmd++;
            if (*cmd)
                cmd++;
        } else {
            while (*cmd && *cmd != ' ' && *cmd != '\t')
                cmd++;
        }
        while (*cmd == ' ' || *cmd == '\t')
            cmd++;
    }
    STARTUPINFOA si = { sizeof si };
    PROCESS_INFORMATION pi;
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
    si.hStdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
    si.hStdError = GetStdHandle(STD_ERROR_HANDLE);
    if (!CreateProcessA(NULL, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi)) {
        say("CreateProcess failed %lu: %s", GetLastError(), cmd);
        return 1;
    }
    WaitForSingleObject(pi.hProcess, INFINITE);
    DWORD code = 1;
    GetExitCodeProcess(pi.hProcess, &code);
    say("command exited %lu", code);
    return (int)code;
}
