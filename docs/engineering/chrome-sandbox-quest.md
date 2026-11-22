# Chrome sandbox quest

The contract, the mechanism and the oracles are in
[modern-windows-compat-roadmap.md](modern-windows-compat-roadmap.md), "The sandbox". This
file tracks the walk: each step is the first thing that stopped the sandboxed browser,
how it was found, and what was done.

## How to take the next step

The Chrome payload's **Run S** (`tools/payloads/chrome/commands.txt`) is Run M with the
sandbox on. Every boot of `./vibeos-cli smoke --arch amd64 --payload chrome` runs it and
then `catfile` (Chrome's log) and `dumpinfo` (the Crashpad report). Read, in order:

1. `exit chrome ... chrome_s.log code=` — `0x80000003` is a CHECK or `ImmediateCrash`.
2. The log's last `FATAL`. A sandbox launch failure prints a `ResultCode` number; decode it
   in Chromium's `sandbox/win/src/sandbox_types.h` (explicit `= N` values).
3. No FATAL: `dumpinfo`'s `exception ... addr=chrome.exe+0x...`. Chrome's sandbox lives in
   chrome.exe; disassemble the local copy around that address
   (`x86_64-w64-mingw32-objdump -d --start-address=...`) and read the strings the failing
   branch passes. Code `0x0517a7ed` is Crashpad's dump-without-crashing, not the crash.

Probes beside it pin each step down on its own: `acgprobe`, `tokprobe`.

## Steps

1. **ACG** (fixed 2026-11-22). The browser turns on `ProhibitDynamicCode` +
   `AllowThreadOptOut` for itself at start and CHECKs it (`process_mitigations.cc:88`,
   Policy 2). Now enforced in MM with a same-thread opt-out (ps-rust `mitigation.rs`);
   `acgprobe` checks what it must and must not block.
2. **`kernelbase.dll!AppContainerRegisterSid`** (fixed 2026-11-22). An int3 at
   chrome.exe+0xad800: `GetProcAddress(GetModuleHandle(L"kernelbase.dll"),
   "AppContainerRegisterSid")` CHECKed non-null. The export now exists and refuses
   (`HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED)`: no lowbox tokens), which the caller treats
   as "no AppContainer". kernelbase was not loaded at all, so kernel32 now loads it into
   every Win32 process, by full path from its own directory (a WoW64 process's 64-bit
   kernel32 otherwise found the 32-bit one in syswow64).
3. **Restricted token: handle access** (fixed 2026-11-22). The browser now starts its
   Network Service process; the GPU process (software, even with `--disable-gpu`) failed
   to launch three times with ResultCode 49, `SBOX_ERROR_CANNOT_CREATE_RESTRICTED_TOKEN`.
   `tokprobe`: every `CreateRestrictedToken` shape succeeded, but the new handle could not
   be queried or duplicated. NtFilterToken (and NtDuplicateToken) passed a null
   `OBJECT_HANDLE_INFORMATION` where the C passes a stack struct, so the handle had access
   0. Fixed; lint rule `scan_null_out_param`.
4. **Integrity levels** (open, next). Still ResultCode 49: `tokprobe` shows
   `TokenIntegrityLevel` query and set, and `TokenLogonSid`, return
   ERROR_INVALID_PARAMETER. Our TOKEN has no label at all (no `SE_GROUP_INTEGRITY` group,
   no `IntegrityLevelIndex`). Needed: logon tokens carry Medium and SYSTEM carries System as
   an integrity group, query/set of the label (a set grows the token's variable part), and
   then the access check's no-write-up rule. Per the roadmap, until the access check
   enforces it a set label must say so loudly, never read as a working sandbox.

Running the payload: use the network (no `--no-net`). Without it the sweep can deadlock
after a Chrome hang (registry: "dhcpcsvc ApiCriticalSection deadlock").

Also seen, not yet a blocker: `windows_version.cc: Unsupported version: 42.4.17`.
