/*
 * PROJECT:     ReactOS Console Server DLL
 * FILE:        win32ss/user/winsrv/consrv/frontends/serial/serialterm.c
 * PURPOSE:     Serial Terminal Front-End for ARM64 (UART console)
 * PROGRAMMERS: ReactOS ARM64 Port
 */

#ifdef SERIALTERM_COMPILE

#include <consrv.h>

#include "include/console.h"
#include "include/settings.h"

#define NDEBUG
#include <debug.h>

/* Serial console private data */
typedef struct _SERIAL_CONSOLE_DATA
{
    HANDLE hSerialDevice;       /* Handle to \\.\COM1 */
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
static DWORD NTAPI
SerialInputThread(PVOID Param)
{
    PSERIAL_CONSOLE_DATA SerialData = (PSERIAL_CONSOLE_DATA)Param;
    PCONSRV_CONSOLE Console = SerialData->Console;
    UCHAR ch;
    DWORD BytesRead;
    INPUT_RECORD er;

    DPRINT1("[SERIAL] Input thread started\n");

    while (!SerialData->Terminate)
    {
        /* ReadFile on the serial device (pl011 driver handles polling) */
        if (!ReadFile(SerialData->hSerialDevice, &ch, 1, &BytesRead, NULL) ||
            BytesRead == 0)
        {
            /* No data or error; brief sleep and retry */
            Sleep(50);
            continue;
        }

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
        else if (ch == 0x1B) /* Escape */
        {
            er.Event.KeyEvent.wVirtualKeyCode = VK_ESCAPE;
            er.Event.KeyEvent.wVirtualScanCode = 0x01;
            er.Event.KeyEvent.uChar.UnicodeChar = 0x1B;
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

        /* Lock console and send key-down event */
        if (ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE))
        {
            ConioProcessInputEvent(Console, &er);

            /* Send corresponding key-up event */
            er.Event.KeyEvent.bKeyDown = FALSE;
            ConioProcessInputEvent(Console, &er);

            LeaveCriticalSection(&Console->Lock);
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
