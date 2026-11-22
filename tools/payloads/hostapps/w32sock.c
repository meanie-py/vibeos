/* w32sock.c: the steps libassuan takes to create gpg-agent's socket on Windows, one line
 * each, to diff against winref. gpg-agent fails here with "error getting nonce for the
 * socket" then "error binding socket ... Invalid argument" (hostapps 2026-11-21). On Windows an
 * AF_UNIX assuan socket is a TCP listener on 127.0.0.1 whose port and a 16-byte nonce are
 * written to the socket file; the nonce comes from CryptGenRandom on a PROV_RSA_FULL
 * verify context. Usage: w32sock.exe <file to write, as assuan does>
 */
#include <winsock2.h>
#include <windows.h>
#include <wincrypt.h>
#include <stdio.h>

int wmain(int argc, wchar_t **argv)
{
    HCRYPTPROV prov = 0;
    BYTE nonce[16] = {0};
    WSADATA wsa;
    SOCKET s;
    struct sockaddr_in a = {0};
    int alen = sizeof(a), r;

    SetLastError(0);
    if (CryptAcquireContextW(&prov, NULL, NULL, PROV_RSA_FULL, CRYPT_VERIFYCONTEXT | CRYPT_SILENT)) {
        printf("CryptAcquireContext ok\n");
        SetLastError(0);
        if (CryptGenRandom(prov, sizeof(nonce), nonce))
            printf("CryptGenRandom ok\n");
        else
            printf("CryptGenRandom err=%08lx\n", GetLastError());
        CryptReleaseContext(prov, 0);
    } else
        printf("CryptAcquireContext err=%08lx\n", GetLastError());

    r = WSAStartup(MAKEWORD(2, 2), &wsa);
    printf("WSAStartup r=%d version=%04x\n", r, wsa.wVersion);
    s = socket(AF_INET, SOCK_STREAM, 0);
    if (s == INVALID_SOCKET) {
        printf("socket err=%d\n", WSAGetLastError());
        return 1;
    }
    printf("socket ok\n");
    a.sin_family = AF_INET;
    a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    a.sin_port = 0;
    r = bind(s, (struct sockaddr *)&a, sizeof(a));
    printf("bind r=%d err=%d\n", r, r ? WSAGetLastError() : 0);
    memset(&a, 0, sizeof(a));
    r = getsockname(s, (struct sockaddr *)&a, &alen);
    printf("getsockname r=%d err=%d family=%d len=%d port_nonzero=%d addr=%08lx\n", r,
           r ? WSAGetLastError() : 0, a.sin_family, alen, a.sin_port != 0,
           (unsigned long)ntohl(a.sin_addr.s_addr));
    r = listen(s, 5);
    printf("listen r=%d err=%d\n", r, r ? WSAGetLastError() : 0);
    if (argc > 1) {
        HANDLE f = CreateFileW(argv[1], GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
        if (f == INVALID_HANDLE_VALUE)
            printf("socket file err=%lu\n", GetLastError());
        else {
            char buf[32];
            DWORD n = 0;
            int len = sprintf(buf, "%u\n", ntohs(a.sin_port));
            BOOL ok = WriteFile(f, buf, len, &n, NULL) && WriteFile(f, nonce, sizeof(nonce), &n, NULL);
            printf("socket file write ok=%d\n", ok);
            CloseHandle(f);
            DeleteFileW(argv[1]);
        }
    }
    closesocket(s);
    WSACleanup();
    printf("w32sock done\n");
    return 0;
}
