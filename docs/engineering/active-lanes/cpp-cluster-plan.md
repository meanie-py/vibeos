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

- Deterministic lowering of per-arch `#if` chains in C++ synthetic units (drwtsn32's
  CONTEXT access was hand-written as `cfg` blocks instead).
- certmgr's hand-ported `src/rosctrls.rs`: rosctrls.h is now modeled by cppapp (rapps), so
  a certmgr regeneration should drop the hand port.
- The shell extensions need what rapps half-built: `atl_rust::com` covers a COM object's
  IUnknown, but not a DLL's object map (`BEGIN_OBJECT_MAP`/`OBJECT_ENTRY_AUTO`), class
  factories, `DllGetClassObject`/`DllCanUnloadNow`, or `.rgs` registration.
- rapps' `src/gdiplus_flat.rs` is a hand-made extern block, copied from desk-rust's C island:
  the C++ island binds no `Gdiplus::DllExports` function. gen_driver_types should bind the
  flat functions reached by an external class's bodies.
- `atl_rust::win::create_window` (the DECLARE_WND_CLASS path) does not give a child window
  `this` as its control ID the way ATL's `CWindowImplBaseT::Create` does;
  `create_window_info` (a module's own GetWndClassInfo) does. Unify them, and check the
  shipped window modules when doing so.
- rapps integrators' local workarounds that want an atl_rust/prelude home: `CAtlList::FindPos`
  (returns POSITION; `Find` returns bool), `CStringW::{Replace(str,str), FindOneOf,
  CopyChars, GetEnvironmentVariable}`, `CStringA::Append`, `InterlockedIncrement64` in
  win32macros-rust, and an `atl_rust::win::unsubclass_window` (loaddlg declares GetPropW /
  RemovePropW locally and hard-codes atl_rust's prop names). The island's
  `ATL__ATL_WNDCLASSINFOW::m_wc` is `WNDCLASSEXA`, and prelude `DLGPROC`/`WNDPROC` are spelled
  over `ntdll_sys::HWND__` rather than the island's; both are transmuted at the call site.
  The model also wrote `new_0::<T>()` on non-generic fns and called `__ctor_0CUiWindow`.
- A `port integrate` pass rewrites extern blocks after the hand bank is applied: a nested
  block's closing brace is dedented to column 0 and `unsafe extern` loses `unsafe` (rapps'
  misc/settings/loaddlg). Harmless on edition 2021, but hand text should be final; find the
  pass and run it before `apply_hand`, or make it preserve the block's own text.
- **Re-run `port cppapp` + `port replay` for the shipped C++ apps** after the member
  default-construction fix (`_ctor_prologue`, 2026-10-15; rapps crashed on it). Audit of the
  saved models: mspaint 10 members (CStyledCursor's `-1`/`CLR_INVALID` cache keys and
  VirtualBrush's `PAL_MODERN` start zeroed -- first comparison differs, looks benign), certmgr 5
  (CSortableHeader, CSplitter: read their ctors), cleanmgr 1 and mmc_new 21 (all CWindow,
  harmless), certutil/drwtsn32/sndrec32 0. cicero/msctf/msutb/msctfime/ctfmon's saved models
  predate a `Method` field rename (`callconv`) and could not be read; regenerate to audit.
  The changed ctors lowered at $0 for rapps. The same re-run picks up three more fixes from
  rapps bring-up: a polymorphic Drop resetting its own vtables, `return this;` as a secondary
  base (a probe of the saved units found 0 such returns in the shipped apps; `this` PASSED as
  a base argument is not yet audited), and `delete member;` (cosmetic once the model repairs it).
- cpp_lower should spell a C++ implicit converting-constructor TEMPORARY (clang's
  `ConstructorConversion` into a `const CStringA&` / `const CStringW&` parameter) as an
  explicit `CStringA(p)`, the way `_temporary` does for written ones. rapps' three FDI
  sites were hand-fixed and banked; the model's guess was a pointer cast.
- ~290 bare fn aliases in already-shipped crates have the shape `_option_fn_aliases` fixes
  (non-`Option` fn pointer); UB only if the value can be zero. Audit with that pass.
- certutil: a SHA1/`hashfile` case on a system with a CSP registered (the live CD has none;
  current suite expects "no context", identical in C and Rust).
- Not exercised at runtime: a real drwtsn32 crash dump, mmc_new's console view/snap-ins.
- Parked stashes (apply by sha, never by index):
  `80285d3d193eb2e2bd274fd359b46ec9645aec8f` sndrec32 cppapp regen vs master;
  `26249d88e2a4605243f43c940bc55631d8904a77` cleanmgr cppapp regen (compiles, not runtime-tested).
- shell32's move onto `atlclasses-rust` (coordinate with the shell32-com lane).
