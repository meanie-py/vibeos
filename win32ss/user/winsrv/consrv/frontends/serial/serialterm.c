/*
 * PROJECT:     ReactOS Console Server DLL
 * FILE:        win32ss/user/winsrv/consrv/frontends/serial/serialterm.c
 * PURPOSE:     Serial Terminal Front-End for ARM64 (UART console)
 * PROGRAMMERS: ReactOS ARM64 Port
 */

#ifdef SERIALTERM_COMPILE

#include <consrv.h>
#include <ntddmou.h>

#include "include/console.h"
#include "include/settings.h"

#define NDEBUG
#include <debug.h>

/* IOCTL to inject a byte into arm64input.sys keyboard driver */
#define IOCTL_ARM64INPUT_INJECT_BYTE \
    CTL_CODE(FILE_DEVICE_KEYBOARD, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)

/* IOCTL to inject a mouse event into arm64input.sys */
#define IOCTL_ARM64INPUT_INJECT_MOUSE \
    CTL_CODE(FILE_DEVICE_MOUSE, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)

/* Serial console private data */
typedef struct _SERIAL_CONSOLE_DATA
{
    HANDLE hSerialDevice;       /* Handle to \\.\COM1 */
    HANDLE hKeyboardDevice;     /* Handle to \\.\KeyboardClass0 (for RIT injection) */
    HANDLE hMouseDevice;        /* Handle to \\.\PointerClass0 (for mouse injection) */
    HANDLE hInputThread;        /* Input polling thread */
    volatile BOOLEAN Terminate; /* Signal thread to stop */

    PCONSRV_CONSOLE Console;    /* Owning console */
    PCONSOLE_SCREEN_BUFFER ActiveBuffer;
} SERIAL_CONSOLE_DATA, *PSERIAL_CONSOLE_DATA;

#define ConsoleOutputUnicodeToAnsiChar(Console, dChar, sWChar) \
do { \
    ASSERT((ULONG_PTR)(dChar) != (ULONG_PTR)(sWChar)); \
    WideCharToMultiByte((Console)->OutputCodePage, 0, (sWChar), 1, (dChar), 1, NULL, NULL); \
} while (0)

/* Forward declarations */
static NTSTATUS NTAPI SerialInitFrontEnd(IN OUT PFRONTEND This, IN PCONSRV_CONSOLE Console);
static VOID NTAPI SerialDeinitFrontEnd(IN OUT PFRONTEND This);
static VOID NTAPI SerialDrawRegion(IN OUT PFRONTEND This, SMALL_RECT* Region);
static VOID NTAPI SerialWriteStream(IN OUT PFRONTEND This, SMALL_RECT* Region,
                                     SHORT CursorStartX, SHORT CursorStartY,
                                     UINT ScrolledLines, PWCHAR Buffer, UINT Length);
static VOID NTAPI SerialRingBell(IN OUT PFRONTEND This);
static BOOL NTAPI SerialSetCursorInfo(IN OUT PFRONTEND This, PCONSOLE_SCREEN_BUFFER ScreenBuffer);
static BOOL NTAPI SerialSetScreenInfo(IN OUT PFRONTEND This, PCONSOLE_SCREEN_BUFFER ScreenBuffer,
                                       SHORT OldCursorX, SHORT OldCursorY);
static VOID NTAPI SerialResizeTerminal(IN OUT PFRONTEND This);
static VOID NTAPI SerialSetActiveScreenBuffer(IN OUT PFRONTEND This);
static VOID NTAPI SerialReleaseScreenBuffer(IN OUT PFRONTEND This, IN PCONSOLE_SCREEN_BUFFER ScreenBuffer);
static VOID NTAPI SerialRefreshInternalInfo(IN OUT PFRONTEND This);
static VOID NTAPI SerialChangeTitle(IN OUT PFRONTEND This);
static BOOL NTAPI SerialChangeIcon(IN OUT PFRONTEND This, HICON IconHandle);
static HDESK NTAPI SerialGetThreadConsoleDesktop(IN OUT PFRONTEND This);
static HWND NTAPI SerialGetConsoleWindowHandle(IN OUT PFRONTEND This);
static VOID NTAPI SerialGetLargestConsoleWindowSize(IN OUT PFRONTEND This, PCOORD pSize);
static BOOL NTAPI SerialGetSelectionInfo(IN OUT PFRONTEND This, PCONSOLE_SELECTION_INFO pSelectionInfo);
static BOOL NTAPI SerialSetPalette(IN OUT PFRONTEND This, HPALETTE PaletteHandle, UINT PaletteUsage);
static BOOL NTAPI SerialSetCodePage(IN OUT PFRONTEND This, UINT CodePage);
static ULONG NTAPI SerialGetDisplayMode(IN OUT PFRONTEND This);
static BOOL NTAPI SerialSetDisplayMode(IN OUT PFRONTEND This, ULONG NewMode);
static INT NTAPI SerialShowMouseCursor(IN OUT PFRONTEND This, BOOL Show);
static BOOL NTAPI SerialSetMouseCursor(IN OUT PFRONTEND This, HCURSOR CursorHandle);
static HMENU NTAPI SerialMenuControl(IN OUT PFRONTEND This, UINT CmdIdLow, UINT CmdIdHigh);
static BOOL NTAPI SerialSetMenuClose(IN OUT PFRONTEND This, BOOL Enable);

static FRONTEND_VTBL SerialVtbl =
{
    SerialInitFrontEnd,
    SerialDeinitFrontEnd,
    SerialDrawRegion,
    SerialWriteStream,
    SerialRingBell,
    SerialSetCursorInfo,
    SerialSetScreenInfo,
    SerialResizeTerminal,
    SerialSetActiveScreenBuffer,
    SerialReleaseScreenBuffer,
    SerialRefreshInternalInfo,
    SerialChangeTitle,
    SerialChangeIcon,
    SerialGetThreadConsoleDesktop,
    SerialGetConsoleWindowHandle,
    SerialGetLargestConsoleWindowSize,
    SerialGetSelectionInfo,
    SerialSetPalette,
    SerialSetCodePage,
    SerialGetDisplayMode,
    SerialSetDisplayMode,
    SerialShowMouseCursor,
    SerialSetMouseCursor,
    SerialMenuControl,
    SerialSetMenuClose,
};


/*
 * Input polling thread: reads bytes from serial, converts to INPUT_RECORD,
 * and queues them via ConioProcessInputEvent.
 */
/*
 * Process a single byte as keyboard input: build INPUT_RECORD, send to
 * console (ConioProcessInputEvent) and to Win32k RIT (arm64input IOCTL).
 */
static VOID
SerialProcessKeyByte(PSERIAL_CONSOLE_DATA SerialData, UCHAR ch)
{
    PCONSRV_CONSOLE Console = SerialData->Console;
    INPUT_RECORD er;

    /* Build a KEY_EVENT INPUT_RECORD */
    RtlZeroMemory(&er, sizeof(er));
    er.EventType = KEY_EVENT;
    er.Event.KeyEvent.bKeyDown = TRUE;
    er.Event.KeyEvent.wRepeatCount = 1;
    er.Event.KeyEvent.dwControlKeyState = 0;

    /* Map special characters to virtual key codes */
    if (ch == '\r')
    {
        er.Event.KeyEvent.wVirtualKeyCode = VK_RETURN;
        er.Event.KeyEvent.wVirtualScanCode = 0x1C;
        er.Event.KeyEvent.uChar.UnicodeChar = L'\r';
    }
    else if (ch == 0x08 || ch == 0x7F)
    {
        er.Event.KeyEvent.wVirtualKeyCode = VK_BACK;
        er.Event.KeyEvent.wVirtualScanCode = 0x0E;
        er.Event.KeyEvent.uChar.UnicodeChar = L'\b';
    }
    else if (ch == 0x03) /* Ctrl-C */
    {
        er.Event.KeyEvent.wVirtualKeyCode = 'C';
        er.Event.KeyEvent.wVirtualScanCode = 0x2E;
        er.Event.KeyEvent.uChar.UnicodeChar = 0x03;
        er.Event.KeyEvent.dwControlKeyState = LEFT_CTRL_PRESSED;
    }
    else if (ch == '\t')
    {
        er.Event.KeyEvent.wVirtualKeyCode = VK_TAB;
        er.Event.KeyEvent.wVirtualScanCode = 0x0F;
        er.Event.KeyEvent.uChar.UnicodeChar = L'\t';
    }
    else
    {
        /* Regular character */
        WCHAR wch;
        MultiByteToWideChar(Console->InputCodePage, 0, (LPCSTR)&ch, 1, &wch, 1);
        er.Event.KeyEvent.uChar.UnicodeChar = wch;
        er.Event.KeyEvent.wVirtualKeyCode = VkKeyScanW(wch) & 0xFF;
        er.Event.KeyEvent.wVirtualScanCode = 0;
    }

    /* Lock console and send key-down + key-up events */
    if (ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE))
    {
        ConioProcessInputEvent(Console, &er);
        er.Event.KeyEvent.bKeyDown = FALSE;
        ConioProcessInputEvent(Console, &er);
        LeaveCriticalSection(&Console->Lock);
    }

    /* Also inject into KeyboardClass0 for the Win32k RIT */
    if (SerialData->hKeyboardDevice &&
        SerialData->hKeyboardDevice != INVALID_HANDLE_VALUE)
    {
        DWORD Ignored;
        DeviceIoControl(SerialData->hKeyboardDevice,
                        IOCTL_ARM64INPUT_INJECT_BYTE,
                        &ch, 1,
                        NULL, 0,
                        &Ignored, NULL);
    }
}

/*
 * Process a mouse escape sequence: ESC [ M <action> <xl> <xh> <yl> <yh>
 * Constructs MOUSE_INPUT_DATA and injects via IOCTL to PointerClass0.
 *
 * Action byte: 0=move, 1=left-down, 2=left-up, 3=right-down, 4=right-up
 * X/Y: 16-bit little-endian, 0-65535 range (MOUSE_MOVE_ABSOLUTE)
 */
static VOID
SerialProcessMouseEscape(PSERIAL_CONSOLE_DATA SerialData, UCHAR *buf)
{
    MOUSE_INPUT_DATA mid;
    DWORD Ignored;
    UCHAR action = buf[0];
    USHORT x = (USHORT)buf[1] | ((USHORT)buf[2] << 8);
    USHORT y = (USHORT)buf[3] | ((USHORT)buf[4] << 8);

    if (!SerialData->hMouseDevice ||
        SerialData->hMouseDevice == INVALID_HANDLE_VALUE)
        return;

    RtlZeroMemory(&mid, sizeof(mid));
    mid.Flags = MOUSE_MOVE_ABSOLUTE;
    mid.LastX = (LONG)x;
    mid.LastY = (LONG)y;

    switch (action)
    {
        case 0: /* Move only */
            break;
        case 1: /* Left button down */
            mid.ButtonFlags = MOUSE_LEFT_BUTTON_DOWN;
            break;
        case 2: /* Left button up */
            mid.ButtonFlags = MOUSE_LEFT_BUTTON_UP;
            break;
        case 3: /* Right button down */
            mid.ButtonFlags = MOUSE_RIGHT_BUTTON_DOWN;
            break;
        case 4: /* Right button up */
            mid.ButtonFlags = MOUSE_RIGHT_BUTTON_UP;
            break;
        default:
            return;
    }

    DeviceIoControl(SerialData->hMouseDevice,
                    IOCTL_ARM64INPUT_INJECT_MOUSE,
                    &mid, sizeof(mid),
                    NULL, 0,
                    &Ignored, NULL);
}

/*
 * Input polling thread: reads bytes from serial, converts to INPUT_RECORD,
 * and queues them via ConioProcessInputEvent.
 *
 * Mouse escape sequence: ESC [ M <action> <xl> <xh> <yl> <yh> (8 bytes total)
 */
static DWORD NTAPI
SerialInputThread(PVOID Param)
{
    PSERIAL_CONSOLE_DATA SerialData = (PSERIAL_CONSOLE_DATA)Param;
    UCHAR ch;
    DWORD BytesRead;
    int esc_state = 0;
    UCHAR esc_buf[5];
    int esc_pos = 0;

    DPRINT1("[SERIAL] Input thread started\n");

    /* Run at BELOW_NORMAL so we don't starve explorer/desktop rendering on
     * arm64 (no timer interrupts → cooperative scheduling, so a high-priority
     * polling loop that re-enters the kernel for ~5ms per Read syscall pegs
     * the CPU and blocks lower-priority threads from making forward progress).
     * Serial input is best-effort here; the smoke harness doesn't drive
     * console input via COM1 anyway. */
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);

    while (!SerialData->Terminate)
    {
        /* ReadFile on the serial device (pl011 driver handles polling) */
        if (!ReadFile(SerialData->hSerialDevice, &ch, 1, &BytesRead, NULL) ||
            BytesRead == 0)
        {
            /* No data; yield and retry. */
            Sleep(50);
            continue;
        }

        /*
         * Escape sequence state machine for mouse events:
         * State 0: Normal input
         * State 1: Got ESC (0x1B)
         * State 2: Got ESC [
         * State 3: Got ESC [ M, collecting 5 data bytes
         */
        switch (esc_state)
        {
        case 0:
            if (ch == 0x1B)
            {
                esc_state = 1;
                continue;
            }
            SerialProcessKeyByte(SerialData, ch);
            continue;

        case 1: /* Got ESC */
            if (ch == '[')
            {
                esc_state = 2;
                continue;
            }
            /* Not an escape sequence — emit ESC as keyboard, then this byte */
            SerialProcessKeyByte(SerialData, 0x1B);
            esc_state = 0;
            SerialProcessKeyByte(SerialData, ch);
            continue;

        case 2: /* Got ESC [ */
            if (ch == 'M')
            {
                esc_state = 3;
                esc_pos = 0;
                continue;
            }
            /* Not a mouse sequence — emit ESC, [, then this byte */
            SerialProcessKeyByte(SerialData, 0x1B);
            SerialProcessKeyByte(SerialData, '[');
            esc_state = 0;
            SerialProcessKeyByte(SerialData, ch);
            continue;

        case 3: /* Collecting mouse data bytes */
            esc_buf[esc_pos++] = ch;
            if (esc_pos == 5)
            {
                SerialProcessMouseEscape(SerialData, esc_buf);
                esc_state = 0;
            }
            continue;
        }
    }

    DPRINT1("[SERIAL] Input thread exiting\n");
    return 0;
}


static NTSTATUS NTAPI
SerialInitFrontEnd(IN OUT PFRONTEND This,
                   IN PCONSRV_CONSOLE Console)
{
    PSERIAL_CONSOLE_DATA SerialData;
    HANDLE hSerial;
    HANDLE ThreadHandle;

    if (This == NULL || Console == NULL)
        return STATUS_INVALID_PARAMETER;

    DPRINT1("[SERIAL] SerialInitFrontEnd: Initializing serial frontend\n");

    /* Allocate private data */
    SerialData = ConsoleAllocHeap(HEAP_ZERO_MEMORY, sizeof(SERIAL_CONSOLE_DATA));
    if (!SerialData)
        return STATUS_NO_MEMORY;

    SerialData->Console = Console;
    SerialData->ActiveBuffer = Console->ActiveBuffer;
    SerialData->Terminate = FALSE;

    /* Open serial device */
    hSerial = CreateFileW(L"\\\\.\\COM1",
                          GENERIC_READ | GENERIC_WRITE,
                          0, NULL,
                          OPEN_EXISTING,
                          FILE_ATTRIBUTE_NORMAL,
                          NULL);
    if (hSerial == INVALID_HANDLE_VALUE)
    {
        DPRINT1("[SERIAL] Failed to open COM1, error %lu\n", GetLastError());
        ConsoleFreeHeap(SerialData);
        return STATUS_UNSUCCESSFUL;
    }
    SerialData->hSerialDevice = hSerial;

    DPRINT1("[SERIAL] COM1 opened successfully\n");

    /* Open keyboard class device for RIT injection */
    SerialData->hKeyboardDevice = CreateFileW(L"\\\\.\\KeyboardClass0",
                                               GENERIC_WRITE,
                                               FILE_SHARE_READ | FILE_SHARE_WRITE,
                                               NULL,
                                               OPEN_EXISTING,
                                               FILE_ATTRIBUTE_NORMAL,
                                               NULL);
    if (SerialData->hKeyboardDevice == INVALID_HANDLE_VALUE)
    {
        DPRINT1("[SERIAL] Failed to open KeyboardClass0, error %lu (RIT injection disabled)\n",
                GetLastError());
    }
    else
    {
        DPRINT1("[SERIAL] KeyboardClass0 opened for RIT injection\n");
    }

    /* Open mouse class device for mouse injection */
    SerialData->hMouseDevice = CreateFileW(L"\\\\.\\PointerClass0",
                                            GENERIC_WRITE,
                                            FILE_SHARE_READ | FILE_SHARE_WRITE,
                                            NULL,
                                            OPEN_EXISTING,
                                            FILE_ATTRIBUTE_NORMAL,
                                            NULL);
    if (SerialData->hMouseDevice == INVALID_HANDLE_VALUE)
    {
        DPRINT1("[SERIAL] Failed to open PointerClass0, error %lu (mouse injection disabled)\n",
                GetLastError());
    }
    else
    {
        DPRINT1("[SERIAL] PointerClass0 opened for mouse injection\n");
    }

    /* Store context */
    This->Context = SerialData;
    This->Context2 = NULL;

    /* Create input polling thread */
    ThreadHandle = CreateThread(NULL, 0, SerialInputThread, SerialData, 0, NULL);
    if (ThreadHandle == NULL)
    {
        DPRINT1("[SERIAL] Failed to create input thread\n");
        CloseHandle(hSerial);
        ConsoleFreeHeap(SerialData);
        This->Context = NULL;
        return STATUS_UNSUCCESSFUL;
    }
    SerialData->hInputThread = ThreadHandle;

    DPRINT1("[SERIAL] Serial frontend initialized OK\n");
    return STATUS_SUCCESS;
}


static VOID NTAPI
SerialDeinitFrontEnd(IN OUT PFRONTEND This)
{
    PSERIAL_CONSOLE_DATA SerialData;

    if (!This || !This->Context)
        return;

    SerialData = (PSERIAL_CONSOLE_DATA)This->Context;

    /* Signal thread to stop */
    SerialData->Terminate = TRUE;

    /* Close the serial device to unblock ReadFile */
    if (SerialData->hSerialDevice && SerialData->hSerialDevice != INVALID_HANDLE_VALUE)
        CloseHandle(SerialData->hSerialDevice);

    /* Close keyboard device */
    if (SerialData->hKeyboardDevice && SerialData->hKeyboardDevice != INVALID_HANDLE_VALUE)
        CloseHandle(SerialData->hKeyboardDevice);

    /* Close mouse device */
    if (SerialData->hMouseDevice && SerialData->hMouseDevice != INVALID_HANDLE_VALUE)
        CloseHandle(SerialData->hMouseDevice);

    /* Wait briefly for thread to exit */
    if (SerialData->hInputThread)
    {
        WaitForSingleObject(SerialData->hInputThread, 2000);
        CloseHandle(SerialData->hInputThread);
    }

    ConsoleFreeHeap(SerialData);
    This->Context = NULL;
}


static VOID NTAPI
SerialDrawRegion(IN OUT PFRONTEND This, SMALL_RECT* Region)
{
    /* No-op: serial terminal handles scrolling naturally */
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(Region);
}


static VOID NTAPI
SerialWriteStream(IN OUT PFRONTEND This,
                  SMALL_RECT* Region,
                  SHORT CursorStartX,
                  SHORT CursorStartY,
                  UINT ScrolledLines,
                  PWCHAR Buffer,
                  UINT Length)
{
    PSERIAL_CONSOLE_DATA SerialData;
    PCHAR AnsiBuffer;
    ULONG AnsiLength;
    DWORD BytesWritten;

    UNREFERENCED_PARAMETER(Region);
    UNREFERENCED_PARAMETER(CursorStartX);
    UNREFERENCED_PARAMETER(CursorStartY);
    UNREFERENCED_PARAMETER(ScrolledLines);

    if (!This || !This->Context || !Buffer || Length == 0)
        return;

    SerialData = (PSERIAL_CONSOLE_DATA)This->Context;

    /* Convert wide string to ANSI using the console's output code page */
    AnsiLength = WideCharToMultiByte(SerialData->Console->OutputCodePage, 0,
                                     Buffer, Length,
                                     NULL, 0, NULL, NULL);
    if (AnsiLength == 0)
        return;

    AnsiBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, AnsiLength);
    if (!AnsiBuffer)
        return;

    WideCharToMultiByte(SerialData->Console->OutputCodePage, 0,
                        Buffer, Length,
                        AnsiBuffer, AnsiLength, NULL, NULL);

    /* Write to serial device (pl011 driver handles LF→CRLF) */
    WriteFile(SerialData->hSerialDevice, AnsiBuffer, AnsiLength, &BytesWritten, NULL);

    RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiBuffer);
}


/* --- No-op stubs for unused frontend functions --- */

static VOID NTAPI
SerialRingBell(IN OUT PFRONTEND This)
{
    /* Could write BEL char to serial, but not needed */
    UNREFERENCED_PARAMETER(This);
}

static BOOL NTAPI
SerialSetCursorInfo(IN OUT PFRONTEND This, PCONSOLE_SCREEN_BUFFER ScreenBuffer)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(ScreenBuffer);
    return TRUE;
}

static BOOL NTAPI
SerialSetScreenInfo(IN OUT PFRONTEND This, PCONSOLE_SCREEN_BUFFER ScreenBuffer,
                     SHORT OldCursorX, SHORT OldCursorY)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(ScreenBuffer);
    UNREFERENCED_PARAMETER(OldCursorX);
    UNREFERENCED_PARAMETER(OldCursorY);
    return TRUE;
}

static VOID NTAPI
SerialResizeTerminal(IN OUT PFRONTEND This)
{
    UNREFERENCED_PARAMETER(This);
}

static VOID NTAPI
SerialSetActiveScreenBuffer(IN OUT PFRONTEND This)
{
    UNREFERENCED_PARAMETER(This);
}

static VOID NTAPI
SerialReleaseScreenBuffer(IN OUT PFRONTEND This, IN PCONSOLE_SCREEN_BUFFER ScreenBuffer)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(ScreenBuffer);
}

static VOID NTAPI
SerialRefreshInternalInfo(IN OUT PFRONTEND This)
{
    UNREFERENCED_PARAMETER(This);
}

static VOID NTAPI
SerialChangeTitle(IN OUT PFRONTEND This)
{
    UNREFERENCED_PARAMETER(This);
}

static BOOL NTAPI
SerialChangeIcon(IN OUT PFRONTEND This, HICON IconHandle)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(IconHandle);
    return TRUE;
}

static HDESK NTAPI
SerialGetThreadConsoleDesktop(IN OUT PFRONTEND This)
{
    UNREFERENCED_PARAMETER(This);
    return NULL;
}

static HWND NTAPI
SerialGetConsoleWindowHandle(IN OUT PFRONTEND This)
{
    UNREFERENCED_PARAMETER(This);
    return NULL;
}

static VOID NTAPI
SerialGetLargestConsoleWindowSize(IN OUT PFRONTEND This, PCOORD pSize)
{
    UNREFERENCED_PARAMETER(This);
    if (pSize)
    {
        pSize->X = 80;
        pSize->Y = 25;
    }
}

static BOOL NTAPI
SerialGetSelectionInfo(IN OUT PFRONTEND This, PCONSOLE_SELECTION_INFO pSelectionInfo)
{
    UNREFERENCED_PARAMETER(This);
    if (pSelectionInfo)
        RtlZeroMemory(pSelectionInfo, sizeof(*pSelectionInfo));
    return TRUE;
}

static BOOL NTAPI
SerialSetPalette(IN OUT PFRONTEND This, HPALETTE PaletteHandle, UINT PaletteUsage)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(PaletteHandle);
    UNREFERENCED_PARAMETER(PaletteUsage);
    return TRUE;
}

static BOOL NTAPI
SerialSetCodePage(IN OUT PFRONTEND This, UINT CodePage)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(CodePage);
    return TRUE;
}

static ULONG NTAPI
SerialGetDisplayMode(IN OUT PFRONTEND This)
{
    UNREFERENCED_PARAMETER(This);
    return CONSOLE_FULLSCREEN_HARDWARE;
}

static BOOL NTAPI
SerialSetDisplayMode(IN OUT PFRONTEND This, ULONG NewMode)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(NewMode);
    return TRUE;
}

static INT NTAPI
SerialShowMouseCursor(IN OUT PFRONTEND This, BOOL Show)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(Show);
    return 0;
}

static BOOL NTAPI
SerialSetMouseCursor(IN OUT PFRONTEND This, HCURSOR CursorHandle)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(CursorHandle);
    return TRUE;
}

static HMENU NTAPI
SerialMenuControl(IN OUT PFRONTEND This, UINT CmdIdLow, UINT CmdIdHigh)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(CmdIdLow);
    UNREFERENCED_PARAMETER(CmdIdHigh);
    return NULL;
}

static BOOL NTAPI
SerialSetMenuClose(IN OUT PFRONTEND This, BOOL Enable)
{
    UNREFERENCED_PARAMETER(This);
    UNREFERENCED_PARAMETER(Enable);
    return TRUE;
}


/* --- Load/Unload --- */

NTSTATUS NTAPI
SerialLoadFrontEnd(IN OUT PFRONTEND FrontEnd,
                   IN OUT PCONSOLE_STATE_INFO ConsoleInfo,
                   IN OUT PCONSOLE_INIT_INFO ConsoleInitInfo,
                   IN HANDLE ConsoleLeaderProcessHandle)
{
    UNREFERENCED_PARAMETER(ConsoleInitInfo);
    UNREFERENCED_PARAMETER(ConsoleLeaderProcessHandle);

    if (FrontEnd == NULL || ConsoleInfo == NULL)
        return STATUS_INVALID_PARAMETER;

    DPRINT1("[SERIAL] SerialLoadFrontEnd called\n");

    /* Set the vtbl; actual initialization happens in SerialInitFrontEnd */
    FrontEnd->Vtbl = &SerialVtbl;
    FrontEnd->Context = NULL;
    FrontEnd->Context2 = NULL;

    return STATUS_SUCCESS;
}

NTSTATUS NTAPI
SerialUnloadFrontEnd(IN OUT PFRONTEND FrontEnd)
{
    if (FrontEnd == NULL)
        return STATUS_INVALID_PARAMETER;
    if (FrontEnd->Context)
        SerialDeinitFrontEnd(FrontEnd);
    return STATUS_SUCCESS;
}

#endif /* SERIALTERM_COMPILE */

/* EOF */
