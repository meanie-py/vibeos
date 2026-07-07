/*
 * stortest.exe — usermode storage proof for the USB mass-storage stack.
 *
 * Closes the loop on the whole storage chain from the top: usbport/usbehci ->
 * usbstor -> disk.sys(classpnp) -> partmgr -> fastfat mount -> mountmgr drive
 * letter -> real Win32 file IO. The test harness attaches a QEMU usb-storage
 * disk (MBR + FAT16, label VIBEUSB, containing MARKER.TXT); this program finds
 * that volume by label, reads the marker, then writes a file and reads it back.
 * The write is also verifiable from the HOST after shutdown (mdir/mcopy on the
 * disk image), giving an end-to-end oracle that bytes really hit the medium.
 *
 * Spawned by winlogon (same pattern as nettest.exe), gated on the presence of
 * HKLM\SYSTEM\CurrentControlSet\Enum\USBSTOR so plain smoke boots never pay
 * the process-creation race against explorer's csrss/win32k handoff. Emits
 * [STORTEST] markers via OutputDebugStringA -> debug serial.
 */

#include <windows.h>
#include <stdio.h>
#include <stdarg.h>
#include <string.h>
#include <mountmgr.h>

#define MARKER_SUBSTR   "vibeos usb e2e marker"
#define WRITE_CONTENT   "vibeos wrote this through fastfat+usbstor\r\n"

static void mark(const char *msg)
{
    OutputDebugStringA(msg);
}

static void markf(const char *fmt, ...)
{
    char buf[256];
    va_list ap;
    va_start(ap, fmt);
    _vsnprintf(buf, sizeof(buf) - 1, fmt, ap);
    va_end(ap);
    buf[sizeof(buf) - 1] = 0;
    OutputDebugStringA(buf);
}

/* Scan D:..Z: for a mounted volume labeled VIBEUSB. Returns the letter or 0. */
static char find_vibeusb(void)
{
    char root[8];
    char label[MAX_PATH];
    char letter;

    for (letter = 'D'; letter <= 'Z'; letter++)
    {
        sprintf(root, "%c:\\", letter);
        label[0] = 0;
        if (GetVolumeInformationA(root, label, sizeof(label), NULL, NULL, NULL, NULL, 0) &&
            _stricmp(label, "VIBEUSB") == 0)
        {
            return letter;
        }
    }
    return 0;
}

static int test_read_marker(char letter)
{
    char path[32];
    char buf[256];
    HANDLE h;
    DWORD n = 0;

    sprintf(path, "%c:\\MARKER.TXT", letter);
    h = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        markf("[STORTEST] FAIL: open %s (error %lu)\n", path, GetLastError());
        return 0;
    }
    if (!ReadFile(h, buf, sizeof(buf) - 1, &n, NULL))
    {
        markf("[STORTEST] FAIL: read %s (error %lu)\n", path, GetLastError());
        CloseHandle(h);
        return 0;
    }
    CloseHandle(h);
    buf[n] = 0;
    if (strstr(buf, MARKER_SUBSTR) == NULL)
    {
        markf("[STORTEST] FAIL: MARKER.TXT content mismatch (%lu bytes)\n", n);
        return 0;
    }
    markf("[STORTEST] read MARKER.TXT OK (%lu bytes)\n", n);
    return 1;
}

static int test_write_readback(char letter)
{
    char path[32];
    char buf[128];
    HANDLE h;
    DWORD n = 0;

    sprintf(path, "%c:\\WRITETST.TXT", letter);
    h = CreateFileA(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        markf("[STORTEST] FAIL: create %s (error %lu)\n", path, GetLastError());
        return 0;
    }
    if (!WriteFile(h, WRITE_CONTENT, (DWORD)strlen(WRITE_CONTENT), &n, NULL) ||
        n != strlen(WRITE_CONTENT))
    {
        markf("[STORTEST] FAIL: write %s (error %lu, wrote %lu)\n", path, GetLastError(), n);
        CloseHandle(h);
        return 0;
    }
    if (!FlushFileBuffers(h))
    {
        markf("[STORTEST] FAIL: flush %s (error %lu)\n", path, GetLastError());
        CloseHandle(h);
        return 0;
    }
    CloseHandle(h);

    h = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        markf("[STORTEST] FAIL: reopen %s (error %lu)\n", path, GetLastError());
        return 0;
    }
    n = 0;
    if (!ReadFile(h, buf, sizeof(buf) - 1, &n, NULL))
    {
        markf("[STORTEST] FAIL: readback %s (error %lu)\n", path, GetLastError());
        CloseHandle(h);
        return 0;
    }
    CloseHandle(h);
    buf[n] = 0;
    if (n != strlen(WRITE_CONTENT) || memcmp(buf, WRITE_CONTENT, n) != 0)
    {
        markf("[STORTEST] FAIL: readback mismatch (%lu bytes)\n", n);
        return 0;
    }
    markf("[STORTEST] write+readback OK (%lu bytes)\n", n);
    return 1;
}

/* mountvol-equivalent: ask mountmgr to assign the next free drive letter to
 * every volume it knows that doesn't have one yet. Late-arriving (hot-plug)
 * volumes never get a letter automatically — only boot-time
 * xHalIoAssignDriveLetters and explicit NEXT_DRIVE_LETTER clients (diskpart,
 * mountvol) assign them. This is the real-client path, not a test shortcut. */
static void request_drive_letters(void)
{
    HANDLE mgr;
    MOUNTMGR_MOUNT_POINT in;
    BYTE outbuf[8192];
    PMOUNTMGR_MOUNT_POINTS points;
    DWORD bytes = 0;
    ULONG i;

    mgr = CreateFileW(MOUNTMGR_DOS_DEVICE_NAME, 0,
                      FILE_SHARE_READ | FILE_SHARE_WRITE, NULL,
                      OPEN_EXISTING, 0, NULL);
    if (mgr == INVALID_HANDLE_VALUE)
    {
        markf("[STORTEST] open MountPointManager failed (error %lu)\n", GetLastError());
        return;
    }

    ZeroMemory(&in, sizeof(in));
    ZeroMemory(outbuf, sizeof(outbuf));
    if (!DeviceIoControl(mgr, IOCTL_MOUNTMGR_QUERY_POINTS, &in, sizeof(in),
                         outbuf, sizeof(outbuf), &bytes, NULL))
    {
        markf("[STORTEST] QUERY_POINTS failed (error %lu)\n", GetLastError());
        CloseHandle(mgr);
        return;
    }

    points = (PMOUNTMGR_MOUNT_POINTS)outbuf;
    markf("[STORTEST] mountmgr knows %lu mount points\n", points->NumberOfMountPoints);

    for (i = 0; i < points->NumberOfMountPoints; i++)
    {
        PMOUNTMGR_MOUNT_POINT pt = &points->MountPoints[i];
        BYTE reqbuf[1024];
        PMOUNTMGR_DRIVE_LETTER_TARGET target = (PMOUNTMGR_DRIVE_LETTER_TARGET)reqbuf;
        MOUNTMGR_DRIVE_LETTER_INFORMATION letterInfo;
        DWORD outBytes = 0;

        if (pt->DeviceNameLength == 0 ||
            pt->DeviceNameLength > sizeof(reqbuf) - sizeof(USHORT))
            continue;

        target->DeviceNameLength = pt->DeviceNameLength;
        memcpy(target->DeviceName, outbuf + pt->DeviceNameOffset, pt->DeviceNameLength);

        ZeroMemory(&letterInfo, sizeof(letterInfo));
        if (DeviceIoControl(mgr, IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER,
                            target, sizeof(USHORT) + target->DeviceNameLength,
                            &letterInfo, sizeof(letterInfo), &outBytes, NULL))
        {
            markf("[STORTEST] next-letter: assigned=%u letter=%c\n",
                  letterInfo.DriveLetterWasAssigned,
                  letterInfo.CurrentDriveLetter ? letterInfo.CurrentDriveLetter : '?');
        }
        else
        {
            markf("[STORTEST] next-letter failed (error %lu)\n", GetLastError());
        }
    }
    CloseHandle(mgr);
}

int main(void)
{
    char letter = 0;
    int attempt;
    int ok = 1;

    mark("[STORTEST] start\n");

    /* The volume may mount + get its letter a little after the shell is up;
     * poll for it. 60 x 2s = 120s ceiling. Every few attempts, act as the
     * mountvol client and ask mountmgr to hand out letters to letterless
     * volumes (late arrivals never get one automatically). */
    for (attempt = 0; attempt < 60 && letter == 0; attempt++)
    {
        if ((attempt % 5) == 0)
            request_drive_letters();
        letter = find_vibeusb();
        if (letter == 0)
            Sleep(2000);
    }

    if (letter == 0)
    {
        mark("[STORTEST] FAIL: no VIBEUSB volume found after 120s\n");
        return 1;
    }
    markf("[STORTEST] found VIBEUSB at %c:\n", letter);

    ok &= test_read_marker(letter);
    ok &= test_write_readback(letter);

    if (ok)
        markf("[STORTEST] ALL OK drive %c:\n", letter);
    else
        mark("[STORTEST] FAILURES (see above)\n");
    return ok ? 0 : 1;
}
