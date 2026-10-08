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
 *   TCP: 10.0.2.100:9997  (guestfwd cmd:cat line echo — OUR port, see below)
 *   UDP: 10.0.2.3:53      (slirp's DNS forwarder)
 */

#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <stdio.h>
#include <string.h>

/* This test owns guestfwd port 9997 exclusively — see tests/qemu/vm.py
 * GUESTFWD_ECHO_PORTS. Two consumers on one guestfwd is not a sharing win: slirp
 * delivers the echo to the FIRST socket matching (addr, port), so the other one
 * waits forever on a reply that was handed to somebody else. */
#define TCP_ECHO_IP   "10.0.2.100"
#define TCP_ECHO_PORT 9997
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
            int err = WSAGetLastError();
            if (attempt == 0)
                markf("[NETTEST] TCP connect() failed, WSAGetLastError=%d\n", err);
            /* The retries are for a host listener that is not up YET, which answers
             * with RST at once. A timeout means nothing answers at all: there is no
             * guestfwd on this boot. Stop, because every attempt holds a SYN-SENT
             * that re-ARPs an address nobody answers, and smoltcp rate-limits ARP
             * for the whole interface -- 40 of them kept the DNS server from ever
             * being resolved, for every program on the system. */
            if (err == WSAETIMEDOUT)
            {
                closesocket(s);
                mark("[NETTEST] usermode winsock TCP: no echo host (connect timed out)\n");
                return 0;
            }
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

/* Skip one DNS name (label sequence or compression pointer); return next offset. */
static int dns_skip_name(const unsigned char *buf, int len, int off)
{
    while (off >= 0 && off < len)
    {
        unsigned char b = buf[off];
        if ((b & 0xC0) == 0xC0) return off + 2; /* compression pointer */
        if (b == 0) return off + 1;             /* root label */
        off += 1 + b;                           /* normal label */
    }
    return -1;
}

/* Pull the first A record out of a DNS response. Returns 1 + fills ip[4]. */
static int dns_get_a(const unsigned char *buf, int len, unsigned char ip[4])
{
    int ancount, off, i;
    if (len < 12) return 0;
    ancount = (buf[6] << 8) | buf[7];
    off = dns_skip_name(buf, len, 12); /* question name */
    if (off < 0 || off + 4 > len) return 0;
    off += 4; /* QTYPE + QCLASS */
    for (i = 0; i < ancount; i++)
    {
        int type, rdlen;
        off = dns_skip_name(buf, len, off);
        if (off < 0 || off + 10 > len) return 0;
        type  = (buf[off] << 8) | buf[off + 1];
        rdlen = (buf[off + 8] << 8) | buf[off + 9];
        off += 10;
        if (off + rdlen > len) return 0;
        if (type == 1 && rdlen == 4)
        {
            ip[0] = buf[off]; ip[1] = buf[off + 1];
            ip[2] = buf[off + 2]; ip[3] = buf[off + 3];
            return 1;
        }
        off += rdlen;
    }
    return 0;
}

/* The real deal: resolve example.com through slirp's DNS forwarder (real
 * recursive DNS), then open a real TCP connection to it on port 80 and pull an
 * actual HTTP response back from the public internet — every byte travelling
 * ws2_32 -> msafd -> AFD -> netstack -> smoltcp -> slirp NAT -> the internet. */
static int test_internet(void)
{
    SOCKET s;
    struct sockaddr_in sa, from;
    int fromlen, n;
    unsigned char rx[1500];
    unsigned char ip[4];
    char buf[256];
    const char *req = "GET / HTTP/1.0\r\nHost: example.com\r\nConnection: close\r\n\r\n";
    DWORD tmo = 5000;

    /* 1. Resolve example.com (UDP DNS to the slirp forwarder at 10.0.2.3). */
    s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (s == INVALID_SOCKET)
    {
        markf("[NETTEST] INET dns socket() failed %d\n", WSAGetLastError());
        return 0;
    }
    memset(&sa, 0, sizeof(sa));
    sa.sin_family = AF_INET;
    sa.sin_port = htons(DNS_PORT);
    sa.sin_addr.s_addr = inet_addr(DNS_IP);
    if (sendto(s, (const char *)DNS_QUERY, sizeof(DNS_QUERY), 0,
               (struct sockaddr *)&sa, sizeof(sa)) <= 0)
    {
        markf("[NETTEST] INET dns sendto failed %d\n", WSAGetLastError());
        closesocket(s);
        return 0;
    }
    setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, (const char *)&tmo, sizeof(tmo));
    fromlen = sizeof(from);
    n = recvfrom(s, (char *)rx, sizeof(rx), 0, (struct sockaddr *)&from, &fromlen);
    closesocket(s);
    if (n <= 0 || !dns_get_a(rx, n, ip))
    {
        mark("[NETTEST] INET could not resolve example.com\n");
        return 0;
    }
    sprintf(buf, "[NETTEST] INET resolved example.com -> %d.%d.%d.%d\n",
            ip[0], ip[1], ip[2], ip[3]);
    mark(buf);

    /* 2. Real TCP connect to the resolved public host + HTTP GET. */
    s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (s == INVALID_SOCKET)
    {
        markf("[NETTEST] INET tcp socket() failed %d\n", WSAGetLastError());
        return 0;
    }
    memset(&sa, 0, sizeof(sa));
    sa.sin_family = AF_INET;
    sa.sin_port = htons(80);
    memcpy(&sa.sin_addr, ip, 4);
    if (connect(s, (struct sockaddr *)&sa, sizeof(sa)) != 0)
    {
        markf("[NETTEST] INET connect to internet failed %d\n", WSAGetLastError());
        closesocket(s);
        return 0;
    }
    mark("[NETTEST] INET TCP connected to the public internet :80\n");
    if (send(s, req, (int)strlen(req), 0) <= 0)
    {
        markf("[NETTEST] INET send failed %d\n", WSAGetLastError());
        closesocket(s);
        return 0;
    }
    n = recv(s, buf, sizeof(buf) - 1, 0);
    closesocket(s);
    if (n <= 0)
    {
        mark("[NETTEST] INET no HTTP response\n");
        return 0;
    }
    buf[n] = 0;
    if (buf[0] == 'H' && buf[1] == 'T' && buf[2] == 'T' && buf[3] == 'P')
    {
        char line[96];
        int i;
        for (i = 0; i < n && i < 95 && buf[i] != '\r' && buf[i] != '\n'; i++)
            line[i] = buf[i];
        line[i] = 0;
        markf("[NETTEST] REAL INTERNET HTTP OK (%d bytes), status: ", n);
        mark(line);
        mark("\n");
        return 1;
    }
    mark("[NETTEST] INET got bytes but not an HTTP response\n");
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
    test_internet();

    mark("[NETTEST] usermode winsock proof done\n");
    WSACleanup();
    return 0;
}
