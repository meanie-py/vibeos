# C++ apps → Rust via `port cppapp` (lane `cpp-cluster`)

Worktree `vibeos-cpp-cluster`, branch `cpp-cluster`. Translator: Haiku 5.5 (`-m haiku`).
Tool reference: `rustconv-pipeline.md` § "`port cppapp`". Per-module results:
`rust-port-status.md` (rows for sndrec32, the CTF cluster, certutil, certmgr, drwtsn32,
mmc_new, …). This doc holds only what is NOT recorded there: the recipe, the queue, the
next design, and what is owed. Paused 2026-10-12 (Sunday night), resumes next weekend.

## State at pause (2026-10-12)

- Last landing: batch 4 (certutil, certmgr, drwtsn32, mmc_new) as master `035be072c956`.
  Lane tip `a8d8da8df97`, tree clean, master 42 commits ahead (merge before resuming).
- Batch 4 spend ≈ $0.45. Runtime: appstorm PASS ×3 arches for certmgr/drwtsn32/mmc_new;
  modrun `certutil.yaml` 11 cases identical to the C oracle ×3.
- Nothing in flight. No paid run interrupted.

## Recipe (per app)

Run from the worktree with `PYTHONPATH=$W/tools/rustconv`:

```
port derive <dir>; port scaffold <dir>; port island <stem>; port cppapp <stem>; port integrate <stem>
port translate <stem> -m haiku --cost-cap <$>
port headers   <stem> -m haiku --cost-cap <$>     # paid; needs -m/--cost-cap
port integrate <stem>; port macros <stem>; port integrate <stem>; port resolve <stem>
crate-repair <stem>-rust --arches i386 amd64 arm64
```

- **`resolve` and `macros` must run AFTER translate.** Before it, the prelude facts and
  externs come out empty, and an integrator will hand-write externs (certutil: 76 → 5 errors
  once re-run in the right order).
- `scaffold` is not idempotent; resume a failed prep from the failing stage, not from the top.
- Resume-cache refusals ("already has purchased output which did not replay") after a
  contract/lowering change: re-buy just those names with `port translate --rebuy-fn <names…>`
  (certmgr: 4 bodies, < $0.01). Loop until the log has no new refusals.
- Then flip CMake (`flip_cmake` drops `cppstl` for C++ modules), add an appstorm entry
  (`tools/payloads/appstorm/appstorm.c`) and, for a console tool, a modrun suite + C oracle.
- Owed (AGENTS.md "a wrapper script is a bug report"): the above order lived in a scratch
  `prep3.sh` and a `rebuy.sh` loop. Fold both into the tool — a `port prep <dir>` that runs
  the stages in the right order and resumes, and `--rebuy-refused` on `port translate`.

## Integration by subagents

Integration (driving `cargo check` to 0 on all three targets, $0) parallelizes; translation
and all shared-file edits stay central. The brief that worked (Sonnet; mmc_new) — key rules:

- Only edit `sdk/lib/rust/<stem>-rust/` and `~/vibeos-rust-staging/port/<stem>/hand/`; no
  paid commands (`port translate`, `port fix`, `replay --fix-cap` > 0); no git, ninja,
  vibeos-cli, qemu. Proposed tool/shared-crate fixes go in the report as diffs.
- Bank each hand-edited function: `port hand <stem> <file>.rs:Class::Method` (a method),
  `<file>.rs:Name` (free fn), `<file>.rs:+Item` (new item). Banking a method under its
  synthetic `Class__Method` name re-applies it as an orphan `fn(&mut self)` outside any impl.
  `port hand --drop <stem> <spec>` removes a bad entry.
- Prove banks with `port replay <stem>` (mmc_new: ~27 min) then crate-repair → 0 errors.
- **Never wait with `pgrep -f <pattern>` when the pattern is in your own command line** — it
  matches itself forever (an integrator lost 3 hours to this). Run long stages in the
  foreground with a long timeout.

## Shell extensions (from 2026-10-17)

**RESUME HERE (2026-10-17, evening):** mmcshext, sendmail, mydocs and acppage are flipped on
`atl_rust::dll`, each with a modrun suite against its C; $0.13 on Haiku for the four, and
`port replay <stem>` rebuilds each at $0. Next in the queue: cabview, stobject, fontext,
ntobjshex, netshell, zipfldr (cabview/fontext/zipfldr use CDataObjectHIDA and
ShellObjectCreator, both now supplied). Landing is a merge request to the integrator
(`vibeos-integration-30` at last contact; find it by role). New linters are Rust rules in
`tools/vibeos-lint`. Run `check` after a gate's builds, not beside them: vibeos-lint peaks near
10 GB. The named stash `6298017a93468426bae860a6887d8112086980cf` is safe to drop.

## rapps (FLIPPED 2026-10-15)

Tooling done for it (pipeline doc § `port cppapp`): per-instantiation class templates, the
rosctrls.h controls modeled as external classes (only reached methods are translated), ATL
COM objects over `atl_rust::com`, and the window glue for a hand-written
`ProcessWindowMessage`/`GetWndClassInfo`, CRTP through a template, `CWindowImpl<T, TBase>`
and window traits. Translation: 607 functions, $0.58. Five were rejected and are to be
hand-written: cabinet's `FNCLOSE(fnFileClose)` (an FDI macro that names the function), loaddlg's
`CDowloadingAppsListView::AddRow`, and misc's `RegKeyHasValues`, `IsSameRegKey` (a
function-local struct with a constructor) and `GuessInstallerType`.

2026-10-15: the crate is at 0 `cargo check` errors on all three arches. Two Sonnet
integrators split the 25 units (UI / the rest); everything they edited is banked in
`~/vibeos-rust-staging/port/rapps/hand/` (~150 entries), so `port replay rapps` puts it back.
Total spend so far ~$0.71. Fidelity fixes they found beyond type errors: a `!(x & F) != 0`
precedence bug, `GetWndClassInfo` statics rebuilt per call (re-registering the class), and
the GDI+ screenshot math done in double as the C++ does.

2026-10-15, flipped: appstorm opens "ReactOS Applications Manager" on all three arches
(i386 9.0 s, amd64 8.2 s, arm64 47.6 s). `port replay rapps` rebuilds it at $0 from the bank
plus ~155 hand entries. Total spend ~$0.73. Bring-up found six generator bugs, each now a
tool fix with a test: member objects not default-constructed, Drop dispatching through the
derived vtables, `return this;` as a secondary base, `delete member;` dropping its operand's
edits, qualified base calls going virtual (infinite recursion), and a `char*` cast to a
CStringA object. `check` then flagged three more in the shipped Rust: two `!BOOL != 0`
(CWindow::DestroyWindow always FALSE; RemoveSelectedAppFromRegistry always bailing) and an
array passed by value into variadic ConPrintf (`rapps /?`). Bring-up method that worked:
`llvm-symbolizer-18 --obj=<exe> <RIP>` on the serial dump, then `atl_rust::win::trace` probes.

## Queue after rapps

Shell extensions: zipfldr, fontext, netshell, stobject, acppage, cabview, mydocs, sendmail,
ntobjshex, mmcshext. Then devmgr, ksproxy + the DirectShow DLLs, telnet, portcls, hdaudbus,
browseui (C++ part), framedyn, iernonce, shimgvw, tftpd, rshell. (explorer.exe is already
Rust — don't requeue it.)

## Owed

- rapps and cleanmgr carry `CWindow__rcDefault` ZEROED (generated before cppapp refused an
  initializer it cannot render); ATL's is `{ CW_USEDEFAULT, CW_USEDEFAULT, 0, 0 }`. Nothing
  reads it today (atl_rust::win sizes windows itself), so it is latent. cppapp now renders
  brace-list statics against the island and substrate structs: a `port cppapp` + replay of
  each fixes it, alongside the regens already owed below.
- cppapp worker bodies drop the C++'s wine debug prints: 88 `ERR`/`FIXME`/`WARN` calls came
  back as `// ERR(...)` comments across msctf (11 files), msctfime, sendmail, mydocs. Only the
  debug channel loses them, but it is the cluster's whole debug surface. Lower them in
  cpp_lower to `WINE_ERR!`/`WINE_FIXME!` (win32macros-rust) before the model sees the body.
- The worker contract's `T__zeroed()` / `x.__ctor_N()` rule is read as covering ATL types too:
  sendmail came back with `CStringW__zeroed()` + `__ctor_1(p)` for `CStringW s(p)` (hand-fixed
  and banked). Saying in the contract that ATL types use their atl_rust API changes prompt
  text, so do it when a re-buy is planned anyway, or lower ATL locals deterministically in
  cpp_lower (`CStringW x(p)` -> `CStringW::from_ptr(p)`), which costs nothing.
- Deterministic lowering of per-arch `#if` chains in C++ synthetic units (drwtsn32's
  CONTEXT access was hand-written as `cfg` blocks instead).
- certmgr's hand-ported `src/rosctrls.rs`: rosctrls.h is now modeled by cppapp (rapps), so
  a certmgr regeneration should drop the hand port.
- `atl_rust::dll` refuses, rather than approximates, what no module in the queue has used
  yet: aggregation, `OBJECT_ENTRY_AUTO`, other class factories, a class's own `ObjectMain`,
  typelib registration. Port each when a module needs it.
- rapps' `src/gdiplus_flat.rs` is a hand-made extern block, copied from desk-rust's C island:
  the C++ island binds no `Gdiplus::DllExports` function. gen_driver_types should bind the
  flat functions reached by an external class's bodies.
- rapps' remaining local workaround: `CComPtr::CoCreateInstance` (geninst declares
  CoCreateInstance itself; atl_rust would need its `com` feature). The island's
  `ATL__ATL_WNDCLASSINFOW::m_wc` is `WNDCLASSEXA`, and prelude `DLGPROC`/`WNDPROC` are spelled
  over `ntdll_sys::HWND__` rather than the island's; both are transmuted at the call site.
  (2026-10-17: CStringW `ReplaceStr`/`FindOneOf`/`CopyChars`/`GetEnvironmentVariable`,
  `CStringA::Append`, `InterlockedIncrement64`, `unsubclass_window` and `CAtlList::FindPos`
  are shared now, and rapps uses them; `create_window` follows ATL's child-ID rule.)
- **Shipped C++ apps vs this session's generator fixes (2026-10-16).** cicero was
  regenerated (`port cppapp` + `port replay`, $0.18 of re-buys; bodies imported from
  vibeos-msvcrt-homebrew with `port bank-import`) and the whole CTF cluster checks clean x3;
  it fixed 17 override self-dispatch recursions. The 19 in mspaint/msutb/msctfime were
  rewritten in place and banked instead (`tools/scan_override_self_dispatch.py` now gates the
  shape). Member default-construction audit: certmgr's CSplitter members are CWindows (a
  zeroed `m_hWnd` is what their constructor writes) and CSortableHeader has none, mspaint's
  are benign (plan, 2026-10-15) -- so no other regeneration is owed for correctness. Still
  owed: a regeneration of mspaint/msutb/msctfime/msctf/ctfmon to drop their hand-banked
  workarounds. `this` passed as a non-primary base: audited 2026-10-17 (28 sites in the
  13 shipped crates), one wrong in msctf, fixed and banked.
- Port passes that read only `*.[ch]`: audited 2026-10-17. `_wine_channel_macro_calls` and
  `_resolve_anonymous_types` are C-only by design (cppapp preludes define no channel macro;
  c2rust cannot read C++). `initguid_storage` was a real miss (a `.cpp`'s `<initguid.h>`),
  fixed. The C++ crates' preambles still carry a dead `__wine_dbch_<x>` static nothing reads.
- Bare fn aliases in shipped crates: 119 in 45 crates (2026-10-17 count), 48 with a field,
  transmute or null-test use. `rustconv fn-alias-audit <crate> [--apply]` reports them and
  rewrites to `Option<fn>` + crate-repair. Confirmed UB: cmd's `MYEX` (fixed), cabinet's
  `FDICreate`/`FCICreate` null tests on bare-fn parameters, ctfmon's
  `TF_CreateLangProfileUtil` -- all three fixed. The other eight flagged (apphelp, cnt, ucrt,
  msiexec x2, mouhid, freeldr, shell32) transmute values that are null-checked first or
  non-null by contract: safe as written. The remaining ~70 have no null test or transmute.
- certutil: a SHA1/`hashfile` case on a system with a CSP registered (the live CD has none;
  current suite expects "no context", identical in C and Rust).
- Not exercised at runtime: a real drwtsn32 crash dump, mmc_new's console view/snap-ins.
- Parked stashes (apply by sha, never by index):
  `80285d3d193eb2e2bd274fd359b46ec9645aec8f` sndrec32 cppapp regen vs master;
  `26249d88e2a4605243f43c940bc55631d8904a77` cleanmgr cppapp regen (compiles, not runtime-tested).
- shell32's move onto `atlclasses-rust` (coordinate with the shell32-com lane).
