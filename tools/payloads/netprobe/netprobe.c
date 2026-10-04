/* netprobe — the client side of the network, one layer at a time, each step's exact error
 * printed (rapps' "Updating Database" failed on arm64 before any socket reached netstack):
 *   0. wait for the DNS Client service: dnsapi asks it over RPC, and before it runs that RPC
 *      raises RPC_S_SERVER_UNAVAILABLE (fatal on arm64, where the Rust clients dropped the C's
 *      RpcTryExcept) -- a startup-folder program gets there first;
 *   1. gethostbyname and getaddrinfo of the rapps database host (dnsapi -> UDP 53);
 *   2. a raw winsock TCP connect to port 80 of the address that came back;
 *   3. wininet InternetOpenUrlW over http, then over https (schannel), reading the body.
 * Verdict: "[NETPROBE] done VERDICT=PASS|FAIL" — PASS needs every step. */
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <wininet.h>
#include <string.h>
#include <windns.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[NETPROBE] " __VA_ARGS__)

static LONG failures;
#define CHECK(cond, ...) do { if (!(cond)) { SAY("FAIL " __VA_ARGS__); failures++; } } while (0)

static const char HOST[] = "rapps.reactos.org";

static void wait_dnscache(void)
{
    SERVICE_STATUS st;
    DWORD err = 0;
    int i;

    SAY("step wait-dnscache\n");
    for (i = 0; i < 240; i++) {
        SC_HANDLE scm = OpenSCManagerW(NULL, NULL, SC_MANAGER_CONNECT), svc;
        BOOL up = FALSE;
        svc = scm ? OpenServiceW(scm, L"Dnscache", SERVICE_QUERY_STATUS) : NULL;
        if (!svc) err = GetLastError();
        else if (QueryServiceStatus(svc, &st)) up = st.dwCurrentState == SERVICE_RUNNING;
        if (svc) CloseServiceHandle(svc);
        if (scm) CloseServiceHandle(scm);
        if (up) { SAY("Dnscache running after %ds\n", i); return; }
        Sleep(1000);
    }
    SAY("FAIL Dnscache not running after 240s (last OpenService err=%lu)\n", err);
    failures++;
}

/* A DNS query for HOST built by hand and sent over a plain UDP socket: separates
 * "no reply reaches a socket" (netstack/slirp) from dnsapi/dnsrslvr's own logic. */
static void step_raw_udp(const char *server)
{
    unsigned char q[64], r[512];
    int n = 0, got, tmo = 3000;
    const char *p = HOST, *dot;
    struct sockaddr_in to, from;
    int fromlen = sizeof from;
    SOCKET s;

    SAY("step raw-udp %s\n", server);
    static const unsigned char hdr[12] = { 0x12, 0x34, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0 };
    memcpy(q, hdr, 12); n = 12;
    for (;;) {
        dot = strchr(p, '.');
        int len = dot ? (int)(dot - p) : (int)strlen(p);
        q[n++] = (unsigned char)len; memcpy(q + n, p, len); n += len;
        if (!dot) break;
        p = dot + 1;
    }
    q[n++] = 0; q[n++] = 0; q[n++] = 1; q[n++] = 0; q[n++] = 1; /* A, IN */

    s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    CHECK(s != INVALID_SOCKET, "udp socket wsa=%d\n", WSAGetLastError());
    if (s == INVALID_SOCKET) return;
    setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, (const char *)&tmo, sizeof tmo);
    ZeroMemory(&to, sizeof to);
    to.sin_family = AF_INET;
    to.sin_port = htons(53);
    to.sin_addr.s_addr = inet_addr(server);
    CHECK(sendto(s, (const char *)q, n, 0, (struct sockaddr *)&to, sizeof to) == n,
          "sendto %s wsa=%d\n", server, WSAGetLastError());
    {
        /* select, not SO_RCVTIMEO: the receive timeout is not honoured (a recvfrom with
         * a 3s SO_RCVTIMEO blocked for 9 minutes), which is its own finding. */
        fd_set rd;
        struct timeval tv = { 3, 0 };
        FD_ZERO(&rd);
        FD_SET(s, &rd);
        got = select(0, &rd, NULL, NULL, &tv);
        SAY("raw-udp %s: select=%d\n", server, got);
        got = got > 0 ? recvfrom(s, (char *)r, sizeof r, 0, (struct sockaddr *)&from, &fromlen) : -1;
    }
    if (got > 0)
        SAY("raw-udp %s: %d-byte reply from %s, rcode=%d answers=%d\n", server, got,
            inet_ntoa(from.sin_addr), r[3] & 15, (r[6] << 8) | r[7]);
    else
        CHECK(0, "raw-udp %s: recvfrom=%d wsa=%d\n", server, got, WSAGetLastError());
    closesocket(s);
}

/* dnsapi directly: which layer drops the A record gethostbyname never sees? */
static void step_dnsquery(const char *name, DWORD options)
{
    PDNS_RECORD rec = NULL, r;
    DNS_STATUS st;

    SAY("step DnsQuery_A %s opts=%lx\n", name, options);
    st = DnsQuery_A(name, DNS_TYPE_A, options, NULL, &rec, NULL);
    SAY("DnsQuery_A %s -> status=%ld rec=%p\n", name, (long)st, rec);
    for (r = rec; r; r = r->pNext)
        SAY("  record type=%u name=%s len=%u a=%s\n", r->wType, r->pName ? r->pName : "(null)",
            r->wDataLength, r->wType == DNS_TYPE_A ? inet_ntoa(*(struct in_addr *)&r->Data.A.IpAddress) : "-");
    CHECK(st == 0 && rec, "DnsQuery_A %s opts=%lx status=%ld\n", name, options, (long)st);
    if (rec) DnsRecordListFree(rec, DnsFreeRecordList);
}

static ULONG step_dns(void)
{
    struct hostent *he;
    struct addrinfo hints, *ai = NULL;
    ULONG addr = 0;
    int rc;

    SAY("step dns\n");
    he = gethostbyname(HOST);
    CHECK(he != NULL, "gethostbyname %s wsa=%d\n", HOST, WSAGetLastError());
    if (he && he->h_addr_list[0]) {
        addr = *(ULONG *)he->h_addr_list[0];
        SAY("gethostbyname %s -> %s\n", HOST, inet_ntoa(*(struct in_addr *)&addr));
    }

    ZeroMemory(&hints, sizeof hints);
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    rc = getaddrinfo(HOST, "80", &hints, &ai);
    CHECK(rc == 0 && ai, "getaddrinfo %s rc=%d wsa=%d\n", HOST, rc, WSAGetLastError());
    if (rc == 0 && ai) {
        struct sockaddr_in *sin = (struct sockaddr_in *)ai->ai_addr;
        SAY("getaddrinfo %s -> %s\n", HOST, inet_ntoa(sin->sin_addr));
        if (!addr) addr = sin->sin_addr.s_addr;
        freeaddrinfo(ai);
    }
    return addr;
}

static void step_tcp(ULONG addr)
{
    struct sockaddr_in sin;
    SOCKET s;

    SAY("step tcp\n");
    if (!addr) { SAY("FAIL tcp: no address to connect to\n"); failures++; return; }
    s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    CHECK(s != INVALID_SOCKET, "socket wsa=%d\n", WSAGetLastError());
    if (s == INVALID_SOCKET) return;
    ZeroMemory(&sin, sizeof sin);
    sin.sin_family = AF_INET;
    sin.sin_port = htons(80);
    sin.sin_addr.s_addr = addr;
    if (connect(s, (struct sockaddr *)&sin, sizeof sin) == 0)
        SAY("connect %s:80 OK\n", inet_ntoa(sin.sin_addr));
    else
        CHECK(0, "connect %s:80 wsa=%d\n", inet_ntoa(sin.sin_addr), WSAGetLastError());
    closesocket(s);
}

static void step_url(HINTERNET hOpen, const WCHAR *url)
{
    HINTERNET h;
    DWORD status = 0, len = sizeof status, got, total = 0;
    char buf[4096];

    SAY("step url %ls\n", url);
    h = InternetOpenUrlW(hOpen, url, NULL, 0,
                         INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE, 0);
    CHECK(h != NULL, "InternetOpenUrlW %ls err=%lu\n", url, GetLastError());
    if (!h) return;
    if (HttpQueryInfoW(h, HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER, &status, &len, NULL))
        SAY("status %lu\n", status);
    else
        SAY("HttpQueryInfoW err=%lu\n", GetLastError());
    while (InternetReadFile(h, buf, sizeof buf, &got) && got)
        total += got;
    SAY("read %lu bytes (last err=%lu)\n", total, GetLastError());
    CHECK(total > 0, "%ls: empty body\n", url);
    InternetCloseHandle(h);
}

int main(void)
{
    WSADATA wsa;
    HINTERNET hOpen;
    ULONG addr;

    SAY("start\n");
    CHECK(WSAStartup(MAKEWORD(2, 2), &wsa) == 0, "WSAStartup\n");
    wait_dnscache();
    step_raw_udp("10.0.2.3");
    step_raw_udp("8.8.8.8");
    SAY("step tcp-by-ip\n");
    step_tcp(inet_addr("78.46.247.105")); /* rapps.reactos.org per the host's resolver */
    step_dnsquery("example.com", DNS_QUERY_STANDARD);
    step_dnsquery(HOST, DNS_QUERY_STANDARD);
    step_dnsquery(HOST, DNS_QUERY_BYPASS_CACHE);
    addr = step_dns();
    step_tcp(addr);

    SAY("step wininet\n");
    hOpen = InternetOpenW(L"netprobe", INTERNET_OPEN_TYPE_PRECONFIG, NULL, NULL, 0);
    CHECK(hOpen != NULL, "InternetOpenW err=%lu\n", GetLastError());
    if (hOpen) {
        step_url(hOpen, L"http://rapps.reactos.org/");
        step_url(hOpen, L"https://rapps.reactos.org/rappmgr2.cab");
        InternetCloseHandle(hOpen);
    }
    WSACleanup();
    SAY("done VERDICT=%s failures=%ld\n", failures ? "FAIL" : "PASS", failures);
    return failures != 0;
}
