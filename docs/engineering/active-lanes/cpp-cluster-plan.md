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

## rapps (in progress, 2026-10-14)

Tooling done for it (pipeline doc § `port cppapp`): per-instantiation class templates, the
rosctrls.h controls modeled as external classes (only reached methods are translated), ATL
COM objects over `atl_rust::com`, and the window glue for a hand-written
`ProcessWindowMessage`/`GetWndClassInfo`, CRTP through a template, `CWindowImpl<T, TBase>`
and window traits. Translation: 607 functions, $0.58. Five were rejected and are to be
hand-written: cabinet's `FNCLOSE(fnFileClose)` (an FDI macro that names the function), loaddlg's
`CDowloadingAppsListView::AddRow`, and misc's `RegKeyHasValues`, `IsSameRegKey` (a
function-local struct with a constructor) and `GuessInstallerType`.

## Queue after rapps

Shell extensions: zipfldr, fontext, netshell, stobject, acppage, cabview, mydocs, sendmail,
ntobjshex, mmcshext. Then devmgr, ksproxy + the DirectShow DLLs, telnet, portcls, hdaudbus,
browseui (C++ part), framedyn, iernonce, shimgvw, tftpd, rshell. (explorer.exe is already
Rust — don't requeue it.)

## Owed

- Deterministic lowering of per-arch `#if` chains in C++ synthetic units (drwtsn32's
  CONTEXT access was hand-written as `cfg` blocks instead).
- certmgr's hand-ported `src/rosctrls.rs`: rosctrls.h is now modeled by cppapp (rapps), so
  a certmgr regeneration should drop the hand port.
- The shell extensions need what rapps half-built: `atl_rust::com` covers a COM object's
  IUnknown, but not a DLL's object map (`BEGIN_OBJECT_MAP`/`OBJECT_ENTRY_AUTO`), class
  factories, `DllGetClassObject`/`DllCanUnloadNow`, or `.rgs` registration.
- `atl_rust::win::create_window` (the DECLARE_WND_CLASS path) does not give a child window
  `this` as its control ID the way ATL's `CWindowImplBaseT::Create` does;
  `create_window_info` (a module's own GetWndClassInfo) does. Unify them, and check the
  shipped window modules when doing so.
- ~290 bare fn aliases in already-shipped crates have the shape `_option_fn_aliases` fixes
  (non-`Option` fn pointer); UB only if the value can be zero. Audit with that pass.
- certutil: a SHA1/`hashfile` case on a system with a CSP registered (the live CD has none;
  current suite expects "no context", identical in C and Rust).
- Not exercised at runtime: a real drwtsn32 crash dump, mmc_new's console view/snap-ins.
- Parked stashes (apply by sha, never by index):
  `80285d3d193eb2e2bd274fd359b46ec9645aec8f` sndrec32 cppapp regen vs master;
  `26249d88e2a4605243f43c940bc55631d8904a77` cleanmgr cppapp regen (compiles, not runtime-tested).
- shell32's move onto `atlclasses-rust` (coordinate with the shell32-com lane).
