# Wine upstream intake: translate current Wine, not ReactOS's copy

**Decided 2026-10-12 (user).** Upstream Wine becomes the source of truth for everything
Wine-derived. We translate the latest Wine, not the copy ReactOS synced. DLLs ReactOS never
pulled in come in as vendored C *intake* and are translated from there. Long term, Wine is
tracked the way the upstream-sync lane tracks ReactOS: semantic ports as upstream moves,
not a patch queue. This makes roadmap §7b ("Wine DLL re-implementation in Rust") concrete.

Rust stays the end state. Vendoring here is a way IN, never a parking place: every intake
module is translation debt the ledger shows, not code moved out of the denominator.

## Why

- **Most of what is left is Wine.** At 2026-10-12 the core is 53% Rust, with 1.50M lines
  of live C, 1.25M of which are Wine-derived and 21% covered (`port_audit.py`). ReactOS's
  own code is 85% done.
- **Translating ReactOS's copy freezes a fork twice.** ReactOS syncs Wine on its own
  cadence (195 of the 267 modules in `media/doc/WINESYNC.txt` are at Wine 10.x, a few are
  pinned far older on purpose, dsound at 1.3). Wine is at 11.19. Translating the 10.x copy
  to Rust and then freezing it inherits ReactOS's lag and adds our own. Every later Wine
  fix would have to be re-ported by hand.
- **There is a real gap ReactOS never pulled.** See the measurement below. Some of it
  (Media Foundation, the MSVC runtimes) is exactly what modern apps load.

## The gap, measured

`python3 tools/wine_gap.py --wine ~/src/wine` (Wine 11.19, shallow clone at `~/src/wine`):

| Area (modules absent here) | Modules | Lines | Useful on NT |
|---|---:|---:|---|
| Other Win32 (comctl32 v6, taskschd, msado15/oledb32, sapi, wmp, iertutil, long tail) | 208 | 235K | yes |
| Unix backends (winex11, winevulkan's unix side, winegstreamer, audio drivers) | 18 | 222K | no |
| Media Foundation / DirectShow / audio (mf, mfplat, evr, xaudio2…) | 63 | 89K | yes |
| MSVC runtimes (msvcp60/90/1xx, vcruntime140, concrt) | 35 | 72K | yes |
| 16-bit | 60 | 71K | no |
| WinRT `windows.*` (mostly stubs) | 29 | 42K | yes, low value |
| Networking / web / security (webservices, wsdapi, kerberos…) | 12 | 33K | yes |

Of 741 upstream DLLs, 377 (707K lines) are absent, plus 55 of 116 programs (60K).
**260 useful absent DLLs (241K lines) import only DLLs that exist here.** That is
DLL-level: whether the FUNCTIONS they call are exported is the next question below. The
most-wanted missing imports are `mfuuid`/`mfplat`/`wmcodecdspuuid` (the Media Foundation
cluster, 22–23 dependents each), so MF comes in as a block, not piecemeal.

Caveat: matching is by module name, so something ReactOS carries under another name
(comctl32 v6 is the likely case) reads as absent. Check before acting on any one row.

## Plan

### Phase 1: pin upstream Wine as source (no translation yet)

- **Vendor a pinned Wine tag as SOURCE**, not shipped binaries: the C that translation
  reads and difftest grades against. Recorded in `vendor.yaml`; not in the build.
- **A drift measurement:** per function, diff ReactOS's Wine-10.x copy against the pin for
  the Wine DLLs we carry. This sizes what "translate current Wine" actually changes, and
  it tests Phase 3's claim that re-syncing is cheap before anything relies on it.
- **`rustconv` reads the pin** for a Wine-derived module, through the existing
  `--target`/profile machinery: the island and the compile flags still come from OUR build
  (ReactOS's `CMakeLists.txt` for that DLL), the function bodies from the pin. A function
  the pin has and our tree lacks is new surface; one our tree has and the pin dropped is a
  ReactOS-ism to keep. Both are reported, never silently resolved.

### Phase 2: intake of missing DLLs as vendored C

- Bring a missing DLL in from the pin as C, built by our CMake (a `Makefile.in` → CMake
  generator, so intake is a command, not a recipe), listed in `vendor.yaml` with an
  `intake: true` flag so it is translation debt, not third-party code.
- **`port_audit.py` gets an "Intake (awaiting translation)" row**, counted in the core
  denominator, so the coverage number stays honest when C comes in.
- **Pilot first:** one leaf DLL end to end (taskschd or webservices: DLL-level import-closed,
  no unix side), then the Media Foundation cluster as one block (with `mfuuid`/
  `wmcodecdspuuid` static GUID libs), then the MSVC runtimes.
- **Each intake DLL's unresolved functions feed the API work.** Build it, then run
  `tools/win10_surface.py gap` over the built PE: every import we don't export, ranked. That
  is the same burn-down the research lane is running for the Windows 11 retarget
  ([modern-windows-compat-roadmap.md](modern-windows-compat-roadmap.md) Rung 0, and its
  extended conformance suite graded against real Windows 11, [winref.md](winref.md)). Intake
  DLLs add their callers to that ranking; they don't start a second list.

### Phase 3: semantic ports as Wine moves

The upstream-sync model ([active-lanes/upstream-sync-plan.md](active-lanes/upstream-sync-plan.md):
upstream is an oracle, not a patch queue), applied to Wine:

- When the pin moves, the change lands per FUNCTION: rustconv's translation cache is keyed
  on each function's C body, so only changed functions are re-bought, and an unchanged one
  replays for $0.
- A changed function whose Rust is hand-edited (banked with `port hand`) is not re-bought
  blindly: the upstream diff is ported into the Rust as a semantic change and graded with
  `rustconv difftest-contract` against the new Wine C, as upstream-sync does for ReactOS.
- Bump cadence follows Wine's stable releases (X.0 each January) at minimum; development
  tags only when a specific fix is wanted.

## What does NOT come in

- **Unix backends** (winex11, winewayland, winemac, ALSA/Pulse/OSS/CoreAudio, winebus,
  winevulkan's unix half, winegstreamer's unix half, opencl, wpcap): Wine talking to the
  host OS. On NT the real driver stack fills that role.
- **16-bit** modules: no NTVDM on our targets.
- **Wine's own NT core** (ntdll, kernelbase, win32u and the user32/gdi32 parts routed through
  `win32u` or unix calls): we have our own kernel, win32k and Rust ntdll. These are read as
  an ORACLE for behaviour, never taken as code.

## Open questions

- **Which DLLs stay ReactOS's copy?** A Wine DLL ReactOS has diverged from heavily
  (comctl32, shell32 parts, user32) may be worth keeping as ReactOS's version and treating
  the pin as an oracle only. Decide per DLL from the drift measurement.
- **Wine's private exports.** Upstream DLLs call `__wine_dbg_*`, `wine_get_version` and other
  Wine-only ntdll extensions. ReactOS already shims many for its own syncs (`sdk/tools/
  winesync`, the `wine/` headers); intake needs the full list, from `win10_surface.py gap`
  on the first built pilot.
- **NT 5.2 vs NT 10 targets.** Some current Wine assumes Vista+ behaviour (that is why
  dsound is pinned at 1.3). Intake waits on the research lane's API surface for those; the
  `gap` ranking tells which.
