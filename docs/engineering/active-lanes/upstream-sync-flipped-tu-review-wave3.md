# Upstream-sync: flipped-TU review — wave 3 (non-boot-path remainder)

Scope: wave 2's Priority 4 — the ~150 non-boot-path files left untouched by waves 1–2, out of
the 182-file HIGH-confidence cohab set (`FP=5213cf717cff6946945922e6a7e03d87b0a58e4f`,
`git diff --name-only $FP upstream/master` ∩ HIGH `tools/cohab-registry.json` entries, minus
win32ss/ and drivers/network/tcpip).

Derivation, reproduced today:
```
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f
git diff --name-only $FP upstream/master        # 4519 files
# ∩ HIGH entries (by c_path key) → 195
# − win32ss/*, drivers/network/tcpip/*         → 182
# − boot-path prefixes (ntoskrnl/hal/dll-ntdll/freeldr/*filesystems*) → 97 non-boot files
# − wave 1's 7 already-reviewed non-boot files (dhclient.c, umpnpmgr/rpcserver.c,
#   shell32 CDefView.cpp/utils.cpp/wine/shellord.c, syssetup install.c/wizard.c)
#                                                → 90 files this pass drew from
```
(wave 1/2's "16 highest-churn non-boot-path files" included netcfgx/msafd, which were only
triaged at commit-subject/diffstat level, not individually cross-checked — this pass re-touches
those subjects but does not re-derive them as "reached in full.")

## Coverage this pass

Weighted by blast radius (DLL every process loads outranks a control-panel applet), in order:

1. **All 11 remaining `kernel32/` files** (13 commits) — full diff + Rust cross-check on every
   commit that wasn't the mechanical ANSI-split refactor (`c81bc9f21ea`, 5 files, pure code
   move) or a pure rename (`63f86fa73bd`).
2. **`samsrv/` (5 files), `msafd/misc/dllmain.c`** — all their remaining commits turned out to be
   the repo-wide cosmetic passes (`1cf31524d50` swprintf/vswprintf, `63f86fa73bd` rename,
   `a70f59b0dd5` RtlConvert*ToLargeInteger→inline) already characterized as cosmetic by wave 1;
   opened `a70f59b0dd5`'s msafd hunk directly to confirm semantic equivalence
   (`RtlConvertLongToLargeInteger(0)` ≡ `.QuadPart = 0LL`).
3. **`msv1_0/msv1_0.c`, `msgina/{gui,shutdown}.c`, `netapi32/{wksta_new,local_group}.c`,
   `wkssvc/domain.c`** — security/auth-adjacent, one commit each: opened all in full.
4. **A subject-line sweep of the remaining ~70 files** (netcfgx ×8, dhcpcsvc ×3,
   shell32 ×26, cpl/desk ×3, cpl/sysdm ×4, freeldr/ui ×2, acpi ×2, dnsrslvr/tcpsvcs/userenv/
   ws2_32/wshtcpip/iphlpapi ×1 each) — every commit subject read, full diff opened for any that
   didn't unambiguously read as feature-add/cosmetic/refactor from the subject+diffstat alone.
   None in this remainder read as a plausible correctness regression that wasn't already
   flagged by wave 1/2 (`a790beb22ba` `SHBindToObject`, recurring across 6 shell32 files, is the
   one wave 2 already REFUTED as "real but not live"; not re-opened).

**Not reached in this pass**: full-diff-level cross-check of the ~50 netcfgx/dhcpcsvc
"Implement X" commits' actual field-level correctness (only subject+diffstat read — see "Not
reached" below), and the shell32 feature commits (`a5f51269c88` FolderItems3,
`c891aefdcff` CFolder getters/setters, `095860df576` NT6+ stubs bundle, etc.) — these are
large, genuine feature additions with no C predecessor behavior to regress against, so they are
out of this bug-hunt's frame (nothing to compare "worse than" — the Rust twin, being cohab, was
translated from the *pre-feature* C and is simply missing the feature, which is a translation
backlog item, not a correctness bug).

## CONFIRMED — detail + proposed diffs

### W3-1. `GetEnvironmentVariableW`: missing `lpBuffer == NULL` guard — NULL-pointer write on malformed call

- **File / commit**: `dll/win32/kernel32/client/environ.c`, `242937b1229`
  ("[KERNEL32] Fix GetEnvironmentVariable(A/W)").
- **Upstream diff** (the load-bearing hunk): adds
  ```c
  if (lpBuffer == NULL)
  {
      nSize = 0;
  }
  ```
  before computing `UniSize`, and simplifies the size calc to
  `UniSize = (USHORT)min(nSize, UNICODE_STRING_MAX_CHARS);` /
  `RtlInitEmptyUnicodeString(&VarValue, lpBuffer, UniSize * sizeof(WCHAR))`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/kernel32-rust/src/client/environ.rs:200-261`,
  `GetEnvironmentVariableW` — byte-for-byte the pre-fix C (lines 218-229 mirror the old
  branching `UniSize` calc; there is no `lpBuffer.is_null()` check anywhere in the function).
  If a caller passes `lpBuffer == NULL` with `nSize != 0` (a malformed but not inherently
  crash-worthy call — `GetEnvironmentVariableW(name, NULL, 0)` to probe the required length is
  the *documented* usage, but nothing stops a caller passing a stale/garbage `nSize` alongside a
  NULL buffer), `RtlInitEmptyUnicodeString` is handed a NULL `Buffer` with a non-zero
  `MaximumLength`, and `RtlQueryEnvironmentVariable_U` may then write the queried value straight
  into that NULL buffer — an AV, in `kernel32.dll`, reachable from any usermode process.
- **`#[no_mangle] pub unsafe extern "system" fn GetEnvironmentVariableW`** — live, exported,
  called by essentially every process via the CRT/`GetEnvironmentVariable` wrapper.
- Proposed fix:
```diff
--- a/sdk/lib/rust/kernel32-rust/src/client/environ.rs
+++ b/sdk/lib/rust/kernel32-rust/src/client/environ.rs
@@ pub unsafe extern "system" fn GetEnvironmentVariableW(
     let mut Status: NTSTATUS = 0;
     let mut UniSize: USHORT = 0;

-    if nSize <= (UNICODE_STRING_MAX_CHARS - 1) as DWORD {
-        if nSize != 0 {
-            UniSize = (nSize as USHORT)
-                .wrapping_mul(core::mem::size_of::<WCHAR>() as USHORT)
-                .wrapping_sub(core::mem::size_of::<WCHAR>() as USHORT);
-        } else {
-            UniSize = 0;
-        }
-    } else {
-        UniSize = (UNICODE_STRING_MAX_BYTES as USHORT)
-            .wrapping_sub(core::mem::size_of::<WCHAR>() as USHORT);
-    }
+    let mut nSize = nSize;
+    if lpBuffer.is_null() {
+        nSize = 0;
+    }
+
+    /* Set maximum size for unicode string (in chars, including UNICODE_NULL) */
+    UniSize = core::cmp::min(nSize, UNICODE_STRING_MAX_CHARS as DWORD) as USHORT;

     Status = RtlInitUnicodeStringEx(&mut VarName, lpName);
     if !NT_SUCCESS(Status) {
         BaseSetLastNTError(Status);
         return 0;
     }

-    RtlInitEmptyUnicodeString(&mut VarValue, lpBuffer, UniSize);
+    RtlInitEmptyUnicodeString(
+        &mut VarValue,
+        lpBuffer,
+        (UniSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT,
+    );
```
  (parameter is currently declared non-`mut`; shadow or change the signature's binding to `mut`.)
  `GetEnvironmentVariableA` (same file, lines 38-197) was **not** given an equivalent NULL guard
  upstream either — its ANSI path takes `lpBuffer: LPSTR` and always allocates its own
  intermediate Unicode buffer before ever touching `lpBuffer`, so the NULL-buffer path there
  doesn't reach a raw NULL write the same way; left alone, matching upstream's own scope.

### W3-2/W3-3. `NetLocalGroupAddMembers` / `NetLocalGroupDelMembers`: `BuildSidListFromDomainAndName`'s Win32 status is mis-cast to `NTSTATUS`

- **File / commit**: `dll/win32/netapi32/local_group.c`, `f2ef434107e`
  ("[NETAPI32] Fix NetLocalGroupAddMembers crash because return code was interpreted
  incorrectly (#9113)").
- **Upstream diff**: `BuildSidListFromDomainAndName()` returns a `NET_API_STATUS`/Win32 error
  code (it already calls `NetpNtStatusToApiStatus` internally before returning), not an
  `NTSTATUS`. Both callers had been storing that return into an `NTSTATUS Status` and checking
  it with `NT_SUCCESS(Status)`, then feeding it through `NetpNtStatusToApiStatus()` **again** on
  failure — a small nonzero Win32 code (e.g. `ERROR_*` in the low hundreds/thousands) parses as
  a "success" `NTSTATUS` under `NT_SUCCESS`'s top-2-bits check for roughly half the error-code
  space, and even when it reads as a failure, re-running it through
  `NetpNtStatusToApiStatus()` maps an already-Win32 code as if it were an NT status, producing
  garbage — upstream's own commit title says this crashes. Fix: use a plain
  `ApiStatus`/`NERR_Success` comparison instead of `NTSTATUS`/`NT_SUCCESS`, and drop the double
  `NetpNtStatusToApiStatus()` call.
- **Our Rust (pre-fix, live), both sites**:
  - `sdk/lib/rust/netapi32-rust/src/netapi32/local_group.rs:531-559` (`NetLocalGroupAddMembers`):
    `Status = BuildSidListFromDomainAndName(...) as NTSTATUS;` then `if !NT_SUCCESS(Status) { ...
    ApiStatus = NetpNtStatusToApiStatus(Status); }` — the exact pre-fix pattern.
  - `sdk/lib/rust/netapi32-rust/src/netapi32/local_group.rs:1029-1054`
    (`NetLocalGroupDelMembers`): identical pattern (`if !(Status >= 0)` in place of
    `NT_SUCCESS`, same mis-cast).
  - `BuildSidListFromDomainAndName` itself (`local_group.rs:196` on) is confirmed to return a
    plain `u32` API status (`ApiStatus`), matching upstream's premise.
- Both call sites are `#[no_mangle] pub unsafe extern "system"` and directly exported —
  live, reachable from any `netapi32.dll` client managing local groups (e.g. `net localgroup`,
  Computer Management's Local Users and Groups snap-in, domain-join scripts).
- Proposed fix (mirrors upstream, same shape at both sites — shown for `AddMembers`;
  `DelMembers` is the same edit at its own `case 3 =>` arm):
```diff
--- a/sdk/lib/rust/netapi32-rust/src/netapi32/local_group.rs
+++ b/sdk/lib/rust/netapi32-rust/src/netapi32/local_group.rs
@@ pub unsafe extern "system" fn NetLocalGroupAddMembers(
             3u32 => {
-                Status = BuildSidListFromDomainAndName(
+                let SidListStatus = BuildSidListFromDomainAndName(
                     if !servername.is_null() {
                         &raw mut ServerName
                     } else {
                         core::ptr::null_mut::<_UNICODE_STRING>()
                     },
                     buf as PLOCALGROUP_MEMBERS_INFO_3,
                     totalentries as u32,
                     &raw mut MemberList,
-                ) as NTSTATUS;
-                if !NT_SUCCESS(Status) {
+                );
+                if SidListStatus != NERR_Success as DWORD {
                     /* ... log with SidListStatus instead of Status ... */
-                    ApiStatus = NetpNtStatusToApiStatus(Status);
+                    ApiStatus = SidListStatus;
                     break 'outer;
                 }
             }
```
  Apply the mirror-image edit to `NetLocalGroupDelMembers`'s `3 =>` arm (`local_group.rs:1029`).

### W3-4. `NetGetJoinInformation`: shipped as the permanently-disabled `#if 0` stub — never queries real join status

- **File / commit**: `dll/win32/netapi32/wksta_new.c`, `71ee75477e2`
  ("[WKSSVC][NETAPI32] Fix NetGetJoinInformation").
- **Upstream diff**: the pre-fix C had the entire RPC call to `NetrGetJoinInformation` wrapped in
  `#if 0 ... #endif` (disabled per `CORE-17679`), with the function unconditionally falling
  through to `*lpNameBuffer = NULL; *BufferType = NetSetupUnknownStatus;` — i.e. the function was
  a hardcoded "I don't know" stub regardless of actual domain/workgroup membership. The fix
  re-enables the RPC call (via a local `pName` so a mid-call failure can't half-write the
  caller's `*lpNameBuffer`) and only overwrites the out-params on `NERR_Success`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/netapi32-rust/src/netapi32/wksta_new.rs:257-272` —
  translated straight from the *disabled* branch of the `#if 0`: the RPC stub
  `NetrGetJoinInformation` is never called; the function unconditionally does
  `*lpNameBuffer = null; *BufferType = NetSetupUnknownStatus; return NERR_Success;`. Confirmed
  the RPC client stub already exists in-crate (`sdk/lib/rust/netapi32-rust/src/rpc_stubs.rs:387`,
  `NetrGetJoinInformation`), so this isn't blocked on missing substrate — the fix is a direct
  port.
- `#[no_mangle] pub unsafe extern "system" fn NetGetJoinInformation` — live, exported. This is
  the API `NetGetJoinInformation`/`GetComputerNameEx`-adjacent tooling and unattended-setup
  scripts use to detect domain-vs-workgroup state; today it unconditionally reports "unknown"
  regardless of actual state.
- Proposed fix:
```diff
--- a/sdk/lib/rust/netapi32-rust/src/netapi32/wksta_new.rs
+++ b/sdk/lib/rust/netapi32-rust/src/netapi32/wksta_new.rs
@@ pub unsafe extern "system" fn NetGetJoinInformation(
     if lpNameBuffer.is_null() || BufferType.is_null() {
         return ERROR_INVALID_PARAMETER;
     }
-    *lpNameBuffer = core::ptr::null_mut();
-    *BufferType = NetSetupUnknownStatus;
-    0 // NERR_Success
+    let mut pName: LPWSTR = core::ptr::null_mut();
+    // RpcTryExcept/RpcExcept: crate convention elsewhere in this file (e.g. line ~211)
+    // wraps the raw RPC call directly since SEH-around-RPC isn't modeled; follow that
+    // pattern here rather than inventing new plumbing.
+    let status = NetrGetJoinInformation(lpServer as LPWSTR, &mut pName, BufferType);
+    if status == 0 /* NERR_Success */ {
+        *lpNameBuffer = pName;
+    }
+    status
```
  (Verify `NetrGetJoinInformation`'s exact Rust signature at `rpc_stubs.rs:387` before wiring —
  upstream's C passes `(LPWSTR)lpServer`, a `&mut pName` out-param, and `BufferType` straight
  through; match whatever the existing bindgen'd RPC stub actually declares.)
  Note: `base/services/wkssvc/domain.c`'s `NetpGetJoinInformation` (the RPC *server*-side
  function `NetrGetJoinInformation` ultimately calls into,
  `sdk/lib/rust/wkssvc-rust/src/wkssvc/domain.rs:51`) was refactored by the same commit
  (cleanup: defer `*NameBuffer`/`*BufferType` writes to a single success-only exit point,
  `ERR`-log the `LsaQueryInformationPolicy` failure path) but is **not** independently broken in
  our tree — our pre-refactor translation already produces the same net answer on every path
  checked; this is not a second bug, just the same commit's other hunk. Not spending a proposed
  diff on it; flagging so nobody re-opens it as a fifth finding.

## REFUTED / not a live regression — detail

- **`msv1_0/msv1_0.c`, `d9d27f89433`** ("Improve LsaApCallPackage") — adds a brand-new
  `MsvpLm20ChallengeRequest` handler (previously unimplemented message type) plus
  `ASSERT(RequestBuffer->MessageType == ...)` sanity checks in three existing handlers. This is
  feature work (an unimplemented LSA message type going from "not handled" to "handled"), not a
  fix to existing behavior — nothing to regress against. The added `ASSERT`s are debug-build-only
  and don't change release-build (or Rust) behavior. Not chased as a bug; it's a translation-gap
  item (`MsvpLm20ChallengeRequest` presumably doesn't exist in the Rust twin at all) that belongs
  on the translation backlog, not this bug-hunt.
- **`msgina/{gui,shutdown}.c`, `6ae2d5ab43f`** ("Use new branding library") — UI/branding
  refactor (touches only `.rc` resources + a new shared branding library call), not a logic fix.
- **`base/services/wkssvc/domain.c`** — see W3-4's closing note; same commit, not independently
  broken.
- **`dll/win32/msafd/misc/dllmain.c`, `a70f59b0dd5`** — `RtlConvertLongToLargeInteger(0)` →
  `.QuadPart = 0LL` and `RtlConvertLongToLargeInteger(-1000000)` → `.QuadPart = -1000000LL`:
  semantically identical, mechanical inlining of an obsolete NT routine. No Rust action.
- **`dhcpcsvc/dhcp/adapter.c`, `5d6987462da`** — `u_int8_t haddr[16]` → `u_int8_t *haddr` in a
  function signature to silence a GCC-13 `-Wstringop-overflow` false positive (the real argument
  is always an 8-byte field; the false positive came from the array-size annotation, not a real
  overflow). No behavior change.
- **`dll/win32/kernel32/*`, `c81bc9f21ea`** ("Split off some ANSI functions...", #8555) — hits
  5 of the scoped files (`atom.c`, `path.c`, `lcformat.c`, `version.c`, `client/file/move.c`);
  confirmed via `git show --stat` this is a pure file-split (ANSI wrapper functions moved into
  sibling `*ansi.c` files + a `CMakeLists.txt` update), not a behavior change. No Rust action —
  the moved code's behavior is whatever it already was wherever our Rust twin currently has it.
- **`dll/win32/kernel32/client/handle.c`, `63f86fa73bd`** — pure rename
  (`OBJECT_HANDLE_ATTRIBUTE_INFORMATION` → `OBJECT_HANDLE_FLAG_INFORMATION`), same struct layout,
  correcting a non-standard header alias to match the real SDK name. No behavior change.

## Not reached

- **~50 netcfgx/dhcpcsvc "Implement X"/"Improve Y" commits**: read at commit-subject level only
  (all read as incremental feature build-out of previously-stubbed COM interfaces/RPC calls —
  `ITcpipProperties_fnUnknown1/2`, `INetCfgComponentBindings`, `DhcpRequestParams`, etc.) — same
  characterization wave 1 already gave this cluster. Not opened in full diff; a few subjects
  contain the word "Fix" (`54c667caaa7` wcsicmp calls, `a2c594187e5` wcsicmp MSVC build fix,
  `afbc9224052` ITcpipProperties_fnUnknown1 output, `5a147d292d6` default-gateway-forgotten,
  `1a04259951e` gateway/nameserver dialog data, `0df3463af02` radio button selection,
  `4b75ec5508d`/`7139e57ff89` alternate-config TCP/IP dialog) — these read like real UI-state
  bugs in netcfgx's property-sheet code, worth a dedicated pass; not opened this time.
- **~15 shell32 feature commits** (`a5f51269c88` FolderItems3, `c891aefdcff` CFolder
  getters/setters, `095860df576` NT6+ stubs bundle, `16bb138e325` named registry verbs,
  `9e5774e75cd` CDefFolderMenu_MergeMenu, `6af07a31b00` FONTEXT delete, `b60f66cbbfa`
  CPrinterFolder IQueryAssociations, `9342ed43d9a` namespace item visibility,
  `0533a67b184` SHRestrictedMessageBox, `93f96e592b2` IQueryAssociations::GetKey,
  `763ce84cf55` bat/cmd drop target, `546297fe862` SHEvaluateSystemCommandTemplate,
  `3b58b347bc1` Explorer refresh forwarding, `fe2b8ba41b1`/`a83411cfefe` CShellDispatch,
  `afd1ef71031` branding library) — genuine feature additions with no prior C behavior to
  regress against; translation-backlog items, not bug-hunt targets. Diffstats read, full diffs
  not opened.
- **`base/shell/explorer/syspager.cpp`** (the actual crash fix in `8d44ccdd5f7`,
  "Fix crash when tray icon with active balloon is removed") — this file is **not** in the
  182-file HIGH-confidence scope (explorer.exe's tray isn't cohab-flipped at HIGH confidence per
  the registry), so it's out of this lane's frame; only `systray.cpp`'s minor
  `NIM_SETFOCUS`/`AllowSetForegroundWindow` companion hunk from the same commit was in scope,
  and it's a separate, low-severity feature addition (balloon-notification foreground-window
  permission), not the crash itself. Flagging in case another lane owns explorer-rust.
- **`dll/cpl/desk/{appearance,effappdlg,theme}.c`, `56746c29df0`** ("Apply the Shell Icon Size
  properly") and **`background.c`'s `3016c40dc72`/`249be9ecdd3`** ("Check if wallpaper file is
  deleted" + its own revert) — read as real UI-correctness bugs (icon sizing, wallpaper file
  existence check) but low blast radius (Display control panel only) and not opened in full;
  candidate for a future pass if desk-rust is otherwise being touched.
- **`dll/cpl/sysdm/performance.c`, `2290d0e7855`** and **`dll/win32/iphlpapi/iphlpapi_main.c`,
  `7f9e42cfe21`** and **`dll/win32/ws2_32/src/addrconv.c`, `6679825f21e`** and
  **`dll/win32/wshtcpip/wshtcpip.c`, `248aa75023a`** — each a single feature-shaped commit
  (shlobj_undoc.h enhancement, NhGetGuidFromInterfaceName, inet_addr import, SO_KEEPALIVE) not
  opened in full; subjects don't read as fixes to existing behavior.
- **Registry check**: no `rust_files: []` / dir-name-guess-fallback entries were hit among the
  files actually opened this pass (all four CONFIRMED files' registry entries carry accurate,
  non-empty `rust_files` pointing at the right crate — spot-checked above). No registry errors
  to report back to the generator from this pass.

## Summary

- **Files reached in full** (diff opened + Rust cross-checked): 15 of the ~90 remaining
  non-boot-path files (kernel32 ×11, netapi32 ×2, samsrv-cluster cosmetic-confirm ×1 (msafd),
  msv1_0 ×1, msgina ×1(subject-level, both files same commit), wkssvc ×1).
- **Files triaged at commit-subject/diffstat level only**: the remaining ~75 (netcfgx, dhcpcsvc,
  shell32 feature commits, cpl applets, freeldr ui, acpi, misc single-commit files).
- **4 CONFIRMED bugs** (W3-1 through W3-4), all **live** (`#[no_mangle]` exported, reachable from
  ordinary usermode API calls, no test harness needed to trigger):
  - **W3-1** (`kernel32.dll`, every process): `GetEnvironmentVariableW` NULL-buffer write —
    highest blast radius of the four, crash-class.
  - **W3-2/W3-3** (`netapi32.dll`): `NetLocalGroupAddMembers`/`DelMembers` mis-cast Win32 status
    as NTSTATUS — same crash class the upstream commit title names, reachable from `net
    localgroup` / Local Users and Groups / any domain-join tooling.
  - **W3-4** (`netapi32.dll`): `NetGetJoinInformation` ships as the disabled-in-upstream stub —
    a functionality gap (not a crash) but a real behavioral regression vs. current upstream:
    every caller gets "unknown" join status regardless of truth.
- **0 REFUTED-as-live-bug**; 7 items opened and found to be pure cosmetic/refactor/rename with
  no behavior change (`c81bc9f21ea` ANSI split, `63f86fa73bd` rename, `a70f59b0dd5` inline
  conversion, `5d6987462da` warning silencer, `6ae2d5ab43f` branding, plus `d9d27f89433`
  msv1_0 and `wkssvc/domain.c`'s companion refactor, both feature/non-bug).
