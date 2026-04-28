/*
 * kdbg_rust_shims.c — outline shims for the Rust kdbg port.
 *
 * Provides:
 *   1. C-variadic functions Rust can't define on stable (KdbPrintf,
 *      KdbpPrint).
 *   2. File-private statics from the original kdb_cli.c that Rust files
 *      reference but cannot easily own (KdbDebuggerCommands and
 *      ComponentTable in particular — they hold function-pointer / macro
 *      tables that are simpler to keep in C).
 *   3. The KdbPromptStr CSTRING.
 *   4. Length symbols (KdbDebuggerCommandsCount / ComponentTableCount) so
 *      Rust loops know how many entries each table has — Rust extern decls
 *      use `[T; 0]` and `size_of_val()` would otherwise return 0.
 *
 * The original C kdb_cli.c is excluded from the build (see ntos.cmake).
 */

#include <ntoskrnl.h>
#include "kdb.h"
#include "../kd/kdterminal.h"

/* PKDBG_CLI_ROUTINE was a file-private typedef in kdb_cli.c. Re-declare it
 * here. PCH is plain `char *` from CRT. */
typedef
BOOLEAN
(NTAPI *PKDBG_CLI_ROUTINE)(
    IN PCHAR Command,
    IN ULONG Argc,
    IN PCH Argv[]);

/* Forward declarations for the !-extension routines (defined elsewhere
 * in ntoskrnl). */
BOOLEAN ExpKdbgExtPool(ULONG Argc, PCHAR Argv[]);
BOOLEAN ExpKdbgExtPoolUsed(ULONG Argc, PCHAR Argv[]);
BOOLEAN ExpKdbgExtPoolFind(ULONG Argc, PCHAR Argv[]);
BOOLEAN ExpKdbgExtFileCache(ULONG Argc, PCHAR Argv[]);
BOOLEAN ExpKdbgExtDefWrites(ULONG Argc, PCHAR Argv[]);
BOOLEAN ExpKdbgExtIrpFind(ULONG Argc, PCHAR Argv[]);
BOOLEAN ExpKdbgExtHandle(ULONG Argc, PCHAR Argv[]);

/* From debug.h via kdb_cli.c — DPFLTR_*_ID constants. */
#define KD_DEBUG_PRINT_FILTER(Name) \
    { #Name, DPFLTR_##Name##_ID }

/* -------------------------------------------------------------------------
 * Statics that Rust kdb_cli.rs references via `extern "C" { static mut X: T; }`
 * Definitions intentionally drop `static` so the symbols are externally
 * linkable. Volatile qualifiers are dropped (Rust has its own access pattern).
 * ------------------------------------------------------------------------- */

/* CLI callback slots — fixed-size array, as in the original. */
PKDBG_CLI_ROUTINE KdbCliCallbacks[10];

/* Output state / mode flags. */
BOOLEAN KdbUseIntelSyntax = TRUE;
BOOLEAN KdbBreakOnModuleLoad = FALSE;
ULONG   KdbNumberOfRowsPrinted = 0;
ULONG   KdbNumberOfColsPrinted = 0;
BOOLEAN KdbOutputAborted = FALSE;
BOOLEAN KdbRepeatLastCommand = FALSE;

/* KdbInitFileBuffer / KdbpBugCheckRequested are NOT static in C —
 * they're defined in kdb_cli.c with external linkage and referenced
 * from kdb.c and kdb_cli.c. Define them here since C kdb_cli.c is gone. */
volatile PCHAR KdbInitFileBuffer = NULL;
BOOLEAN KdbpBugCheckRequested = FALSE;

/* Dmesg ring buffer state. */
const ULONG     KdpDmesgBufferSize = 128 * 1024;
PCHAR           KdpDmesgBuffer = NULL;
volatile ULONG  KdpDmesgCurrentPosition = 0;
volatile ULONG  KdpDmesgFreeBytes = 0;
volatile ULONG  KdbDmesgTotalWritten = 0;
volatile BOOLEAN KdbpIsInDmesgMode = FALSE;
KSPIN_LOCK      KdpDmesgLogSpinLock;

/* Prompt string. */
const CSTRING KdbPromptStr = RTL_CONSTANT_STRING("kdb:> ");

/* -------------------------------------------------------------------------
 * Component table (debug filter components). Used by KdbpCmdFilter.
 * ------------------------------------------------------------------------- */

const struct
{
    PCSTR Name;
    ULONG Id;
}
ComponentTable[] =
{
    { "WIN2000", MAXULONG },
    KD_DEBUG_PRINT_FILTER(DEFAULT),
    KD_DEBUG_PRINT_FILTER(SYSTEM),
    KD_DEBUG_PRINT_FILTER(SMSS),
    KD_DEBUG_PRINT_FILTER(SETUP),
    KD_DEBUG_PRINT_FILTER(NTFS),
    KD_DEBUG_PRINT_FILTER(FSTUB),
    KD_DEBUG_PRINT_FILTER(CRASHDUMP),
    KD_DEBUG_PRINT_FILTER(CDAUDIO),
    KD_DEBUG_PRINT_FILTER(CDROM),
    KD_DEBUG_PRINT_FILTER(CLASSPNP),
    KD_DEBUG_PRINT_FILTER(DISK),
    KD_DEBUG_PRINT_FILTER(REDBOOK),
    KD_DEBUG_PRINT_FILTER(STORPROP),
    KD_DEBUG_PRINT_FILTER(SCSIPORT),
    KD_DEBUG_PRINT_FILTER(SCSIMINIPORT),
    KD_DEBUG_PRINT_FILTER(CONFIG),
    KD_DEBUG_PRINT_FILTER(I8042PRT),
    KD_DEBUG_PRINT_FILTER(SERMOUSE),
    KD_DEBUG_PRINT_FILTER(LSERMOUS),
    KD_DEBUG_PRINT_FILTER(KBDHID),
    KD_DEBUG_PRINT_FILTER(MOUHID),
    KD_DEBUG_PRINT_FILTER(KBDCLASS),
    KD_DEBUG_PRINT_FILTER(MOUCLASS),
    KD_DEBUG_PRINT_FILTER(TWOTRACK),
    KD_DEBUG_PRINT_FILTER(WMILIB),
    KD_DEBUG_PRINT_FILTER(ACPI),
    KD_DEBUG_PRINT_FILTER(AMLI),
    KD_DEBUG_PRINT_FILTER(HALIA64),
    KD_DEBUG_PRINT_FILTER(VIDEO),
    KD_DEBUG_PRINT_FILTER(SVCHOST),
    KD_DEBUG_PRINT_FILTER(VIDEOPRT),
    KD_DEBUG_PRINT_FILTER(TCPIP),
    KD_DEBUG_PRINT_FILTER(DMSYNTH),
    KD_DEBUG_PRINT_FILTER(NTOSPNP),
    KD_DEBUG_PRINT_FILTER(FASTFAT),
    KD_DEBUG_PRINT_FILTER(SAMSS),
    KD_DEBUG_PRINT_FILTER(PNPMGR),
    KD_DEBUG_PRINT_FILTER(NETAPI),
    KD_DEBUG_PRINT_FILTER(SCSERVER),
    KD_DEBUG_PRINT_FILTER(SCCLIENT),
    KD_DEBUG_PRINT_FILTER(SERIAL),
    KD_DEBUG_PRINT_FILTER(SERENUM),
    KD_DEBUG_PRINT_FILTER(UHCD),
    KD_DEBUG_PRINT_FILTER(RPCPROXY),
    KD_DEBUG_PRINT_FILTER(AUTOCHK),
    KD_DEBUG_PRINT_FILTER(DCOMSS),
    KD_DEBUG_PRINT_FILTER(UNIMODEM),
    KD_DEBUG_PRINT_FILTER(SIS),
    KD_DEBUG_PRINT_FILTER(FLTMGR),
    KD_DEBUG_PRINT_FILTER(WMICORE),
    KD_DEBUG_PRINT_FILTER(BURNENG),
    KD_DEBUG_PRINT_FILTER(IMAPI),
    KD_DEBUG_PRINT_FILTER(SXS),
    KD_DEBUG_PRINT_FILTER(FUSION),
    KD_DEBUG_PRINT_FILTER(IDLETASK),
    KD_DEBUG_PRINT_FILTER(SOFTPCI),
    KD_DEBUG_PRINT_FILTER(TAPE),
    KD_DEBUG_PRINT_FILTER(MCHGR),
    KD_DEBUG_PRINT_FILTER(IDEP),
    KD_DEBUG_PRINT_FILTER(PCIIDE),
    KD_DEBUG_PRINT_FILTER(FLOPPY),
    KD_DEBUG_PRINT_FILTER(FDC),
    KD_DEBUG_PRINT_FILTER(TERMSRV),
    KD_DEBUG_PRINT_FILTER(W32TIME),
    KD_DEBUG_PRINT_FILTER(PREFETCHER),
    KD_DEBUG_PRINT_FILTER(RSFILTER),
    KD_DEBUG_PRINT_FILTER(FCPORT),
    KD_DEBUG_PRINT_FILTER(PCI),
    KD_DEBUG_PRINT_FILTER(DMIO),
    KD_DEBUG_PRINT_FILTER(DMCONFIG),
    KD_DEBUG_PRINT_FILTER(DMADMIN),
    KD_DEBUG_PRINT_FILTER(WSOCKTRANSPORT),
    KD_DEBUG_PRINT_FILTER(VSS),
    KD_DEBUG_PRINT_FILTER(PNPMEM),
    KD_DEBUG_PRINT_FILTER(PROCESSOR),
    KD_DEBUG_PRINT_FILTER(DMSERVER),
    KD_DEBUG_PRINT_FILTER(SR),
    KD_DEBUG_PRINT_FILTER(INFINIBAND),
    KD_DEBUG_PRINT_FILTER(IHVDRIVER),
    KD_DEBUG_PRINT_FILTER(IHVVIDEO),
    KD_DEBUG_PRINT_FILTER(IHVAUDIO),
    KD_DEBUG_PRINT_FILTER(IHVNETWORK),
    KD_DEBUG_PRINT_FILTER(IHVSTREAMING),
    KD_DEBUG_PRINT_FILTER(IHVBUS),
    KD_DEBUG_PRINT_FILTER(HPS),
    KD_DEBUG_PRINT_FILTER(RTLTHREADPOOL),
    KD_DEBUG_PRINT_FILTER(LDR),
    KD_DEBUG_PRINT_FILTER(TCPIP6),
    KD_DEBUG_PRINT_FILTER(ISAPNP),
    KD_DEBUG_PRINT_FILTER(SHPC),
    KD_DEBUG_PRINT_FILTER(STORPORT),
    KD_DEBUG_PRINT_FILTER(STORMINIPORT),
    KD_DEBUG_PRINT_FILTER(PRINTSPOOLER),
    KD_DEBUG_PRINT_FILTER(VSSDYNDISK),
    KD_DEBUG_PRINT_FILTER(VERIFIER),
    KD_DEBUG_PRINT_FILTER(VDS),
    KD_DEBUG_PRINT_FILTER(VDSBAS),
    KD_DEBUG_PRINT_FILTER(VDSDYN),
    KD_DEBUG_PRINT_FILTER(VDSDYNDR),
    KD_DEBUG_PRINT_FILTER(VDSLDR),
    KD_DEBUG_PRINT_FILTER(VDSUTIL),
    KD_DEBUG_PRINT_FILTER(DFRGIFC),
    KD_DEBUG_PRINT_FILTER(MM),
    KD_DEBUG_PRINT_FILTER(DFSC),
    KD_DEBUG_PRINT_FILTER(WOW64),
    KD_DEBUG_PRINT_FILTER(ALPC),
    KD_DEBUG_PRINT_FILTER(WDI),
    KD_DEBUG_PRINT_FILTER(PERFLIB),
    KD_DEBUG_PRINT_FILTER(KTM),
    KD_DEBUG_PRINT_FILTER(IOSTRESS),
    KD_DEBUG_PRINT_FILTER(HEAP),
    KD_DEBUG_PRINT_FILTER(WHEA),
    KD_DEBUG_PRINT_FILTER(USERGDI),
    KD_DEBUG_PRINT_FILTER(MMCSS),
    KD_DEBUG_PRINT_FILTER(TPM),
    KD_DEBUG_PRINT_FILTER(THREADORDER),
    KD_DEBUG_PRINT_FILTER(ENVIRON),
    KD_DEBUG_PRINT_FILTER(EMS),
    KD_DEBUG_PRINT_FILTER(WDT),
    KD_DEBUG_PRINT_FILTER(FVEVOL),
    KD_DEBUG_PRINT_FILTER(NDIS),
    KD_DEBUG_PRINT_FILTER(NVCTRACE),
    KD_DEBUG_PRINT_FILTER(LUAFV),
    KD_DEBUG_PRINT_FILTER(APPCOMPAT),
    KD_DEBUG_PRINT_FILTER(USBSTOR),
    KD_DEBUG_PRINT_FILTER(SBP2PORT),
    KD_DEBUG_PRINT_FILTER(COVERAGE),
    KD_DEBUG_PRINT_FILTER(CACHEMGR),
    KD_DEBUG_PRINT_FILTER(MOUNTMGR),
    KD_DEBUG_PRINT_FILTER(CFR),
    KD_DEBUG_PRINT_FILTER(TXF),
    KD_DEBUG_PRINT_FILTER(KSECDD),
    KD_DEBUG_PRINT_FILTER(FLTREGRESS),
    KD_DEBUG_PRINT_FILTER(MPIO),
    KD_DEBUG_PRINT_FILTER(MSDSM),
    KD_DEBUG_PRINT_FILTER(UDFS),
    KD_DEBUG_PRINT_FILTER(PSHED),
    KD_DEBUG_PRINT_FILTER(STORVSP),
    KD_DEBUG_PRINT_FILTER(LSASS),
    KD_DEBUG_PRINT_FILTER(SSPICLI),
    KD_DEBUG_PRINT_FILTER(CNG),
    KD_DEBUG_PRINT_FILTER(EXFAT),
    KD_DEBUG_PRINT_FILTER(FILETRACE),
    KD_DEBUG_PRINT_FILTER(XSAVE),
    KD_DEBUG_PRINT_FILTER(SE),
    KD_DEBUG_PRINT_FILTER(DRIVEEXTENDER),
    KD_DEBUG_PRINT_FILTER(POWER),
    KD_DEBUG_PRINT_FILTER(CRASHDUMPXHCI),
    KD_DEBUG_PRINT_FILTER(GPIO),
    KD_DEBUG_PRINT_FILTER(REFS),
    KD_DEBUG_PRINT_FILTER(WER),
    KD_DEBUG_PRINT_FILTER(CAPIMG),
    KD_DEBUG_PRINT_FILTER(VPCI),
    KD_DEBUG_PRINT_FILTER(STORAGECLASSMEMORY),
    KD_DEBUG_PRINT_FILTER(FSLIB),
};
#undef KD_DEBUG_PRINT_FILTER

const ULONG ComponentTableCount = sizeof(ComponentTable) / sizeof(ComponentTable[0]);

/* -------------------------------------------------------------------------
 * Command table. KdbpCmd* functions are defined in Rust kdb_cli.rs and
 * exported with extern "C" + #[no_mangle] so they're plain C symbols here.
 * The conditional commands (TSS for i386 only, dt for ROS_DWARF only) are
 * conditionally compiled.
 * ------------------------------------------------------------------------- */

/* Forward declarations for KdbpCmd* functions defined in Rust kdb_cli.rs. */
BOOLEAN KdbpCmdEvalExpression(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdDisassembleX(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdRegs(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdBackTrace(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdContinue(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdStep(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdBreakPointList(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdEnableDisableClearBreakPoint(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdBreakPoint(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdThread(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdProc(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdMod(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdGdtLdtIdt(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdPcr(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdTss(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdBugCheck(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdReboot(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdFilter(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdSet(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdHelp(ULONG Argc, PCHAR Argv[]);
BOOLEAN KdbpCmdDmesg(ULONG Argc, PCHAR Argv[]);
#ifdef __ROS_DWARF__
BOOLEAN KdbpCmdPrintStruct(ULONG Argc, PCHAR Argv[]);
#endif

const struct
{
    PCHAR Name;
    PCHAR Syntax;
    PCHAR Help;
    BOOLEAN (*Fn)(ULONG Argc, PCHAR Argv[]);
} KdbDebuggerCommands[] = {
    /* Data */
    { NULL, NULL, "Data", NULL },
    { "?", "? expression", "Evaluate expression.", KdbpCmdEvalExpression },
#ifdef _M_IX86 // FIXME: this is broken on x64
    { "disasm", "disasm [address] [L count]", "Disassemble count instructions at address.", KdbpCmdDisassembleX },
#endif // _M_IX86
    { "x", "x [address] [L count]", "Display count dwords, starting at address.", KdbpCmdDisassembleX },
    { "regs", "regs", "Display general purpose registers.", KdbpCmdRegs },
    { "cregs", "cregs", "Display control, descriptor table and task segment registers.", KdbpCmdRegs },
    { "sregs", "sregs", "Display status registers.", KdbpCmdRegs },
    { "dregs", "dregs", "Display debug registers.", KdbpCmdRegs },
    { "bt", "bt [*frameaddr|thread id]", "Prints current backtrace or from given frame address.", KdbpCmdBackTrace },
#ifdef __ROS_DWARF__
    { "dt", "dt [mod] [type] [addr]", "Print a struct. The address is optional.", KdbpCmdPrintStruct },
#endif
    /* Flow control */
    { NULL, NULL, "Flow control", NULL },
    { "cont", "cont", "Continue execution (leave debugger).", KdbpCmdContinue },
    { "step", "step [count]", "Execute single instructions, stepping into interrupts.", KdbpCmdStep },
    { "next", "next [count]", "Execute single instructions, skipping calls and reps.", KdbpCmdStep },
    { "bl", "bl", "List breakpoints.", KdbpCmdBreakPointList },
    { "be", "be [breakpoint]", "Enable breakpoint.", KdbpCmdEnableDisableClearBreakPoint },
    { "bd", "bd [breakpoint]", "Disable breakpoint.", KdbpCmdEnableDisableClearBreakPoint },
    { "bc", "bc [breakpoint]", "Clear breakpoint.", KdbpCmdEnableDisableClearBreakPoint },
    { "bpx", "bpx [address] [IF condition]", "Set software execution breakpoint at address.", KdbpCmdBreakPoint },
    { "bpm", "bpm [r|w|rw|x] [byte|word|dword] [address] [IF condition]", "Set memory breakpoint at address.", KdbpCmdBreakPoint },

    /* Process/Thread */
    { NULL, NULL, "Process/Thread", NULL },
    { "thread", "thread [list[ pid]|[attach ]tid]", "List threads in current or specified process, display thread with given id or attach to thread.", KdbpCmdThread },
    { "proc", "proc [list|[attach ]pid]", "List processes, display process with given id or attach to process.", KdbpCmdProc },

    /* System information */
    { NULL, NULL, "System info", NULL },
    { "mod", "mod [address]", "List all modules or the one containing address.", KdbpCmdMod },
    { "gdt", "gdt", "Display the global descriptor table.", KdbpCmdGdtLdtIdt },
    { "ldt", "ldt", "Display the local descriptor table.", KdbpCmdGdtLdtIdt },
    { "idt", "idt", "Display the interrupt descriptor table.", KdbpCmdGdtLdtIdt },
    { "pcr", "pcr", "Display the processor control region.", KdbpCmdPcr },
#ifdef _M_IX86
    { "tss", "tss [selector|*descaddr]", "Display the current task state segment, or the one specified by its selector number or descriptor address.", KdbpCmdTss },
#endif

    /* Others */
    { NULL, NULL, "Others", NULL },
    { "bugcheck", "bugcheck", "Bugchecks the system.", KdbpCmdBugCheck },
    { "reboot", "reboot", "Reboots the system.", KdbpCmdReboot},
    { "filter", "filter [error|warning|trace|info|level]+|-[componentname|default]", "Enable/disable debug channels.", KdbpCmdFilter },
    { "set", "set [var] [value]", "Sets var to value or displays value of var.", KdbpCmdSet },
    { "dmesg", "dmesg", "Display debug messages on screen, with navigation on pages.", KdbpCmdDmesg },
    { "kmsg", "kmsg", "Kernel dmesg. Alias for dmesg.", KdbpCmdDmesg },
    { "help", "help", "Display help screen.", KdbpCmdHelp },
    { "!pool", "!pool [Address [Flags]]", "Display information about pool allocations.", ExpKdbgExtPool },
    { "!poolused", "!poolused [Flags [Tag]]", "Display pool usage.", ExpKdbgExtPoolUsed },
    { "!poolfind", "!poolfind Tag [Pool]", "Search for pool tag allocations.", ExpKdbgExtPoolFind },
    { "!filecache", "!filecache", "Display cache usage.", ExpKdbgExtFileCache },
    { "!defwrites", "!defwrites", "Display cache write values.", ExpKdbgExtDefWrites },
    { "!irpfind", "!irpfind [Pool [startaddress [criteria data]]]", "Lists IRPs potentially matching criteria.", ExpKdbgExtIrpFind },
    { "!handle", "!handle [Handle]", "Displays info about handles.", ExpKdbgExtHandle },
};

const ULONG KdbDebuggerCommandsCount = sizeof(KdbDebuggerCommands) / sizeof(KdbDebuggerCommands[0]);

/* -------------------------------------------------------------------------
 * Out-of-line wrappers for FORCEINLINE / macro-like helpers from
 * intrin_i.h that Rust references via extern "C". These have no exported
 * symbols of their own in the C build (they're inlined), so we provide
 * thin wrappers here.
 * ------------------------------------------------------------------------- */

/* -------------------------------------------------------------------------
 * Macro helpers from kdb_cli.c that the Rust port references via extern
 * decls. Provide thin out-of-line wrappers here so Rust can call them.
 * ------------------------------------------------------------------------- */

PCHAR
__cdecl
KDB_ACCESS_TYPE_TO_STRING(KDB_ACCESS_TYPE Type)
{
    return (Type == KdbAccessRead) ? "read" :
           (Type == KdbAccessWrite) ? "write" :
           (Type == KdbAccessReadWrite) ? "rdwr" : "exec";
}

PCHAR
__cdecl
KDB_ENTER_CONDITION_TO_STRING(KDB_ENTER_CONDITION cond)
{
    return (cond == KdbDoNotEnter) ? "never" :
           (cond == KdbEnterAlways) ? "always" :
           (cond == KdbEnterFromKmode) ? "kmode" : "umode";
}

PSTR
__cdecl
NPX_STATE_TO_STRING(ULONG state)
{
    /* From psfuncs.h: NPX_STATE_LOADED = 0, NPX_STATE_NOT_LOADED = 0x10. */
    return (state == 0) ? "Loaded" :
           (state == 0x10) ? "Not loaded" : "Unknown";
}


/* Ke386GetGlobalDescriptorTable / Ke386GetLocalDescriptorTable are macros
 * for __sgdt / __sldt in intrin_i.h. __sgdt is itself a FORCEINLINE on
 * i386, exported via shim_exports.rs on amd64. Provide out-of-line
 * wrappers here for Rust to link to. We undef any pre-existing macros. */
#undef Ke386GetGlobalDescriptorTable
#undef Ke386GetLocalDescriptorTable

VOID
__cdecl
Ke386GetGlobalDescriptorTable(_Out_ PVOID Descriptor)
{
    __sgdt(Descriptor);
}

VOID
__cdecl
Ke386GetLocalDescriptorTable(_Out_ PUSHORT Limit)
{
    __sldt(Limit);
}

#ifdef _M_IX86

USHORT
__cdecl
Ke386GetTr_Shim(VOID)
{
    USHORT Tr;
    __asm__("str %0\n\t" : "=m"(Tr));
    return Tr;
}
/* Real Ke386GetTr() is a FORCEINLINE in intrin_i.h. Rust calls it via
 * `Ke386GetTr` - alias the shim under that name. */
USHORT __cdecl __attribute__((alias("Ke386GetTr_Shim"))) Ke386GetTr(VOID);

/* FORCEINLINE accessors from internal/i386/ke.h. Rust calls these via
 * extern "C" — provide out-of-line wrappers. */
ULONG_PTR
__cdecl
KeGetContextFrameRegister(PCONTEXT Context)
{
    return Context->Ebp;
}

VOID
__cdecl
KeSetContextFrameRegister(PCONTEXT Context, ULONG_PTR Frame)
{
    Context->Ebp = Frame;
}

ULONG_PTR
__cdecl
KeGetTrapFrameFrameRegister(PKTRAP_FRAME TrapFrame)
{
    return TrapFrame->Ebp;
}

ULONG_PTR
__cdecl
KeGetTrapFrameStackRegister(PKTRAP_FRAME TrapFrame)
{
    if (TrapFrame->PreviousPreviousMode == KernelMode)
        return TrapFrame->TempEsp;
    return TrapFrame->HardwareEsp;
}
#endif /* _M_IX86 */

/* amd64 versions */
#ifdef _M_AMD64
ULONG_PTR
__cdecl
KeGetContextFrameRegister(PCONTEXT Context)
{
    return Context->Rbp;
}

VOID
__cdecl
KeSetContextFrameRegister(PCONTEXT Context, ULONG_PTR Frame)
{
    Context->Rbp = Frame;
}

ULONG_PTR
__cdecl
KeGetTrapFrameFrameRegister(PKTRAP_FRAME TrapFrame)
{
    return TrapFrame->Rbp;
}

ULONG_PTR
__cdecl
KeGetTrapFrameStackRegister(PKTRAP_FRAME TrapFrame)
{
    return TrapFrame->Rsp;
}
#endif /* _M_AMD64 */

/* -------------------------------------------------------------------------
 * Variadic functions. Stable Rust can't define C-variadic exported fns,
 * so they live here. Both delegate to Rust-side helpers (KdbPutsN /
 * KdbpPagerInternal) for the actual output.
 * ------------------------------------------------------------------------- */

VOID KdbpPagerInternal(IN PCHAR Buffer, IN ULONG BufLength, IN BOOLEAN DoPage);

VOID
__cdecl
KdbPrintf(
    _In_ PCSTR Format,
    ...)
{
    va_list ap;
    SIZE_T Length;
    CHAR Buffer[1024];

    va_start(ap, Format);
    Length = _vsnprintf(Buffer, sizeof(Buffer), Format, ap);
    Length = min(Length, MAXUSHORT - sizeof(ANSI_NULL));
    va_end(ap);

    KdbPutsN(Buffer, (USHORT)Length);
}

VOID
KdbpPrint(
    _In_ PSTR Format,
    _In_ ...)
{
    static CHAR Buffer[4096];
    ULONG Length;
    va_list ap;

    if (KdbOutputAborted)
        return;

    va_start(ap, Format);
    Length = _vsnprintf(Buffer, sizeof(Buffer) - 1, Format, ap);
    Buffer[Length] = '\0';
    va_end(ap);

    KdbpPagerInternal(Buffer, Length, FALSE);
}
