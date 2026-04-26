/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     In-kernel test dispatcher — serial command loop
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Activated by /TESTKERNEL boot option + REACTOS_TEST_KERNEL compile flag.
 * Pytest sends commands over serial; kernel runs tests and reports results.
 *
 * Protocol:
 *   Kernel -> Host:  TESTKERNEL READY\n
 *   Host -> Kernel:  RUN TestName\n
 *   Kernel -> Host:  [TEST START] TestName\n
 *   Kernel -> Host:  [TEST PASS] TestName\n  or  [TEST FAIL] TestName: reason\n
 *   Host -> Kernel:  LIST\n
 *   Kernel -> Host:  [TEST LIST] Name1,Name2,...\n
 *   Host -> Kernel:  QUIT\n
 *   Kernel -> Host:  [TEST QUIT]\n
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/* Import serial port interface from kd subsystem */
#include "../kd/kd.h"

/* Test function signature: returns TRUE on pass, FALSE on fail */
typedef BOOLEAN (NTAPI *PKERNEL_TEST_FUNC)(VOID);

/* Forward-declare test functions */
extern BOOLEAN NTAPI NtTestMmPool(VOID);
extern BOOLEAN NTAPI NtTestObHandle(VOID);
extern BOOLEAN NTAPI NtTestKeEvent(VOID);
extern BOOLEAN NTAPI NtTestDpc(VOID);
extern BOOLEAN NTAPI NtTestTimer(VOID);
extern BOOLEAN NTAPI NtTestVmem(VOID);
extern BOOLEAN NTAPI NtTestRegistry(VOID);
extern BOOLEAN NTAPI NtTestWorkItem(VOID);
extern BOOLEAN NTAPI NtTestIrql(VOID);
extern BOOLEAN NTAPI NtTestThread(VOID);
extern BOOLEAN NTAPI NtTestMutex(VOID);
extern BOOLEAN NTAPI NtTestSemaphore(VOID);
extern BOOLEAN NTAPI NtTestMultiWait(VOID);
extern BOOLEAN NTAPI NtTestObRef(VOID);
extern BOOLEAN NTAPI NtTestRtlStr(VOID);
extern BOOLEAN NTAPI NtTestSList(VOID);
extern BOOLEAN NTAPI NtTestMdl(VOID);
extern BOOLEAN NTAPI NtTestFileIo(VOID);
extern BOOLEAN NTAPI NtTestLookaside(VOID);
extern BOOLEAN NTAPI NtTestFastMutex(VOID);
extern BOOLEAN NTAPI NtTestBitmap(VOID);
extern BOOLEAN NTAPI NtTestCapture(VOID);
extern BOOLEAN NTAPI NtTestMemOps(VOID);
extern BOOLEAN NTAPI NtTestSysTime(VOID);
extern BOOLEAN NTAPI NtTestProcInfo(VOID);
extern BOOLEAN NTAPI NtTestObjDir(VOID);
extern BOOLEAN NTAPI NtTestSymLink(VOID);
extern BOOLEAN NTAPI NtTestInterlockedList(VOID);

/* Test registry table */
static const struct {
    const char *Name;
    PKERNEL_TEST_FUNC Func;
} TestTable[] = {
    { "MmPool",     NtTestMmPool },
    { "ObHandle",   NtTestObHandle },
    { "KeEvent",    NtTestKeEvent },
    { "Dpc",        NtTestDpc },
    { "Timer",      NtTestTimer },
    { "Vmem",       NtTestVmem },
    { "Registry",   NtTestRegistry },
    { "WorkItem",   NtTestWorkItem },
    { "Irql",       NtTestIrql },
    { "Thread",     NtTestThread },
    { "Mutex",      NtTestMutex },
    { "Semaphore",  NtTestSemaphore },
    { "MultiWait",  NtTestMultiWait },
    { "ObRef",      NtTestObRef },
    { "RtlStr",     NtTestRtlStr },
    { "SList",      NtTestSList },
    { "Mdl",        NtTestMdl },
    { "FileIo",     NtTestFileIo },
    { "Lookaside",  NtTestLookaside },
    { "FastMutex",  NtTestFastMutex },
    { "Bitmap",     NtTestBitmap },
    { "Capture",    NtTestCapture },
    { "MemOps",     NtTestMemOps },
    { "SysTime",    NtTestSysTime },
    { "ProcInfo",   NtTestProcInfo },
    { "ObjDir",     NtTestObjDir },
    { "SymLink",    NtTestSymLink },
    { "InterlockedList", NtTestInterlockedList },
    { NULL, NULL }  /* sentinel */
};

/* Read one byte from serial, non-blocking. Returns TRUE if byte received. */
static BOOLEAN
TestReadByte(PUCHAR Byte)
{
    return KdPortGetByteEx(&SerialPortInfo, Byte);
}

/* Read a line from serial into Buffer (up to MaxLen-1 chars).
 * Blocks until \n received. Returns length excluding \n. */
static ULONG
TestReadLine(char *Buffer, ULONG MaxLen)
{
    ULONG pos = 0;
    UCHAR byte;

    while (pos < MaxLen - 1)
    {
        if (TestReadByte(&byte))
        {
            if (byte == '\n')
                break;
            if (byte == '\r')
                continue;
            Buffer[pos++] = (char)byte;
        }
        else
        {
            /* No data — stall 1ms and retry */
            KeStallExecutionProcessor(1000);
        }
    }

    Buffer[pos] = '\0';
    return pos;
}

/* Find a test by name. Returns index or -1. */
static int
TestFindByName(const char *Name)
{
    int i;
    for (i = 0; TestTable[i].Name != NULL; i++)
    {
        if (strcmp(Name, TestTable[i].Name) == 0)
            return i;
    }
    return -1;
}

/* Run a single test by index. */
static void
TestRunOne(int idx)
{
    BOOLEAN result;

    DbgPrint("[TEST START] %s\n", TestTable[idx].Name);

    result = TestTable[idx].Func();

    if (result)
        DbgPrint("[TEST PASS] %s\n", TestTable[idx].Name);
    else
        DbgPrint("[TEST FAIL] %s: test returned FALSE\n", TestTable[idx].Name);
}

/* Print comma-separated list of all test names. */
static void
TestPrintList(void)
{
    int i;

    DbgPrint("[TEST LIST] ");
    for (i = 0; TestTable[i].Name != NULL; i++)
    {
        if (i > 0)
            DbgPrint(",");
        DbgPrint("%s", TestTable[i].Name);
    }
    DbgPrint("\n");
}

/**
 * NtTestKernelDispatchLoop - Main entry point for the test kernel.
 *
 * Called from Phase1InitializationDiscard when /TESTKERNEL is on the
 * command line. All kernel subsystems are initialized at this point.
 * Enters a serial command loop; never launches SMSS.
 */
VOID
NTAPI
NtTestKernelDispatchLoop(VOID)
{
    char line[256];
    ULONG len;

    DbgPrint("TESTKERNEL READY\n");

    for (;;)
    {
        len = TestReadLine(line, sizeof(line));

        if (len == 0)
            continue;

        /* QUIT command */
        if (strcmp(line, "QUIT") == 0)
        {
            DbgPrint("[TEST QUIT]\n");
            break;
        }

        /* LIST command */
        if (strcmp(line, "LIST") == 0)
        {
            TestPrintList();
            continue;
        }

        /* RUN <TestName> command */
        if (strncmp(line, "RUN ", 4) == 0)
        {
            const char *name = line + 4;
            int idx = TestFindByName(name);

            if (idx < 0)
            {
                DbgPrint("[TEST FAIL] %s: unknown test\n", name);
            }
            else
            {
                TestRunOne(idx);
            }
            continue;
        }

        /* Unknown command */
        DbgPrint("[TEST ERROR] Unknown command: %s\n", line);
    }
}
