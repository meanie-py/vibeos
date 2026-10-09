/* ndisprobe — ndis.sys end to end, through a real miniport, from user mode.
 *
 * Nothing else on our media reaches NDIS: netstack drives virtio-net directly and no
 * miniport binds in any default QEMU config, so DriverEntry is all of ndis.sys a boot runs.
 * Boot with a NIC an NDIS miniport drives (payload.yaml adds QEMU's e1000 82544GC), and ndisuio, the
 * NDIS protocol behind \\.\Ndisuio, puts every layer within reach:
 *   1. open \\.\Ndisuio and enumerate its bindings (protocol bind: NdisOpenAdapter);
 *   2. open the first binding;
 *   3. query OID_802_3_CURRENT_ADDRESS (NdisRequest -> MiniportQueryInformation);
 *   4. set OID_GEN_CURRENT_PACKET_FILTER to directed|broadcast (MiniportSetInformation);
 *   5. send an ARP who-has for slirp's gateway 10.0.2.2 (NdisSend -> MiniportSend);
 *   6. read frames until the ARP reply (interrupt -> DPC -> NdisMIndicateReceive ->
 *      ProtocolReceive).
 * Verdict: "[NDISPROBE] done VERDICT=PASS|FAIL" -- PASS needs every step. */
#include <windows.h>
#include <winioctl.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[NDISPROBE] " __VA_ARGS__)

/* sdk/include/reactos/drivers/ndisuio/nuiouser.h (not in mingw's headers) */
#define FILE_DEVICE_NETWORK_ 0x00000012
#define IOCTL_NDISUIO_OPEN_DEVICE CTL_CODE(FILE_DEVICE_NETWORK_, 0x200, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_NDISUIO_QUERY_OID_VALUE CTL_CODE(FILE_DEVICE_NETWORK_, 0x201, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_NDISUIO_QUERY_BINDING CTL_CODE(FILE_DEVICE_NETWORK_, 0x203, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_NDISUIO_SET_OID_VALUE CTL_CODE(FILE_DEVICE_NETWORK_, 0x205, METHOD_BUFFERED, FILE_ANY_ACCESS)
typedef struct { ULONG BindingIndex, DeviceNameOffset, DeviceNameLength, DeviceDescrOffset, DeviceDescrLength; } QUERY_BINDING;
#define OID_802_3_CURRENT_ADDRESS 0x01010102
#define OID_GEN_CURRENT_PACKET_FILTER 0x0001010E
#define OID_GEN_MEDIA_CONNECT_STATUS 0x00010114
#define NDIS_PACKET_TYPE_DIRECTED 0x1
#define NDIS_PACKET_TYPE_BROADCAST 0x8

static LONG failures;
#define FAIL(...) do { SAY("FAIL " __VA_ARGS__); failures++; } while (0)

static BOOL ioctl(HANDLE h, DWORD code, void *buf, DWORD in, DWORD out, DWORD *got)
{
    OVERLAPPED ov = {0};
    BOOL ok;
    ov.hEvent = CreateEventW(NULL, TRUE, FALSE, NULL);
    ok = DeviceIoControl(h, code, buf, in, buf, out, got, &ov);
    if (!ok && GetLastError() == ERROR_IO_PENDING)
        ok = GetOverlappedResult(h, &ov, got, TRUE);
    CloseHandle(ov.hEvent);
    return ok;
}

static void finish(void)
{
    SAY("done VERDICT=%s (%ld failure(s))\n", failures ? "FAIL" : "PASS", failures);
}

int main(void)
{
    HANDLE h;
    BYTE buf[512];
    WCHAR name[128];
    DWORD got = 0, i;
    BYTE mac[6];
    ULONG filter;
    BYTE frame[60], rx[1600];
    OVERLAPPED ov = {0};
    DWORD start;
    BOOL replied = FALSE;
    static const BYTE gw_ip[4] = {10, 0, 2, 2}, my_ip[4] = {10, 0, 2, 15};

    SAY("start\n");
    /* Give PnP time to install and start the miniport, and ndisuio to bind to it. */
    for (i = 0; i < 60; i++)
    {
        h = CreateFileW(L"\\\\.\\Ndisuio", GENERIC_READ | GENERIC_WRITE, 0, NULL, OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED, NULL);
        if (h == INVALID_HANDLE_VALUE)
        {
            Sleep(1000);
            continue;
        }
        memset(buf, 0, sizeof(buf));
        ((QUERY_BINDING *)buf)->BindingIndex = 0;
        if (ioctl(h, IOCTL_NDISUIO_QUERY_BINDING, buf, sizeof(buf), sizeof(buf), &got))
            break;
        CloseHandle(h);
        h = INVALID_HANDLE_VALUE;
        Sleep(1000);
    }
    if (h == INVALID_HANDLE_VALUE)
    {
        FAIL("step bind: no \\\\.\\Ndisuio binding after 60s (error %lu)\n", GetLastError());
        finish();
        return 1;
    }
    {
        QUERY_BINDING *qb = (QUERY_BINDING *)buf;
        DWORD n = qb->DeviceNameLength / sizeof(WCHAR);
        if (n >= 128)
            n = 127;
        memcpy(name, buf + qb->DeviceNameOffset, n * sizeof(WCHAR));
        name[n] = 0;
        SAY("step bind: binding 0 is %ls\n", name);
    }

    if (!ioctl(h, IOCTL_NDISUIO_OPEN_DEVICE, name, (DWORD)(wcslen(name) * sizeof(WCHAR)), 0, &got))
    {
        FAIL("step open: OPEN_DEVICE error %lu\n", GetLastError());
        finish();
        return 1;
    }
    SAY("step open: ok\n");

    memset(buf, 0, sizeof(buf));
    *(ULONG *)buf = OID_802_3_CURRENT_ADDRESS;
    if (!ioctl(h, IOCTL_NDISUIO_QUERY_OID_VALUE, buf, sizeof(ULONG) + 6, sizeof(ULONG) + 6, &got) || got < sizeof(ULONG) + 6)
    {
        FAIL("step mac: query error %lu, %lu bytes\n", GetLastError(), got);
        finish();
        return 1;
    }
    memcpy(mac, buf + sizeof(ULONG), 6);
    SAY("step mac: %02x:%02x:%02x:%02x:%02x:%02x\n", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    if (!(mac[0] | mac[1] | mac[2] | mac[3] | mac[4] | mac[5]))
        FAIL("step mac: all zero\n");

    memset(buf, 0, sizeof(buf));
    *(ULONG *)buf = OID_GEN_MEDIA_CONNECT_STATUS;
    if (ioctl(h, IOCTL_NDISUIO_QUERY_OID_VALUE, buf, 2 * sizeof(ULONG), 2 * sizeof(ULONG), &got))
        SAY("step link: media connect status %lu (0 = connected)\n", *(ULONG *)(buf + sizeof(ULONG)));
    else
        FAIL("step link: query error %lu\n", GetLastError());

    filter = NDIS_PACKET_TYPE_DIRECTED | NDIS_PACKET_TYPE_BROADCAST;
    *(ULONG *)buf = OID_GEN_CURRENT_PACKET_FILTER;
    memcpy(buf + sizeof(ULONG), &filter, sizeof(filter));
    if (!ioctl(h, IOCTL_NDISUIO_SET_OID_VALUE, buf, 2 * sizeof(ULONG), 0, &got))
        FAIL("step filter: set error %lu\n", GetLastError());
    else
        SAY("step filter: ok\n");

    /* ARP who-has 10.0.2.2 tell 10.0.2.15, broadcast, padded to the 60-byte minimum. */
    memset(frame, 0, sizeof(frame));
    memset(frame, 0xff, 6);
    memcpy(frame + 6, mac, 6);
    frame[12] = 0x08; frame[13] = 0x06;           /* ethertype ARP */
    frame[14] = 0x00; frame[15] = 0x01;           /* htype ethernet */
    frame[16] = 0x08; frame[17] = 0x00;           /* ptype IPv4 */
    frame[18] = 6; frame[19] = 4;
    frame[20] = 0x00; frame[21] = 0x01;           /* op request */
    memcpy(frame + 22, mac, 6);
    memcpy(frame + 28, my_ip, 4);
    memcpy(frame + 38, gw_ip, 4);

    ov.hEvent = CreateEventW(NULL, TRUE, FALSE, NULL);
    for (i = 0; i < 3 && !replied; i++)
    {
        if (!WriteFile(h, frame, sizeof(frame), &got, &ov) &&
            !(GetLastError() == ERROR_IO_PENDING && GetOverlappedResult(h, &ov, &got, TRUE)))
        {
            FAIL("step send: error %lu\n", GetLastError());
            break;
        }
        SAY("step send: %lu bytes (try %lu)\n", got, i + 1);
        start = GetTickCount();
        while (GetTickCount() - start < 5000)
        {
            ResetEvent(ov.hEvent);
            if (!ReadFile(h, rx, sizeof(rx), &got, &ov))
            {
                if (GetLastError() != ERROR_IO_PENDING)
                {
                    FAIL("step recv: error %lu\n", GetLastError());
                    break;
                }
                if (WaitForSingleObject(ov.hEvent, 5000) != WAIT_OBJECT_0)
                {
                    CancelIo(h);
                    GetOverlappedResult(h, &ov, &got, TRUE);
                    break;
                }
                if (!GetOverlappedResult(h, &ov, &got, FALSE))
                    continue;
            }
            SAY("step recv: %lu-byte frame, ethertype %02x%02x\n", got, rx[12], rx[13]);
            if (got >= 42 && rx[12] == 0x08 && rx[13] == 0x06 && rx[21] == 2 &&
                !memcmp(rx + 28, gw_ip, 4) && !memcmp(rx, mac, 6))
            {
                SAY("step recv: ARP reply, 10.0.2.2 is at %02x:%02x:%02x:%02x:%02x:%02x\n",
                    rx[22], rx[23], rx[24], rx[25], rx[26], rx[27]);
                replied = TRUE;
                break;
            }
        }
    }
    if (!replied)
        FAIL("step recv: no ARP reply from 10.0.2.2\n");
    CloseHandle(ov.hEvent);
    CloseHandle(h);
    finish();
    return failures ? 1 : 0;
}
