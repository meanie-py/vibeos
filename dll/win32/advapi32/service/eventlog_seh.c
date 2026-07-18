/*
 * eventlog_seh.c — native-SEH wrappers for the EventLog RPC client stubs.
 *
 * The advapi32-rust cohab (src/service/eventlog.rs) translated eventlog.c but
 * DROPPED the RpcTryExcept/RpcExcept the C original wrapped around every RPC
 * client-stub call (see service/eventlog.c, e.g. ElfRegisterEventSourceW at
 * ~line 1276). Rust has no __try/__except, so when the EventLog RPC server is
 * absent the stub's RPC_S_SERVER_UNAVAILABLE raise escaped and killed the
 * process: on the arm64 minimal-SCM boot, services.exe died servicing a
 * CreateService at ScmLoadService with 0x6BA (=1722, RPC_S_SERVER_UNAVAILABLE)
 * instead of logging-fail gracefully. (Latent on i386/amd64 too — it just never
 * fires there because the EventLog service is present on the desktop boot.)
 *
 * These thunks restore the catch at a native-SEH C boundary. Compiled with
 * _USE_NATIVE_SEH so RpcTryExcept -> _SEH2_TRY -> a *real* __try: pseh2.h checks
 * _USE_NATIVE_SEH BEFORE the _USE_DUMMY_PSEH default (the global C compile mode),
 * so this TU gets native SEH while the rest of advapi32 keeps DUMMY. The
 * "calls-only" native-SEH contract (dummy-pseh-verdict.md §C / ntoskrnl/
 * kmsafecopy.c) holds trivially here: the __try body is nothing but the RPC stub
 * call, so the raise lands one frame down in a callee whose return address sits
 * inside this scope table. And this is a SOFTWARE raise (RpcRaiseException), which
 * clang native SEH catches on amd64 AND arm64 — kmsafecopy's arm64 *hardware*-
 * page-fault gap (field manual §9.69) does NOT apply. Rust (eventlog.rs) calls
 * these *_seh thunks instead of the raw Elfr* stubs.
 *
 * i386: NOT opted into _USE_NATIVE_SEH. Clang native SEH is unreliable on i386
 * (ESP-relative filter locals — the very reason USE_DUMMY_PSEH is the i386
 * default), and the kmtest native-SEH precedent likewise excludes i386. The #else
 * pass-through keeps i386 behaviour identical to the DUMMY_PSEH C original (whose
 * RpcTryExcept was also a dead handler there). The residual server-absent gap is
 * i386-only and matches the existing task #5 SEH gap — honest and documented, not
 * hidden. Rust still routes through the thunk uniformly on all three arches.
 */

#include <advapi32.h>
#include <ndk/kefuncs.h>
#include <eventlogrpc_c.h>

#if defined(_USE_NATIVE_SEH)

NTSTATUS __stdcall
ElfrRegisterEventSourceW_seh(
    EVENTLOG_HANDLE_W UNCServerName,
    PRPC_UNICODE_STRING ModuleName,
    PRPC_UNICODE_STRING RegModuleName,
    ULONG MajorVersion,
    ULONG MinorVersion,
    PIELF_HANDLE LogHandle)
{
    NTSTATUS Status;

    RpcTryExcept
    {
        Status = ElfrRegisterEventSourceW(UNCServerName,
                                          ModuleName,
                                          RegModuleName,
                                          MajorVersion,
                                          MinorVersion,
                                          LogHandle);
    }
    RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = I_RpcMapWin32Status(RpcExceptionCode());
    }
    RpcEndExcept;

    return Status;
}

NTSTATUS __stdcall
ElfrReportEventW_seh(
    IELF_HANDLE LogHandle,
    ULONG Time,
    USHORT EventType,
    USHORT EventCategory,
    ULONG EventID,
    USHORT NumStrings,
    ULONG DataSize,
    PRPC_UNICODE_STRING ComputerName,
    PRPC_SID UserSID,
    PRPC_UNICODE_STRING Strings[],
    PBYTE Data,
    USHORT Flags,
    PULONG RecordNumber,
    PULONG TimeWritten)
{
    NTSTATUS Status;

    RpcTryExcept
    {
        Status = ElfrReportEventW(LogHandle,
                                  Time,
                                  EventType,
                                  EventCategory,
                                  EventID,
                                  NumStrings,
                                  DataSize,
                                  ComputerName,
                                  UserSID,
                                  Strings,
                                  Data,
                                  Flags,
                                  RecordNumber,
                                  TimeWritten);
    }
    RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = I_RpcMapWin32Status(RpcExceptionCode());
    }
    RpcEndExcept;

    return Status;
}

NTSTATUS __stdcall
ElfrDeregisterEventSource_seh(PIELF_HANDLE LogHandle)
{
    NTSTATUS Status;

    RpcTryExcept
    {
        Status = ElfrDeregisterEventSource(LogHandle);
    }
    RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = I_RpcMapWin32Status(RpcExceptionCode());
    }
    RpcEndExcept;

    return Status;
}

#else /* !_USE_NATIVE_SEH — i386: pass through (no reliable clang native SEH) */

NTSTATUS __stdcall
ElfrRegisterEventSourceW_seh(
    EVENTLOG_HANDLE_W UNCServerName,
    PRPC_UNICODE_STRING ModuleName,
    PRPC_UNICODE_STRING RegModuleName,
    ULONG MajorVersion,
    ULONG MinorVersion,
    PIELF_HANDLE LogHandle)
{
    return ElfrRegisterEventSourceW(UNCServerName, ModuleName, RegModuleName,
                                    MajorVersion, MinorVersion, LogHandle);
}

NTSTATUS __stdcall
ElfrReportEventW_seh(
    IELF_HANDLE LogHandle,
    ULONG Time,
    USHORT EventType,
    USHORT EventCategory,
    ULONG EventID,
    USHORT NumStrings,
    ULONG DataSize,
    PRPC_UNICODE_STRING ComputerName,
    PRPC_SID UserSID,
    PRPC_UNICODE_STRING Strings[],
    PBYTE Data,
    USHORT Flags,
    PULONG RecordNumber,
    PULONG TimeWritten)
{
    return ElfrReportEventW(LogHandle, Time, EventType, EventCategory, EventID,
                            NumStrings, DataSize, ComputerName, UserSID, Strings,
                            Data, Flags, RecordNumber, TimeWritten);
}

NTSTATUS __stdcall
ElfrDeregisterEventSource_seh(PIELF_HANDLE LogHandle)
{
    return ElfrDeregisterEventSource(LogHandle);
}

#endif
