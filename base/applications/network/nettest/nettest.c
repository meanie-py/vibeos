/*
 * nettest.exe — usermode winsock proof for the smoltcp netstack (netstack.sys).
 *
 * The kernel TDI self-tests (netstack-rust) drive the AFD-facing TDI surface
 * directly via IRPs. This program closes the loop from the OTHER end: a REAL
 * usermode winsock (ws2_32 -> msafd -> AFD -> \Device\Tcp|Udp -> netstack)
 * round-trip. It is autostarted from the HKCU..\Run key on the LiveCD and emits
 * its results to the kernel debug serial port via OutputDebugStringA (which,
 * with no usermode debugger attached, calls ntdll!DbgPrint -> COM1), so the
 * markers land in the same serial stream tests/qemu/test_net.py reads.
 *
 * Targets the test harness's slirp services (tests/qemu/vm.py):
 *   TCP: 10.0.2.100:9999  (guestfwd cmd:cat line echo)
 *   UDP: 10.0.2.3:53      (slirp's DNS forwarder)
 */

#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <stdio.h>

#define TCP_ECHO_IP   "10.0.2.100"
#define TCP_ECHO_PORT 9999
#define DNS_IP        "10.0.2.3"
#define DNS_PORT      53

static void mark(const char *msg)
{
    OutputDebugStringA(msg);
}

static void markf(const char *fmt, int a)
{
    char buf[160];
    sprintf(buf, fmt, a);
    OutputDebugStringA(buf);
}

/* A static DNS query for "example.com" A (same wire bytes the kernel test uses). */
static const unsigned char DNS_QUERY[] = {
    0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    7, 'e','x','a','m','p','l','e', 3, 'c','o','m', 0,
    0x00, 0x01, 0x00, 0x01
};

/* Real usermode TCP round-trip: connect to the slirp echo, send, recv. */
static int test_tcp(void)
{
    SOCKET s;
    struct sockaddr_in sa;
    const char *probe = "VIBEOS-USERMODE-WINSOCK\n";
    char rx[128];
    int n, attempt;

    for (attempt = 0; attempt < 40; attempt++)
    {
        s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        if (s == INVALID_SOCKET)
        {
            /* A failed socket() (e.g. WSAEINVAL 10022 = no provider in the empty
             * Winsock2 catalog) is permanent — don't retry, just report + exit. */
            markf("[NETTEST] TCP socket() failed, WSAGetLastError=%d\n", WSAGetLastError());
            return 0;
        }
        memset(&sa, 0, sizeof(sa));
        sa.sin_family = AF_INET;
        sa.sin_port = htons(TCP_ECHO_PORT);
        sa.sin_addr.s_addr = inet_addr(TCP_ECHO_IP);

        if (connect(s, (struct sockaddr *)&sa, sizeof(sa)) != 0)
        {
            if (attempt == 0)
                markf("[NETTEST] TCP connect() failed, WSAGetLastError=%d\n", WSAGetLastError());
        }
        else
        {
            if (send(s, probe, (int)strlen(probe), 0) > 0)
            {
                n = recv(s, rx, sizeof(rx), 0);
                if (n > 0)
                {
                    closesocket(s);
                    markf("[NETTEST] usermode winsock TCP echo OK (%d bytes via ws2_32->AFD->netstack)\n", n);
                    return 1;
                }
            }
        }
        closesocket(s);
        Sleep(1000);
    }
    mark("[NETTEST] usermode winsock TCP FAILED\n");
    return 0;
}

/* Real usermode UDP round-trip: a DNS query to slirp's forwarder, recvfrom. */
static int test_udp(void)
{
    SOCKET s;
    struct sockaddr_in sa, from;
    int fromlen, n, attempt;
    char rx[512];

    for (attempt = 0; attempt < 30; attempt++)
    {
        s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
        if (s == INVALID_SOCKET)
        {
            /* Permanent failure (no provider) — report + exit, don't spin. */
            markf("[NETTEST] UDP socket() failed, WSAGetLastError=%d\n", WSAGetLastError());
            return 0;
        }
        memset(&sa, 0, sizeof(sa));
        sa.sin_family = AF_INET;
        sa.sin_port = htons(DNS_PORT);
        sa.sin_addr.s_addr = inet_addr(DNS_IP);

        if (sendto(s, (const char *)DNS_QUERY, sizeof(DNS_QUERY), 0,
                   (struct sockaddr *)&sa, sizeof(sa)) > 0)
        {
            /* Bounded receive wait so we don't block forever on a drop. */
            DWORD tmo = 3000;
            setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, (const char *)&tmo, sizeof(tmo));
            fromlen = sizeof(from);
            n = recvfrom(s, rx, sizeof(rx), 0, (struct sockaddr *)&from, &fromlen);
            if (n > 0)
            {
                closesocket(s);
                markf("[NETTEST] usermode winsock UDP DNS OK (%d bytes via ws2_32->AFD->netstack)\n", n);
                return 1;
            }
        }
        closesocket(s);
        Sleep(1000);
    }
    mark("[NETTEST] usermode winsock UDP FAILED\n");
    return 0;
}

int main(void)
{
    WSADATA wsd;

    mark("[NETTEST] usermode winsock proof starting\n");
    if (WSAStartup(MAKEWORD(2, 2), &wsd) != 0)
    {
        markf("[NETTEST] WSAStartup FAILED, error=%d\n", WSAGetLastError());
        return 1;
    }
    mark("[NETTEST] WSAStartup OK\n");

    test_tcp();
    test_udp();

    mark("[NETTEST] usermode winsock proof done\n");
    WSACleanup();
    return 0;
}
