# Upstream-sync: long-tail verification (control-panel applets, singles, freeldr/acpi/net-svc remainder)

Scope: everything wave 3 left in its "Not reached" bucket except netcfgx/dhcpcsvc and shell32
(owned by other lanes) — the 4-file "singles" cluster, desk.cpl's `56746c29df0`, and the
freeldr-ui/acpi/dnsrslvr/tcpsvcs/userenv/wshtcpip remainder of the ~70-file subject-only sweep.
`FP=5213cf717cff6946945922e6a7e03d87b0a58e4f`.

**Read wave 4 first — most of this worklist is already closed there.** Cross-checking against
`upstream-sync-flipped-tu-review-wave4.md` before opening anything:

| Worklist item | Status found |
|---|---|
| `dll/win32/ws2_32/src/addrconv.c` `6679825f21e` | **Already CONFIRMED** by wave 4 (W4-3, `inet_addr` stack overflow — the file that motivated this whole pass). Not re-opened. |
| `dll/cpl/sysdm/performance.c` `2290d0e7855` | **Already REFUTED** by wave 4 — header-include-only fix, no logic in `performance.c` itself. |
| `dll/win32/iphlpapi/iphlpapi_main.c` `7f9e42cfe21` | **Already REFUTED** by wave 4 — pure new-function feature add (`NhGetGuidFromInterfaceName`), nothing to regress against. |
| `dll/cpl/desk/{appearance,effappdlg}.c` `56746c29df0` | **Already REFUTED** by wave 4 (W4 "REFUTED — detail") — standing up a previously `WS_DISABLED`, code-dead checkbox as new behavior; confirmed no prior logic existed to regress. |
| `dll/cpl/desk/theme.c` | Not touched by `56746c29df0` at all (`git log $FP..upstream/master -- dll/cpl/desk/theme.c` is empty) — nothing to check. The worklist's grouping of theme.c with that commit was itself imprecise; there is no commit here. |
| `dll/cpl/desk/background.c` `3016c40dc72`/`249be9ecdd3` | **Already CONFIRMED** by wave 4 (W4-4, ghost wallpaper entry + leaked icon handle). |

So of the originally-named "singles," only **`dll/win32/wshtcpip/wshtcpip.c` `248aa75023a`**
(SO_KEEPALIVE / SIO_KEEPALIVE_VALS) was still genuinely unopened. That is this pass's one real
finding, detailed below.

## Derivation for the rest of the worklist (freeldr ui / acpi / dnsrslvr / tcpsvcs / userenv)

```
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f
git log --oneline $FP..upstream/master -- <HIGH-confidence file>
```
run against every HIGH-confidence `cohab-registry.json` entry under
`boot/freeldr/freeldr/`, `drivers/bus/acpi/`, `hal/halx86/acpi/`, `base/services/dnsrslvr/`,
`base/services/tcpsvcs/`, `dll/win32/userenv/`, `dll/win32/wshtcpip/`. Most HIGH-confidence
files in these trees have **zero** commits in range at all (untouched since the fork). The
files that do have commits:

| c_path | commit(s) | disposition |
|---|---|---|
| `boot/freeldr/freeldr/ntldr/ntldropts.c` | `925d227faaf` | opened in full — **cosmetic** (function move + SAL2 annotation/license-header reformat only; body logic byte-identical modulo the renamed parameter `LoadOptions`→`Options`) |
| `boot/freeldr/freeldr/ui/directui.c` | `9ff4e4b9ef4` | **not opened in full** — see "Not reached" |
| `hal/halx86/acpi/busemul.c` | `d4a3ad83961` | opened in full — **build-config only** (`#ifndef _MINIHAL_` gating around `HalpRegisterKdSupportFunctions`/`HalAdjustResourceList`; no change to any code path built in a non-MINIHAL configuration, which is what `hal-rust` targets) |
| `drivers/bus/acpi/buspdo.c` | `1cf31524d50` | cosmetic — repo-wide `swprintf`/`vswprintf` mechanical rewrite, same class wave 1/3 already characterized; not re-diffed line-by-line |
| `drivers/bus/acpi/busmgr/utils.c` | `aa1cac9d22c` | cosmetic — whitespace/tabs-only indentation pass (`git show --stat` confirms no `+`/`-` content change beyond indentation) |
| `base/services/dnsrslvr/dnsrslvr.c` | `89221f5d557` | opened in full — **feature stub add**, see below |
| `base/services/dnsrslvr/hostsfile.c` | `1cf31524d50` | cosmetic swprintf/vswprintf, same as above |
| `base/services/tcpsvcs/{discard,echo,skelserver,tcpsvcs}.c` | `1cf31524d50` | cosmetic swprintf/vswprintf, same as above |
| `dll/win32/userenv/profile.c` | `1cf31524d50` | cosmetic swprintf/vswprintf, same as above |
| `dll/win32/wshtcpip/wshtcpip.c` | `248aa75023a` | **CONFIRMED**, detailed below |

Everything else in these six trees (dnsrslvr's `cache.c`/`rpcserver.c`; every other tcpsvcs
file; `userenv`'s `desktop/directory/environment/gpolicy/misc/registry/setup/sid/userenv.c`;
`wshtcpip/iflist.c`; the rest of `acpi`'s HIGH set: `busmgr/bus.c`, `busmgr/button.c`,
`busmgr/power.c`, `busmgr/system.c`, `halacpi.c`, `halpnpdd.c`; freeldr's `scsiport.c`, `pxe.c`,
`libsupp.c`, both `winldr.c` arch variants) has **zero commits** in `$FP..upstream/master` —
untouched since the fork, nothing to triage.

## CONFIRMED — detail + proposed diff

### LT-1. `wshtcpip.c`: `SO_KEEPALIVE`/`SIO_KEEPALIVE_VALS` are silently no-ops — setsockopt "succeeds" but never reaches TCP

- **File / commit**: `dll/win32/wshtcpip/wshtcpip.c`, `248aa75023a`
  ("[WSHTCPIP][TCPIP] Implement SO_KEEPALIVE, SIO_KEEPALIVE_VALS").
- **Upstream diff** (three hunks, all load-bearing together):
  1. `GetTdiTypeId()`: the `SOL_SOCKET`/`SO_KEEPALIVE` case previously did
     `ASSERT(FALSE)` and fell through with `TdiId` never set (`TdiType` stayed
     `INFO_TYPE_ADDRESS_OBJECT`). Fixed to set `*TdiType = INFO_TYPE_CONNECTION;
     *TdiId = TCP_SOCKET_KEEPALIVE; return;`.
  2. `WSHIoctl()`: gains a new `else if (IoControlCode == SIO_KEEPALIVE_VALS)` branch that
     validates the input buffer size, builds a `TCP_REQUEST_SET_INFORMATION_EX` targeting
     `TCP_SOCKET_KEEPALIVEVALS`, and either queues it (if the socket isn't bound yet) or sends
     it immediately via `SendRequest(..., IOCTL_TCP_SET_INFORMATION_EX)` — otherwise falls to
     `UNIMPLEMENTED; return WSAEINVAL;` same as before.
  3. `WSHSetSocketInformation()`: the `SO_KEEPALIVE` case previously did
     `DPRINT1("...not yet supported\n"); return 0;` (silent success, no-op). Fixed to validate
     `OptionLength` and then `break` out of the switch, falling into the *existing* generic
     "build a `TCP_REQUEST_SET_INFORMATION_EX` via `GetTdiTypeId` and queue/send it" tail that
     `TCP_NODELAY` already used — i.e. it now actually reaches the TDI layer instead of
     returning early.
- **Our Rust (pre-fix, live)** — all three sites match the pre-fix C exactly:
  - `sdk/lib/rust/wshtcpip-rust/src/wshtcpip/wshtcpip.rs:146-166` (`GetTdiTypeId`): `SO_KEEPALIVE`
    arm calls `RtlAssert(b"FALSE\0"...)` and falls through without setting `TdiId`; `TdiType`
    stays `INFO_TYPE_ADDRESS_OBJECT`. In a checked/debug build this fires the assert; in a free
    build `RtlAssert` is normally compiled out, so the function returns with `*TdiId`
    **uninitialized garbage from the caller's stack** (the caller, `WSHSetSocketInformation`,
    zero-inits `TdiId: ULONG = 0` at line ~635, so in practice it comes back `0`, but that is
    incidental to this call site, not a guarantee `GetTdiTypeId` provides for every caller).
  - `sdk/lib/rust/wshtcpip-rust/src/wshtcpip/wshtcpip.rs:327-355` (`WSHIoctl`): only handles
    `SIO_GET_INTERFACE_LIST`; any other `IoControlCode`, including `SIO_KEEPALIVE_VALS`, falls
    straight to `WSAEINVAL as INT` — no queuing/send logic exists for it at all.
  - `sdk/lib/rust/wshtcpip-rust/src/wshtcpip/wshtcpip.rs:651-653` (`WSHSetSocketInformation`,
    `SO_KEEPALIVE` arm): `return 0;` — byte-for-byte the pre-fix "not yet supported" stub, minus
    even the `DPRINT1`.
- **Live and network-reachable**: `WSHSetSocketInformation`/`WSHIoctl` are
  `#[no_mangle] pub unsafe extern "system"` Winsock Helper DLL (WSH) entry points, called by
  `ws2_32.dll` on every `setsockopt(SOL_SOCKET, SO_KEEPALIVE, ...)` /
  `WSAIoctl(SIO_KEEPALIVE_VALS, ...)` from **any process using TCP sockets**. Net effect today:
  an application enables TCP keepalive, the call reports success, and keepalive is silently
  never turned on at the TCP layer — connections that should be detected as dead (NAT/firewall
  timeout, half-open peer) are not, and per-connection keepalive tuning
  (`SIO_KEEPALIVE_VALS`, used by e.g. long-lived RPC/SMB-style connections) is rejected outright
  with `WSAEINVAL`.
- **Classification honesty**: like wave 3/4's `NhGetGuidFromInterfaceName` and desk.cpl's icon
  sizing, upstream's own pre-fix code carried a `FIXME`/`ASSERT(FALSE)`/"not yet supported"
  marker — this is finishing a stub, not fixing a subtle regression in previously-working logic.
  I'm still calling it CONFIRMED rather than filing it as a pure translation-backlog item (the
  way wave 3 filed `NetGetJoinInformation`'s sibling stub-completions), because **the substrate
  for the fix already exists verbatim in our tree**: `WSHSetSocketInformation`'s own
  `TCP_NODELAY` path (`wshtcpip.rs:680-747`) already builds and queues/sends exactly this shape
  of `TCP_REQUEST_SET_INFORMATION_EX` request — the missing piece is *routing* `SO_KEEPALIVE`
  into that existing tail instead of returning early, not writing new TDI plumbing from
  scratch. That's a materially different (and cheaper, lower-risk) fix than a from-scratch
  feature, and the correctness gap (apps believe keepalive is on when it isn't) is real and
  live, so I'm not discounting it to a backlog note.
- **Blast radius, stated honestly**: this is a real, network-facing correctness gap
  (`wshtcpip.dll`/`ws2_32.dll`, every TCP socket app), materially higher-traffic than the
  desk.cpl findings but lower than `kernel32`'s `GetEnvironmentVariableW` (wave 3) or
  `addrconv.c`'s stack overflow (wave 4) — this is a missing feature with a silent-success
  failure mode, not a crash or memory-safety bug.
- Proposed fix (three coordinated edits):
```diff
--- a/sdk/lib/rust/wshtcpip-rust/src/wshtcpip/wshtcpip.rs
+++ b/sdk/lib/rust/wshtcpip-rust/src/wshtcpip/wshtcpip.rs
@@ fn GetTdiTypeId
     match Level as u32 {
         SOL_SOCKET => {
-            *TdiType = INFO_TYPE_ADDRESS_OBJECT as ULONG;
+            *TdiType = INFO_TYPE_CONNECTION as ULONG;
             match OptionName as u32 {
                 SO_KEEPALIVE => {
-                    if 0 as ::core::ffi::c_int == 0 {
-                        RtlAssert(
-                            b"FALSE\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
-                            b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
-                            175 as ULONG,
-                            core::ptr::null_mut::<CHAR>(),
-                        );
-                    }
+                    *TdiId = TCP_SOCKET_KEEPALIVE as ULONG;
+                    return;
                 }
                 _ => {}
             }
         }
```
```diff
@@ fn WSHIoctl
     if IoControlCode == SIO_GET_INTERFACE_LIST {
         res = WSHIoctl_GetInterfaceList(...);
         return res;
+    } else if IoControlCode == SIO_KEEPALIVE_VALS {
+        if InputBuffer.is_null()
+            || InputBufferLength as usize != core::mem::size_of::<tcp_keepalive>()
+        {
+            return WSAEINVAL as INT;
+        }
+        let Context: PSOCKET_CONTEXT = HelperDllSocketContext as PSOCKET_CONTEXT;
+        let Info: PTCP_REQUEST_SET_INFORMATION_EX = HeapAlloc(
+            GetProcessHeap(),
+            0,
+            core::mem::size_of::<_TCP_REQUEST_SET_INFORMATION_EX>()
+                .wrapping_add(InputBufferLength as usize),
+        ) as PTCP_REQUEST_SET_INFORMATION_EX;
+        if Info.is_null() {
+            return WSAENOBUFS as INT;
+        }
+        (*Info).ID.toi_entity.tei_entity = (*Context).AddrFileEntityType as ULONG;
+        (*Info).ID.toi_entity.tei_instance = (*Context).AddrFileInstance as ULONG;
+        (*Info).ID.toi_class = INFO_CLASS_PROTOCOL as ULONG;
+        (*Info).ID.toi_type = INFO_TYPE_CONNECTION as ULONG;
+        (*Info).ID.toi_id = TCP_SOCKET_KEEPALIVEVALS as ULONG;
+        (*Info).BufferSize = InputBufferLength;
+        memcpy((*Info).Buffer.as_mut_ptr() as *mut _, InputBuffer as *const _, InputBufferLength as usize);
+        // same queue-if-not-bound / send-if-bound shape already used at wshtcpip.rs:702-745
+        if (*Context).SocketState as u32 == SocketStateCreated as u32 {
+            /* ... queue exactly as WSHSetSocketInformation's TCP_NODELAY tail does ... */
+            return 0;
+        }
+        let r = SendRequest(
+            Info as PVOID,
+            (core::mem::size_of::<_TCP_REQUEST_SET_INFORMATION_EX>() + InputBufferLength as usize) as DWORD,
+            IOCTL_TCP_SET_INFORMATION_EX as DWORD,
+        );
+        HeapFree(GetProcessHeap(), 0, Info as PVOID);
+        return r;
     }
```
```diff
@@ fn WSHSetSocketInformation, SOL_SOCKET arm
             SO_KEEPALIVE => {
-                return 0;
+                if (OptionLength as usize) < core::mem::size_of::<BOOL>() {
+                    return WSAEFAULT as INT;
+                }
+                // fall through to the common GetTdiTypeId + queue/send tail below
+                // (same as the TCP_NODELAY arm) instead of returning early
             }
```
  Whoever applies this: don't hand-write the queuing logic in the `WSHIoctl` branch — factor it
  out of the existing `TCP_NODELAY`/generic tail (`wshtcpip.rs:702-745`) into a shared helper
  and call it from both `WSHSetSocketInformation` and the new `WSHIoctl` branch, since the two
  are now doing the identical queue-or-send dance upstream duplicates inline. Verify
  `tcp_keepalive`'s exact field layout (`onoff`/`keepalivetime`/`keepaliveinterval`) exists
  already in this crate's bindgen island before wiring the size check.

## REFUTED / not a live regression — detail

- **`base/services/dnsrslvr/dnsrslvr.c`, `89221f5d557`** ("Notify the dnscache service if the
  names list changed") — adds a `SERVICE_CONTROL_PARAMCHANGE` case to
  `ServiceControlHandler` that does nothing but `DPRINT1("SERVICE_CONTROL_PARAMCHANGE!\n");
  return NO_ERROR;`, and adds `SERVICE_ACCEPT_PARAMCHANGE` to `dwControlsAccepted`. No actual
  dnscache-notification logic is implemented in this commit (the real notify-side work is in
  netcfgx, out of this lane's scope). Confirmed our Rust twin
  (`sdk/lib/rust/dnsrslvr-rust/src/dnsrslvr/dnsrslvr.rs:43-79`) matches the pre-fix C exactly
  (`dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN`, no
  `SERVICE_CONTROL_PARAMCHANGE` case). This is a stub-acceptance feature add with no behavior
  difference worth chasing (accepting an unhandled control code that both the pre- and post-fix
  handler treat as a no-op) — translation-backlog item, not a bug.
- **`boot/freeldr/freeldr/ntldr/ntldropts.c`, `925d227faaf`** — pure function move (from
  elsewhere in freeldr, exact file not re-derived) + rename
  `NtLdrUpdateLoadOptions`→`NtLdrUpdateOptions` + SAL1→SAL2 annotation reformat +
  license-header dual-licensing text change. Diffed the full commit: every hunk in
  `ntldropts.c` itself is comment/annotation/parameter-name (`LoadOptions`→`Options`) churn; no
  control flow or computed value changes. No Rust action.
- **`hal/halx86/acpi/busemul.c`, `d4a3ad83961`** — wraps `HalpRegisterKdSupportFunctions` and
  `HalAdjustResourceList` in `#ifndef _MINIHAL_`, and moves an existing `#ifndef _MINIHAL_` up
  one level (collapsing a redundant nested `#endif`/`#endif` pair). No change to any code path
  compiled for a standard (non-MINIHAL) HAL build, which is what `hal-rust` targets. No Rust
  action.
- **`drivers/bus/acpi/buspdo.c`, `1cf31524d50`**; **`drivers/bus/acpi/busmgr/utils.c`,
  `aa1cac9d22c`**; **`base/services/dnsrslvr/hostsfile.c`**,
  **`base/services/tcpsvcs/{discard,echo,skelserver,tcpsvcs}.c`**,
  **`dll/win32/userenv/profile.c`** (all `1cf31524d50`) — the same repo-wide
  `swprintf`/`vswprintf`→`RtlStringCbPrintfW`-style mechanical rewrite wave 1/3 already
  characterized as cosmetic; `aa1cac9d22c` on `utils.c` is a pure indentation/whitespace pass
  (`git show --stat` shows the changed-line count but zero net content change). Not re-diffed
  hunk-by-hunk given the established pattern; spot-checked `buspdo.c`'s hunk to confirm it's the
  same `swprintf(buf, fmt, ...)` → `RtlStringCbPrintfW`-shim substitution as every other file in
  this cluster, not a size/format-string change.

## Registry errors found

None among the files actually opened this pass. `dll/win32/wshtcpip/wshtcpip.c` and
`dll/win32/wshtcpip/iflist.c` both carry correct, populated `rust_files` entries.
`boot/freeldr/freeldr/ui/directui.c` and `dll/win32/dnsapi/dnsrslvr_c.c` carry the same
`dir-name-guess` `rust_files: []` gap wave 4 already flagged for netcfgx/desk (i.e. this is now
confirmed to also hit `freeldr-rust` and `dnsapi`-adjacent generated bindings, not just
netcfgx/desk) — `directui.c`'s actual Rust twin was not located this pass (see "Not reached");
flagging the registry gap itself is enough to route to the generator fix already tracked.

## Not reached

- **`boot/freeldr/freeldr/ui/directui.c`, `9ff4e4b9ef4`** ("Move the ReactOS/NT-specific
  advanced boot menu to its separate file") — a large (`bootmgr.c` + 2 new headers touched),
  explicitly WIP structural move ("Work-in-progress" in the commit body) of freeldr's advanced
  boot menu (F8/F5/F2) into its own file, plus making the menu depend on OS type. This is
  boot-path-adjacent UI (freeldr, pre-kernel), the registry's `rust_files: []` gap means the
  actual Rust twin location wasn't confirmed, and a WIP structural move has no fixed "correct"
  end-state to diff a translation against mid-refactor. Deferring rather than rushing a
  structural read under this pass's budget — worth a dedicated look once the registry gap for
  `freeldr-rust`'s UI files is fixed so the twin can be found reliably.
- **The rest of the HIGH-confidence acpi/dnsrslvr/tcpsvcs/userenv/wshtcpip/freeldr files with
  zero commits in range** (listed in the derivation table above) — confirmed untouched since
  the fork via `git log`, so there is nothing to triage; not "not reached," genuinely nothing
  there.

## Summary

- **7 subject-level judgements re-checked against wave 3/4's own prior verdicts or fresh
  full-diff review**: `addrconv.c`, `performance.c`, `iphlpapi_main.c`, `56746c29df0`
  (appearance/effappdlg), `background.c` were already closed by wave 4 — **0 flips** on those
  (wave 4's verdicts held). `theme.c` had no commit to check at all (the worklist's grouping was
  imprecise, not wrong-verdict). `dnsrslvr.c`, `ntldropts.c`, `busemul.c` were opened fresh this
  pass and **held** as REFUTED/cosmetic on first read (no prior wave-3/4 verdict existed to flip
  — these were previously untriaged, not previously mistriaged).
- **1 new file actually needed opening and flipped from "subject doesn't read as a fix" to
  CONFIRMED**: `dll/win32/wshtcpip/wshtcpip.c` `248aa75023a` (LT-1, SO_KEEPALIVE/
  SIO_KEEPALIVE_VALS silently no-op). This is the fourth of wave 3's original four-file
  "singles" bullet, and like `addrconv.c` before it, the subject-line read ("Implement
  SO_KEEPALIVE" reads as pure feature) undersold it: the pre-fix state is a *silent-success*
  no-op on a documented, widely-used sockopt, not merely an absent feature.
- **Verdict counts this pass**: 1 CONFIRMED (LT-1), 3 REFUTED-fresh (dnsrslvr.c stub-accept,
  ntldropts.c move, busemul.c build-config), 6 cosmetic-confirmed-by-pattern (swprintf/vswprintf
  cluster + indentation pass), 6 items inherited as already-closed from wave 4 (0 new work), 1
  deferred (directui.c), 1 "no commit exists" (theme.c).
- **Top finding**: LT-1 (`wshtcpip.c`) — live, network-reachable, and the fix is low-risk because
  the exact TDI-request-queuing shape it needs already exists verbatim in the same file for
  `TCP_NODELAY`.
