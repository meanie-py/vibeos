# kernel32-rust / proc.c — hand-translation manifest (shared contract)

The C source is `dll/win32/kernel32/client/proc.c` (4841 lines) in this worktree
(`/home/curi/Projects/os-playground/vibeos-wt-kernel32`). The crate **foundation
is already built and compiles ×3** — DO NOT touch it. You translate one body
module to a `/tmp` file; the integrator wires it in and builds. Do not build, do
not edit any other file.

## What `use crate::substrate::*;` already gives you (the frozen contract)

- **`ntdll_sys::nt::*`** — every `Nt*`/`Zw*`/`Rtl*`/`Ldr*`/`Dbg*`/`Csr*` function
  and all NT/Win32 types (PEB, TEB, UNICODE_STRING, ANSI_STRING, OBJECT_ATTRIBUTES,
  CLIENT_ID, IO_STATUS_BLOCK, STARTUPINFOW/A, PROCESS_BASIC_INFORMATION,
  QUOTA_LIMITS_EX, KERNEL_USER_TIMES, SECTION_IMAGE_INFORMATION, INITIAL_TEB,
  CONTEXT, PROCESS_PRIORITY_CLASS, RTL_USER_PROCESS_PARAMETERS, LIST_ENTRY,
  LARGE_INTEGER, RTL_CRITICAL_SECTION, RTL_QUERY_REGISTRY_TABLE, BASE_API_MESSAGE
  + the `Data` union (CreateProcessRequest/CheckVDMRequest/ShutdownParametersRequest/
  ExitProcessRequest), HANDLE/DWORD/ULONG/BOOL/BOOLEAN/PVOID/LPWSTR/... windows
  typedefs) and **all `STATUS_*` / `NTSTATUS` constants** and the
  `NtCurrentPeb()/NtCurrentTeb()/NtCurrentProcess()/NtCurrentThread()` inlines and
  `InitializeListHead/IsListEmpty/InsertTailList/RemoveEntryList/...` inlines.
  - `NT_SUCCESS(s)` may NOT exist as a fn — use `s >= 0` (NTSTATUS is i32; success
    is `>= 0`). Define a local `#[inline] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }`
    at the top of your module if you use it a lot.
- **k32-internal types** (`crate::k32_types`): `WaitForInputIdleType`,
  `PPROCESS_START_ROUTINE`, `PBASEP_APPCERT_PLUGIN_FUNC`,
  `PBASEP_APPCERT_EMBEDDED_FUNC`, `PSAFER_REPLACE_PROCESS_THREAD_TOKENS`,
  `BASEP_APPCERT_ENTRY`/`PBASEP_APPCERT_ENTRY`, `BASE_MSG_SXS_HANDLES`, and
  constants `VDM_UNDO_*`, `BINARY_TYPE_*`, `VDM_NOT_LOADED/NOT_READY/READY`,
  `VdmEntryUndo`/`VdmEntryUpdateProcess`.
- **cohab imports** (`crate::k32_externs`): the `Base*` helpers in the other
  kernel32 .c files (`BaseSetLastNTError`, `BaseFormatObjectAttributes`,
  `BaseCreateStack`, `BaseInitializeContext`, `BasepIsRealtimeAllowed`,
  `BaseComputeProcessDllPath/ExePath`, `BaseIsDosApplication`, `BasepCheckBadapp`,
  `BasepFreeAppCompatData`, `BasepCheckWinSaferRestrictions`,
  `BasepAnsiStringToHeapUnicodeString`, `Basep8BitStringToDynamicUnicodeString`,
  all the `BaseCheckVDM/BaseCreateVDMEnvironment/...` VDM fns), the cross-file
  Win32 exports (`GetFileAttributesW`, `SearchPathW`, `CreateFileW`, `CloseHandle`,
  `GetLastError`, `SetLastError`, `GetModuleHandleW`, `GetProcAddress`,
  `LoadLibraryW`, `GetSystemDirectoryW`, `GetFullPathNameW`, `ExitThread`,
  `UnhandledExceptionFilter`, `IsBadWritePtr`), `RtlGetProcessHeap`, the extern
  data `BaseStaticServerData`/`BaseRunningInServerProcess`
  (+`BaseCsrConnectionEstablished` arm64), the console consts
  `HANDLE_DETACHED_PROCESS/CREATE_NEW_CONSOLE/CREATE_NO_WINDOW`, and the macro
  inlines:
  - `SharedUserData->X`  →  `SharedUserData_NtMajorVersion()`,
    `_NtMinorVersion()`, `_SuiteMask()`, `_ImageNumberLow()`, `_ImageNumberHigh()`
    (all `unsafe fn`). For any OTHER SharedUserData field, STOP and tell the
    integrator — do not invent an offset.
  - `IsConsoleHandle(h)` → `IsConsoleHandle(h) -> bool`
  - `AddToHandle(x,y)`/`RemoveFromHandle(x,y)` → **return a value** (C mutates in
    place): write `h = AddToHandle(h, 2);`
  - `MAKELONG`, `HandleToUlong`, `UlongToHandle`, `InterlockedCompareExchangePointer`.
- **proc.c globals** (`crate::k32_globals`, all `static mut`, access in `unsafe`):
  `UserWaitForInputIdleRoutine`, `BasepExeLdrEntry`, `BasepAppCertDllsList`,
  `gcsAppCert`, `BasePathVariableName`, `BaseUnicodeCommandLine`,
  `BaseAnsiCommandLine`, `BaseAnsiStartupInfo`, `g_AppCertInitialized`,
  `g_HaveAppCerts`, `fEmbeddedCertFunc`, `g_AppCertStatus`,
  `g_SaferReplaceProcessThreadTokens`, `gSaferHandle`.

## Calling a proc.c function that another module owns

Every proc.c function is `#[no_mangle] pub unsafe extern "system" fn`. To call one
that lives in another module, use its crate path:
- `crate::proc_getters::{ReadProcessMemory, ExitProcess, OpenProcess, FatalAppExitW}`
- `crate::proc_helpers::{StuffStdHandle, BuildSubSysCommandLine, BasepIsImageVersionOk,
  BasepCheckWebBladeHashes, BasepIsProcessAllowed, BasepReplaceProcessThreadTokens}`
- `crate::proc_params::BasePushProcessParameters`

## Signature rule

Every exported function is `#[no_mangle] pub unsafe extern "system" fn NAME(...) -> RET`
(WINAPI and NTAPI both → `extern "system"`; correct i386 `_NAME@n` decoration is
automatic). VOID → no return. The single `__cdecl` thing is the
`WaitForInputIdleType` callback (already typed). Match the C parameter order/types;
use the windows typedefs from the substrate (HANDLE, DWORD, LPDWORD, PSIZE_T,
PDWORD_PTR, LPFILETIME, PIO_COUNTERS, PBOOL, LPCVOID, UINT, LPCSTR, LPCWSTR,
LPSTARTUPINFOW/A, LPPROCESS_INFORMATION, PHANDLE, LPSECURITY_ATTRIBUTES, ...).
`int` → `core::ffi::c_int`.

## PSEH / control-flow lowering (DUMMY_PSEH — all arches)

- `_SEH2_TRY { A } _SEH2_EXCEPT(filter) { B } _SEH2_END;`  → just emit `A`
  (the EXCEPT handler `B` is dead code under DUMMY_PSEH). Validate pointers before
  deref as usual.
- `_SEH2_TRY { A } _SEH2_FINALLY { C } _SEH2_END;` → run `A` then `C` on the normal
  path. Model as `'try_blk: { A }  C` with `_SEH2_LEAVE` → `break 'try_blk`.
  `_SEH2_AbnormalTermination()` → `false`.
- `_SEH2_YIELD(return X)` → `return X`.
- `_SEH2_GetExceptionCode()`/`GetExceptionInformation()` appear only inside dead
  EXCEPT arms — drop with the arm.
- C `goto Label;` → Rust labeled block + `break`. The big function
  `CreateProcessInternalW` has labels `AppNameRetry`, `StartScan` (forward+back
  jumps → model the retry as `'app_name_retry: loop { ... }` with
  `continue 'app_name_retry` for `goto AppNameRetry` and an inner
  `'start_scan: loop`/restructure for `goto StartScan`), `VdmShortCircuit`, and
  `Quickie` (cleanup — model the whole body as `'quickie: { ... }` and
  `goto Quickie` → `break 'quickie`, with the cleanup code after the block).
  `BasePushProcessParameters` has `Quickie`/`FailPath` (FailPath sets error then
  `goto Quickie`; model as a closure or `'quickie:` block + a `fail` path).
  `GetProcessVersion` has an inner `Error:` label inside the (FINALLY) try.
- `ASSERT(x)` → `debug_assert!(x)` (or drop if it references dead state).
- `DPRINT`/`DPRINT1` are compiled out (`#define NDEBUG`) → **drop them**. The two
  direct `DbgPrint(...)`/`DbgPrint` calls (the "BASE: Calling debugger" line and
  the "No mapping for ImageInformation.Machine" line) are cosmetic → drop them
  (leave a `// DbgPrint: ...` comment).
- Wide string literals `L"..."` → build a `[u16; N]` array (NUL-terminated) and
  pass `.as_ptr()`; for `RtlInitUnicodeString(&s, L"x")` you need a static/local
  UTF-16 buffer. `wcscpy/wcscat/wcslen/_wcsnicmp` are not in substrate — implement
  tiny local helpers (operate on `*mut u16`/`*const u16`) at the top of your module
  if you need them, OR prefer the `Rtl*` string APIs where the C does.

## OUTPUT RULES
- Write ONLY your one file to the given `/tmp` path. No other files. Do not run
  cargo/build. Return a 2-line summary (functions done, any STOP items) as your
  final message — raw, no prose preamble.
- Top of file: `use crate::substrate::*;` plus any cross-module `use crate::proc_*`
  lines you need. Crate-level `#![allow(...)]` already covers naming/unused.
- Faithful translation: preserve the C logic exactly, including the error-return
  idiom `BaseSetLastNTError(Status); return FALSE;`. Don't "improve" it.
