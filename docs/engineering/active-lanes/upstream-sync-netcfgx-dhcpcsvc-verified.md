# Upstream-sync: netcfgx/dhcpcsvc cluster — verified (wave 5)

Scope: the ~50-commit `dll/win32/netcfgx` + `base/services/dhcpcsvc` cluster upstream of
`FP=5213cf717cff6946945922e6a7e03d87b0a58e4f`, picking up where wave 4 stopped. Wave 4 already
opened and CONFIRMED the netcfgx basic/alternate TCP/IP dialog bugs (W4-1, W4-2) and the
`ws2_32` `inet_addr` stack overflow (W4-3) and the desk.cpl wallpaper bug (W4-4) — **not
redone here**. This pass opens the remainder of the eight "Fix"-titled commits wave 3 flagged
as unopened, plus a representative sweep of the DHCPCSVC RPC sub-cluster, using the full
commit list derived directly from the repo (see below) rather than trusting either wave's
subject-only list.

Full derived list (`git log --oneline $FP..upstream/master -- dll/win32/netcfgx
base/services/dhcpcsvc`): 50 commits. Already-landed-on-our-tree fixes excluded per brief:
`ed367320c29` (DHCP option stack overflow) and the `stop_selecting` time_t-truncation fix
(`b3a0868a180`) — not re-verified, per instruction.

## Verdict table

| Commit | Subject | Verdict | Notes |
|---|---|---|---|
| `0df3463af02`, `5a147d292d6`, `1a04259951e` | netcfgx basic dialog fixes | **CONFIRMED (wave 4, W4-1)** | not redone |
| `4b75ec5508d` | netcfgx alternate dialog fix | **CONFIRMED (wave 4, W4-2)** | not redone |
| `6679825f21e` | ws2_32 inet_addr rewrite | **CONFIRMED (wave 4, W4-3)** | not redone |
| `54c667caaa7`, `a2c594187e5` | wcsicmp→_wcsicmp | **REFUTED (wave 4)** | not redone |
| `afbc9224052` | Fix output of `ITcpipProperties_fnUnknown1` | **N/A — feature not ported** | see detail |
| `7139e57ff89` | Fix the alternate network configuration | **NOT APPLICABLE YET (confirmed deferred)** | see detail |
| `5d6987462da` | dhcpcsvc GCC13 stringop-overflow warning | **REFUTED — false positive, already correct in Rust** | see detail |
| `bd363a821e9` | Improve `ApplyPnpChanges` (pass PnpReconfigCallback) | **REFUTED — feature add, NULL→NULL unchanged** | see detail |
| `c00b0a40358`, `1d6dae3bb24`, `2e13f97a62c`, `60c8612e738`, `7dbfc3f2b69`, `3733f5f71e0`, `fb47d37a497`, `cbfc755c2f8`, `ed5124b7a86`, `98b9b434c6b`, `10092f84660`, `b2e9439a538`, `4b9adaa971f` | DhcpRequestParams / DhcpRegisterParamChange / DhcpFallbackRefreshParams / alternate-config RPC cluster | **N/A — functions are unimplemented stubs or entirely absent in our Rust** | see detail |
| `d55dbfc87f0`, `526a79ce029`, `1cf31524d50`, `581edfe58ec`, `c2265d992e2`, `c04c092a3e6`, `2586be9bc6e`, `9b450d9f7f7`, `8dc9e50c09e`, `b0c8475980e`, `01a09069244`, `89221f5d557`, `0bd6afd1a89`, `4d2238e07a6`, `e2d6e875f97`, `72c5ab502b0`, `4369126acc8`, `68d15708598`, `2a40a01d4ae`, `a34fd324ef6`, `065c3a8b583`, `1ae75e00ae1`, `16de117a01f`, `1f8f73d3b2e`, `3f9f0e2a961` | remaining feature/refactor commits | **Not reached** | see "Not reached" |
| `ed367320c29`, `b3a0868a180` | already-fixed dhcpcsvc bugs | **excluded per brief** | not re-verified |

## Detail

### `afbc9224052` — REFUTED / N/A: the interface it fixes doesn't exist in our tree

- `git show afbc9224052 -- dll/win32/netcfgx/tcpipconf_notify.c` touches
  `ITcpipProperties_fnUnknown1` / `BuildParametersString`. The whole `ITcpipProperties`
  undocumented interface was **added** by `c04c092a3e6` ("Add the undocumented ITcpipProperties
  interface to the tcpip notify object"), which itself postdates `FP`.
- Our worktree's `dll/win32/netcfgx/tcpipconf_notify.c` is *exactly* at `FP`
  (`git log -1 -- dll/win32/netcfgx/tcpipconf_notify.c` → `5213cf717cf`, the fork point commit
  itself) — `grep -c ITcpipProperties dll/win32/netcfgx/tcpipconf_notify.c` → 0.
  `grep -rn ITcpipProperties sdk/lib/rust/netcfgx-rust/` → 0 hits anywhere in the crate (the only
  `Unknown1` hits are the unrelated `INetCfgComponentPrivate_fnUnknown1`, a different interface
  that predates `FP`).
- So there is nothing on either side of the fork to regress: the code this commit fixes was
  never ported, C or Rust. This is a case where wave 3's original subject-only characterization
  ("incremental feature build-out of previously-stubbed COM interfaces") **HELD** despite wave 4
  flagging it as worth a dedicated look — the "Fix" in the title is fixing a feature that only
  exists upstream, past our fork.

### `7139e57ff89` — confirmed genuinely deferred, not a missed bug

- Reading the full diff: this replaces the *entire* alternate-configuration dialog data model —
  new `AlternateConfiguration` struct (`IpAddress`/`SubnetMask`/`DefaultGateway`/`DnsServer1/2`),
  new dedicated resource IDs (`IDC_ALTIPADDR`, `IDC_ALTSUBNETMASK`, `IDC_ALTDEFGATEWAY`,
  `IDC_ALTDNS1/2`, `IDC_ALTAPIPA`, `IDC_ALTSTATIC` replacing the old shared
  `IDC_USEDHCP`/`IDC_NODHCP`/`IDC_DEFGATEWAY`/`IDC_DNS1/2` IDs the basic dialog also uses), and a
  new `StoreTcpipAlternateSettings` function.
- Commit order (newest→oldest in `git log`) puts `7139e57ff89` *after* `a34fd324ef6` ("Keep two
  configurations in the TCPIP notification object") and after all three of W4-1's fixes
  (`5a147d292d6`, `0df3463af02`) and W4-2's fix (`4b75ec5508d`) — i.e. chronologically newest of
  this whole sub-cluster. It depends on the `a34fd324ef6` struct split (`OldAltConfig`), which we
  also don't have.
- Our current `InitializeTcpipAltDlgCtrls`/`TcpipAltConfDlg` (both C and Rust, per W4-2) are at
  the *pre*-`4b75ec5508d` shape — still using the shared `IDC_USEDHCP`/`IDC_NODHCP`/
  `IDC_DEFGATEWAY`/`IDC_DNS1/2` IDs, no `AlternateConfiguration` struct at all. Applying
  `7139e57ff89` cleanly requires first adopting `a34fd324ef6`'s struct rewrite and the resource-ID
  renumbering across every localized `.rc` — a compound structural change, not a small patch.
- Upstream's own commit message even says `TODO: Fix the format of the alternate configuration
  data`, meaning upstream itself didn't consider this the final word.
- **Verdict**: wave 4's characterization ("real candidate, deferring the actual patch") is
  correct and confirmed — this is not a case that should be force-fit into a small diff. W4-2's
  existing proposed fix (independent of this commit) already closes the live stale-data bug in
  our current code shape; `7139e57ff89` + `a34fd324ef6` should be tracked as one future
  structural-adoption task, not mixed into this diff-hunting pass.

### `5d6987462da` — REFUTED, false-positive GCC diagnostic already handled correctly in Rust

- The diff only changes `AdapterFindByHardwareAddress`'s C *signature* from
  `u_int8_t haddr[16]` to `u_int8_t *haddr` (plus the matching header declaration in
  `rosdhcp.h`). `haddr[16]` is a fixed-size-array parameter *decoration* that C decays to a
  pointer at the call site regardless — it never bounded any real access. GCC 13's
  `-Wstringop-overflow` flagged `AdapterFindByHardwareAddress(Table->table[i].bPhysAddr,
  Table->table[i].dwPhysAddrLen)` because `bPhysAddr` is `BYTE[8]` (`MIB_IFROW`) being passed to
  a parameter *decorated* as `[16]`, purely a diagnostic artifact.
- Read the pre-fix function body (`git show 5d6987462da^:base/services/dhcpcsvc/dhcp/adapter.c`):
  the only access is `memcmp(Adapter->DhclientInfo.hw_address.haddr, haddr, hlen)` — bounded by
  the caller-supplied `hlen`, never by the fake `[16]` size. No real OOB read exists before or
  after this commit; it's purely a signature-decoration fix for the compiler warning.
- Our Rust twin (`sdk/lib/rust/dhcpcsvc-rust/src/dhcp/adapter.rs:364`) already declares
  `AdapterFindByHardwareAddress(haddr: *const u8, hlen: u8)` — a bare pointer, with the same
  `memcmp(..., hlen as usize)` bound at line 371-375, and the call site
  (line 592-595, inside `AdapterDiscoveryThread`) passes `.bPhysAddr.as_ptr()` +
  `.dwPhysAddrLen as u8`. The translation already sidesteps the C array-decoration entirely
  (Rust has no equivalent fake-array-size-in-signature convention), so there is nothing to fix
  and nothing was ever wrong here on either side.
- Also worth noting: `tools/cohab-registry.json`'s `dhcpcsvc-rust` entries (unlike netcfgx/desk)
  correctly list `rust_files` for `adapter.c` — the registry generator gap wave 4 found is
  specific to netcfgx/desk, not dhcpcsvc.

### `bd363a821e9` — REFUTED, feature add with an explicit pre-existing `FIXME`

- Wires a real `INetCfgPnpReconfigCallback` implementation into `ApplyOrCancelChanges` in place
  of a `NULL` that the pre-fix code passed with an explicit `//FIXME ... implement
  INetCfgPnpReconfigCallback and pass it to` comment right above it.
- Our C/Rust (pre-`FP`) still passes `NULL` — the exact behavior the `FIXME` comment describes,
  not a regression, just an unimplemented feature. `INetCfgPnpReconfigCallback_fnSendPnpReconfig`
  is itself `E_NOTIMPL` even in the *post*-fix upstream code, so the feature isn't even
  functional yet upstream either. Nothing to regress against.

### DHCPCSVC RPC sub-cluster (`c00b0a40358`, `1d6dae3bb24`, `2e13f97a62c`, `60c8612e738`, `7dbfc3f2b69`, `3733f5f71e0`, `fb47d37a497`, `cbfc755c2f8`, `ed5124b7a86`, `98b9b434c6b`, `10092f84660`, `b2e9439a538`, `4b9adaa971f`) — N/A, all target unimplemented/absent code

- Sampled `60c8612e738` ("Improve debug message and parameter checks for DhcpRequestParams") in
  full: adds `Flags`/`SendParams`/`RecdParams` NULL/range validation to the *client-side* RPC
  wrapper `DhcpRequestParams` in `dhcpcsvc.c`.
- Checked our Rust: `sdk/lib/rust/dhcpcsvc-rust/src/dhcpcsvc/dhcpcsvc.rs:232`,
  `DhcpRequestParams` is a **bare stub returning 0**, no body at all beyond the signature — none
  of the validation this commit adds, nor the RPC call it wraps, exists yet.
- Spot-checked three more names from this cluster by function-name grep across
  `sdk/lib/rust/dhcpcsvc-rust/`: `DhcpRegisterParamChange`, `DhcpDeRegisterParamChange`,
  `DhcpUndoRequestParams`, `DhcpFallbackRefreshParams`, `DhcpAcquireParametersByBroadcast` — **zero
  hits**, i.e. not translated/present at all. `DhcpNotifyConfigChange`
  (`dhcpcsvc.rs:153`) and `DhcpHandlePnPEvent` (`dhcpcsvc.rs:133`) exist but are themselves
  explicit stubs (`DbgPrint("... not implemented yet")`, return 0/`ERROR_INVALID_PARAMETER`
  immediately) predating the alternate-configuration and PnP-reconfig work these commits build
  out.
- **Verdict for the whole sub-cluster**: this is exactly the "incremental feature build-out of
  previously-stubbed COM interfaces/RPC calls" wave 2/3's subject-only triage described — and
  here that characterization **HOLDS**, verified by checking the actual Rust (not just reading
  subjects) and finding the target functions are stubs or entirely absent. Not opening each of
  the 13 individually beyond this sample was a judgment call given the stub-verification above
  applies uniformly (none of them touch a *live, implemented* code path); flagging this as the
  one place in this pass where a batch judgment substitutes for per-commit diff reading — if a
  future pass wants per-commit rigor here, the function-presence check above is the fast
  discriminator to apply first (absent/stub = skip, present = open the diff).

## Registry errors found (adds to wave 4's list)

`tools/cohab-registry.json` reports `"rust_files": []` for **every** `dll/win32/netcfgx/*.c`
file, not just the two wave 4 already flagged (`tcpipconf_notify.c`, and by extension the whole
crate):

| c_path | registry `rust_files` | actual Rust file (exists on disk) |
|---|---|---|
| `dll/win32/netcfgx/classfactory.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/classfactory.rs` |
| `dll/win32/netcfgx/inetcfgcomp_iface.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/inetcfgcomp_iface.rs` |
| `dll/win32/netcfgx/installer.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/installer.rs` |
| `dll/win32/netcfgx/netcfg_iface.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/netcfg_iface.rs` |
| `dll/win32/netcfgx/netcfgbindinginterface_iface.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/netcfgbindinginterface_iface.rs` |
| `dll/win32/netcfgx/netcfgbindingpath_iface.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/netcfgbindingpath_iface.rs` |
| `dll/win32/netcfgx/netcfgclass_iface.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/netcfgclass_iface.rs` |
| `dll/win32/netcfgx/netcfgx.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/netcfgx.rs` |
| `dll/win32/netcfgx/netinstall.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/netinstall.rs` |
| `dll/win32/netcfgx/propertypage.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/propertypage.rs` |
| `dll/win32/netcfgx/tcpipconf_notify.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs` (already reported, wave 4) |

By contrast `base/services/dhcpcsvc/**` entries checked this pass (`adapter.c`, and the
`dhcpcsvc.c`→`dhcpcsvc.rs` mapping used above) are all correctly populated — the `dir-name-guess`
fallback gap is confirmed netcfgx/desk-specific (wave 4's suspicion), not systemic to the whole
registry.

## Not reached

The remaining 25 commits (listed in the table above) are genuine feature/refactor work on top of
interfaces and struct layouts that postdate `FP` and were not opened individually this pass:
COM interface build-out (`INetCfgComponentBindings::SupportsBindingInterface`,
`INetCfgComponentPrivate_fnUnknown1`, the notify-object rewrite `9b450d9f7f7`/`065c3a8b583`,
registry read/write additions `4369126acc8`/`68d15708598`/`581edfe58ec`), the
`a34fd324ef6` two-configuration struct split `7139e57ff89` depends on, and the DHCPCSVC
alternate-configuration RPC plumbing (`98b9b434c6b`, `10092f84660`, `b2e9439a538`,
`4b9adaa971f`) already judged N/A by the stub-presence check above but not individually
diffed. None of these were flagged by any wave's subject reading as "Fix"-titled correctness
commits, and the two "Fix"-titled ones that *were* flagged and unopened (`afbc9224052`,
`7139e57ff89`) are now resolved above (both N/A, for different reasons).

## Summary

- **Re-checked this pass**: 2 remaining explicitly-flagged "Fix" commits opened in full
  (`afbc9224052`, `7139e57ff89`), plus 2 more opened in full as spot checks
  (`5d6987462da`, `bd363a821e9`), plus 1 representative DHCPCSVC RPC commit opened in full
  (`60c8612e738`) with its cluster-mates verified by function-presence check rather than
  individual diff reads (13 commits, batch judgment — see caveat above). **5 commits fully
  diff-read this pass, 13 more resolved by a documented shortcut, 4 already-resolved wave-4
  items referenced but not redone.**
- **Held vs. flipped**: 0 flips this pass. `afbc9224052` and the DHCPCSVC RPC cluster both
  **HELD** the original "feature build-out of stubbed interfaces" subject-level characterization
  — in both cases verified by checking that the target code doesn't exist in our tree yet, not
  by trusting the subject. `7139e57ff89` **HELD** wave 4's own deferred-not-refuted judgment
  call. `5d6987462da` and `bd363a821e9` are both new REFUTED findings (false-positive compiler
  warning already handled correctly in Rust; feature-add with a pre-existing FIXME,
  respectively) — neither was previously triaged, so there's no prior verdict to flip.
- **No new CONFIRMED bugs this pass.** The cluster's real, confirmed bugs remain the four from
  wave 4 (W4-1 through W4-4). This wave's contribution is narrowing the "not reached" list and
  confirming, with evidence rather than subject-reading, that most of the remaining netcfgx/
  dhcpcsvc commits target code we haven't ported yet and therefore carry no risk.
- **Blast radius note**: consistent with the brief — this is UI/config/RPC-stub code. Nothing
  found this pass changes that assessment; the only high-severity item in the whole cluster
  remains W4-3 (`ws2_32` `inet_addr`), already reported.
- **Registry errors**: 11 netcfgx files confirmed with `rust_files: []` despite real Rust twins
  (1 already known from wave 4, 10 new this pass) — table above, all following the same
  `sdk/lib/rust/<crate>/src/<c_dir_basename>/<file>.rs` pattern the generator should use.
  dhcpcsvc's registry entries, by contrast, are accurate.
