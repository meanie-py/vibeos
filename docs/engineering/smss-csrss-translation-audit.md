# SMSS + CSRSS C→Rust Translation Audit

Last updated 2026-05-16. **Status: Phase 1 COMPLETE (green ×3,
committed `30476658`); Phase 2 csrsrv translated+integrated, broken-
but-visible 239→84 (committed `e5228058`, all systematic roots
cleared, per-site mechanical tail owed); Phase 0 + P1a COMPLETE;
csrss.exe fully Rust.** (worktree `worktree-smss-csrss`, merged
`continue-rust-port` at `689cb2f6`.)

### Phase 1–2 reality + durable learnings (2026-05-16)

- **The Phase-0 substrate was NOT actually complete** ("PROVEN" was
  optimistic — it measured resolve%, not compile). Real gaps found
  only when the first multi-file component pulled the full surface:
  (1) **no `ntdll-sys/src/constants.rs`** — 138 hand STATUS_/DBG_
  codes had to be ported from ntkrnl-sys (ntstatus.h cast-macros are
  not bindgen-evaluable, as the audit said — but the consts layer
  the audit's `base.rs` box implied was never populated by P1a);
  (2) **`NtCurrentTeb` was an ambiguous-glob-reexport** (inlines::*
  AND bindgen::* → unresolvable) — fixed with an explicit named
  re-export (which then must also name every new inline);
  (3) **no universal LIST_ENTRY/Interlocked inlines** — ported from
  ntkrnl-sys into `ntdll-sys/inlines.rs`.
- **`cmd_submit` was never target-aware** (latent bug, all phases):
  hardcoded ntoskrnl → misrouted crate + dropped the user-mode
  system-prompt addendum. Fixed (honors `detect_target`).
- **Tightly-coupled multi-file components need a per-component
  `*_inlines.rs`** translating the component's own `.h` macros once
  (the ntoskrnl-rust `ps/inlines.rs` / win32k intra-component
  analog). csrsrv: `csrsrv/srv_inlines.rs` (CsrAcquire/ReleaseLock,
  CsrGetClientThread, InterlockedUL). The 7 files `use super::
  srv_inlines::*;`. This single fix cleared ~90 of 239.
- **`cargo fix --broken-code --allow-dirty` is a strong deterministic
  lever** for the mechanical tail — auto-cleared ~25% (113→84) of
  rustc machine-applicable suggestions, zero risk (committed
  checkpoint). RUN IT EARLY in Phase 3/4 integration.
- **Cost reality:** translation is cheap (~$0.05/file, as budgeted);
  **integration-to-green is the real cost** — even with a hardened
  pipeline a 5.4K-LOC/7-file component is a heavy heterogeneous
  per-site grind (ptr casts, i386 stdcall-vs-system fn-ptr, int
  widths, per-struct bindgen-anon paths, LUID/SB_API_MSG/_TEB
  direct-vs-anon). No systematic shortcut for the long tail; it is
  visible-in-cargo and owed, not hidden (CLAUDE.md-correct).

(historical:) `ntdll-sys`
built + size-verified, cargo-clean ×3, rustconv-wired; the user-mode
pipeline fixes are landed and proven; `csrss.exe` links the Rust
`smss-csrss-rust` staticlib (csrss.c cohabitation-removed, C kept on
disk), `ninja csrss` green, binary verified to contain the Rust
`__main`/`smss_csrss_rust::csrss::*`. **Smoke is OWED, not faked**:
worktree cold `ninja livecd` is blocked at the `.code16`
`boot/freeldr/bootsect` asm (system-gcc/ASM16 cold-configure gap) —
the documented `feedback_worktree_build_reality` /
`freeldr_clean_build_regression` cold-build wall, independent of
csrss; the real smoke path is the main checkout's warm `build-i386`. This audit defines the keep-set, the spine,
the substrate prerequisite, the translation order, and the rustconv
configuration for the bulk translation phase.

### Phase 0.1 — what landed (2026-05-16)

- **`sdk/lib/rust/ntdll-sys/`** built (ntkrnl-sys/win32k-sys clone):
  wrapper = the resolved 8-header set, bindgen ×3 arches, **cargo-clean
  on i686/x86_64-pc-windows-gnu + aarch64-pc-windows-gnullvm**. i386
  1,257 fns / 991 structs / 11,367 consts; amd64/arm64 comparable.
  Workspace member added.
- **Wrapper gotchas banked** (for the inevitable regen): comment text
  must not contain `*/` (`Rtl*/Csr*`, `Nt*/Zw*` closed the block
  comment → clang parsed prose as code); `__gnuc_va_list` needs a
  `typedef __builtin_va_list __gnuc_va_list;` shim (ROS
  `sdk/include/crt/stdarg.h` shadows clang's under the lifted
  `-nostdinc`); include order must mirror the components' PCHs
  (`WIN32_NO_STATUS` + win base headers, then `NTOS_MODE_USER` +
  `ndk/ntndk.h`, then sm/csr/win subsys headers).
- **postprocess_bindgen.py**: added a scoped, layout-neutral pass that
  renames `new_bitfield_1` ctor params colliding with a `pub const`
  (i386-only: `KernelMode`/`UserMode` are bare macro-consts there, a
  typed enum on amd64/arm64 → `error[E0005]` refutable pattern). 2
  params renamed on i386, 0 elsewhere — generic, survives regen.
- **rustconv wired**: `TARGETS["smss-csrss"]` (bindgen_files =
  ntdll-sys, scanned as the "ddk" surface — NOT ntkrnl-sys),
  `detect_target` routes `base/system/smss`, `subsystems/csr`,
  `subsystems/win/basesrv`. **winsrv deliberately NOT routed** (stays
  win32k until the Phase-5 ownership gate).
- **Proving-slice resolve** (prove-before-scale): catalog = 17,068
  "ddk" symbols; the SM/CSR/BASE protocol struct surface is captured
  (52 `CSR_API_MSG`/`CSR_PROCESS`/`CSR_THREAD` hits — the substrate's
  whole reason for existing). Raw resolve rate is **17–57%** (csrss.c
  57%, smss.c 33%, csrsrv/init.c 23%, sminit.c 17%) — *but the
  unresolved set is the same non-gap noise win32k had pre-tuning*:
  macros (`NTAPI`, `UNREFERENCED_PARAMETER`), locals (`Status`,
  `Response`, `BasePriority`), FORCEINLINE current-object accessors
  (`NtCurrentPeb/Process/Thread`), and intra-crate
  csrsrv-internal/static fns (`CsrLoadServerDll`, defined *in*
  csrsrv — the CSR analog of win32k's `Int*`/`co_*`). win32k went
  233→41 (~82%) only *after* de-prefix + gap_functions + local/macro
  filtering; the same tuning slice is the next step here. Genuine
  gaps so far look small (a few `STATUS_*` not in the ntstatus
  allowlist scope, the FORCEINLINE `NtCurrent*`).

### Phase 0.2 — landed (2026-05-16, commit 2a5dd92a)

Resolver tuned (globally correct, not target-scoped): NT/Win32
annotation macros → `BUILTIN_NAMES`; in-file function names + params
skipped (never FFI imports for any target — catches the intra-crate
csrsrv-internal pattern); FORCEINLINE `NtCurrent*`/`Zw*Current*`/
`RtlProcessHeap` → `GAP_FUNCTIONS`. `generate_bindings.sh` gained a
`VIBEOS_BUILD_ROOT` override (a fresh worktree has no gitignored
`build-*/` dirs and there's no cold-cmake path — regen points at the
main checkout's warm dirs).

**Re-resolve: standalone spike csrss.c 57% → 79% (win32k parity).**
csrss.c is the *clean substrate measurement* — no sibling-file deps,
so its rate is the pure external-Native-API-surface test. Multi-file
components stay numerically low (csrsrv/init.c 28%, sminit.c 20%) but
the residual is exclusively two **non-substrate** classes: (1)
csrsrv-internal `Csr*`/`Sb*` defined in *sibling* .c files — the
win32k `Int*`/`co_*` intra-component analog, resolved at translation
time *within* the output crate, not external FFI; (2) Hungarian
body-locals the per-file resolver can't scope. **Zero genuine
external-substrate gaps.** `STATUS_*` confirmed at parity with
ntkrnl-sys (0) / win32k-sys (4): `(NTSTATUS)0x…` cast-macros are not
bindgen-evaluable *by design* — handled by hand-written consts + the
LLM prompt, not a defect.

**Phase 0 substrate is PROVEN.** Phase 1 is unblocked.

### Phase 0.3 — landed (2026-05-16, commit 2680d357)

Generic resolve-truthfulness fixes (all globally correct — they also
sharpen ntoskrnl/hal/win32k resolution, not target-scoped):

1. **catalog.py** `_scan_bindgen` now indexes `pub static mut` extern
   data. It only matched struct/type/const/fn — every bindgen *data*
   export (`CsrObjectDirectory`, kernel `KdDebuggerNotPresent`, …) was
   a phantom "unresolved" for **all** targets. Real generic bug.
2. **scanner.py** collects local-variable / parameter declarator names
   via tree-sitter (`_collect_local_decls` → `ScanResult.local_names`);
   the resolver skips them (after catalog lookup, so a genuine import
   is never suppressed; a true local shadows in C anyway). Stops the
   Hungarian-body-local false-positive flood.
3. **resolver.py** universal NT macros (`FIELD_OFFSET`,
   `RTL_NUMBER_OF`, `RTL_CONSTANT_STRING`, `UNICODE_NULL`, `ALIGN_*`)
   → `BUILTIN_NAMES`.
4. **ntdll-sys wrapper** pulls the component master internal headers
   `srv.h` + `basesrv.h` + `smss.h` (the win32k-sys-pulls-win32kp.h
   move; their `-I` dirs added in `generate_bindings.sh`; `FN_ALLOW`
   broadened for lowercase-second-segment internal prefixes
   `Smp*`/`Basep*`/`Csrp*`).

Resolve (i386): csrss.c 57→**95%**, csrsrv/init.c 23→**71%**, smss.c
33→**72%**, smloop.c **71%**, capture.c **85%**, thredsup **68%**;
aggregate **~57%** with core files at win32k parity (71–95%).

**Polish stopped here (prove-before-scale).** Residual is three
benign, well-understood classes, none of which gate translation: (a)
per-component globals/fns needing yet more header coverage (e.g.
`sdk/lib/smlib`'s own header not added — low ROI); (b) cast-macro
consts `STATUS_*`/`SECURITY_*` — *parity with ntkrnl-sys/win32k-sys,
not bindgen-evaluable by design*; (c) struct-field-access identifiers
(generic scanner). The resolve-% does **not** gate translation
quality — the LLM receives full C source and the checker iterates
imports. Substrate is PROVEN; Phase 1 is unblocked.

Still owed (not Phase-1 blockers, tracked): optional smlib own-header
for its resolve.

### verify_sizes.py — landed (commit 2a8d6e27)

`ntdll-sys/verify_sizes.py` (adapted from ntkrnl-sys: lifts ninja
flags like generate_bindings.sh, `#include`s wrapper.h, VIBEOS_BUILD_ROOT
override). 825 (i386) / 839 (amd64) / 832 (arm64) compile-time
`size_of::<nt::T>() == N` assertions wired into lib.rs; cargo build
clean ×3 WITH them active ⇒ every bindgen struct layout exactly
matches the C layout.

### csrss.exe spike + P1a pipeline fixes — landed (03166b57, 8eb28d57)

The 102-LOC csrss.c spike (the win32k-gdiobj.c "prove on a leaf,
flush the cascade" pattern) surfaced 3 **systematic** Phase-1 pipeline
gaps. All fixed once-for-all-files in P1a and validated by
re-translating csrss.exe clean:

1. **Wrong FFI crate** → `Target.sys_crate`/`sys_umbrella` +
   `postprocess.collapse_sys_crate` (deterministically rewrites the
   kernel-oriented `ntkrnl_sys::{ddk,base,…}::` the kernel prompt
   produces to a single `use ntdll_sys::nt::*;`; LLM-independent,
   threaded through all 4 postprocess call sites) +
   `templates/system_prompt_user.md` (appended via
   `Target.system_prompt_addendum`).
2. **FORCEINLINE `NtCurrent*` mis-translation** → `ntdll-sys/src/
   inlines.rs` (the user-mode `ke-inlines` analog: `NtCurrentTeb`
   `fs:[0x18]`/`gs:[0x30]`/`x18` per arch, `NtCurrentPeb`,
   `Nt/ZwCurrent{Process,Thread}` pseudo-handles, over the real
   bindgen `_TEB`/`_PEB`) + `postprocess.strip_user_inline_
   redeclarations` (deterministic net dropping LLM extern re-decls +
   bogus hardcoded `struct PEB/TEB`). Also
   `postprocess_bindgen.add_enum_deprefix_aliases` (~1,500/arch
   bare-name aliases for type-prefixed constified enums —
   `ProcessBasePriority`, `OptionOk`, …; **must run before** the
   bitfield-collision fix or a new bare alias like `Transition` itself
   collides with a `new_bitfield_1` ctor param).
3. **i386 `_main` decoration** → **non-issue, verified**: GNU i686
   prepends the cdecl underscore, so Rust `#[no_mangle] extern "C" fn
   _main` emits `__main` — byte-identical to the C `csrss.c.obj` and
   the `nt` CRT `entry_point.c` expectation.

`smss-csrss-rust` (workspace member) builds clean ×3. Remaining for
csrss.exe: the cohabitation cmake-flip (comment `csrss.c`, link the
rlib, build `csrss.exe`, smoke-gate) — needs the main checkout's
build tree (the worktree has none).

Companion to:
- [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md) — kernel translation, ~99.7% done
- [hal-translation-audit.md](hal-translation-audit.md) — HAL translation
- [freeldr-translation-audit.md](freeldr-translation-audit.md) — FreeLDR + boot DLLs
- [win32k-translation-audit.md](win32k-translation-audit.md) — kernel-mode win32k.sys; **claims `winsrv.dll` as its own "Phase N+1" — see the winsrv conflict in §Scope below**
- [win32k-pretranslation-audit.md](win32k-pretranslation-audit.md) — the SEH/lock pretranslation machinery (mostly *not* needed here; see §C-side prep)
- [pseh-elimination.md](pseh-elimination.md) — overall PSEH strategy

## TL;DR

- **This is the user-mode boot spine, not a single module.** ~46K LOC
  C across **5 PE binaries + 2 static libs**, all on the critical path
  *before* `shell ready`.
- **First user-mode translation target in the project.** Every prior
  audit (ntoskrnl/hal/freeldr/win32k) was kernel-mode. No
  `kernel-rust` staticlib umbrella, no `ntkrnl-sys` — these link
  `ntdll`/`nt`/`libcntpr`. The headline prerequisite is a new
  **`ntdll-sys`** Native-API bindgen crate (the `win32k-sys` analog
  for user mode). It does not exist yet; nothing ships until it does.
- **Smoke is already a perfect gate — no harness work needed first.**
  win32k's hard problem was post-`shell ready` regressions being
  invisible. Here *every file runs before `shell ready`*: any
  regression = no boot = existing smoke catches it instantly. This
  stack is **substantially lower-risk than win32k** despite the
  5-binary spread.
- **SEH is a non-issue.** 19 `_SEH2_TRY` sites total across 46K LOC
  (vs win32k's 282). The entire Pass-0/1/1b/2/2b/3 pretranslation
  machine is unnecessary; C-side prep folds into half a page.
- **The dependency hub is the SM/CSR LPC protocol, not a handle
  table.** `smlib`↔`csrsrv` and `csrlib`↔`csrsrv` are matched
  client/server pairs over LPC ports with a capture-buffer marshalling
  layer (`csrlib/capture.c`). The protocol structs must be a shared
  crate translated *first*, or client/server drift silently across the
  port.
- **csrss.exe is 102 LOC** — translate it as the project's first
  fully-Rust user-mode PE: a clean methodology spike.
- **winsrv is 60% of the LOC (~27K) and the only win32k-coupled
  piece.** It is in scope here as Phase 5, **but pause before starting
  it**: the win32k audit claims it as "Phase N+1." Resolve ownership
  (this doc vs that one) at the Phase 5 gate, don't translate it twice.
- **Cost via rustconv: budget ~$8–15 lifetime** (~60 files × ~$0.05 ×
  3–5 iters). Far cheaper than win32k; the risk is calendar, not money
  — every regression here is a non-boot, fast to catch but boot-blocking.

## The spine

```
ntoskrnl  ──exec──▶  smss.exe          (first user-mode process)
                       │  ├─ pagefile setup, \KnownDlls, env, registry
                       │  ├─ ExecuteImage → autochk
                       │  └─ ExecuteImage → csrss.exe ──▶ csrsrv.dll
                       │                                     ├─ load basesrv.dll  (ServerDll 1)
                       │                                     └─ load winsrv.dll   (consrv + usersrv)
                       └─ ExecuteImage → winlogon.exe ──▶ … ──▶ explorer.exe ──▶ "shell ready"
```

Every box left of `winlogon` is in this audit. Every one of them must
work for smoke to reach `shell ready` — which it does today on i386
7/7, amd64 7/7, arm64 5/5. That makes existing smoke a complete
regression gate for this entire stack with **zero harness changes**.

## Today's binaries + source layout

| Component | Path | LOC | Files | Artifact | Module type | Exports | SEH |
|---|---|--:|--:|---|---|--:|--:|
| **SMSS** | `base/system/smss/` | 6,865 | 10 | `smss.exe` | nativecui | — | 1 |
| **smlib** | `sdk/lib/smlib/` | 917 | 2 | static (`smlib`) | — | `smdll.spec` 13 | 0 |
| **csrss** | `subsystems/csr/csrss/` | 102 | 1 | `csrss.exe` | nativecui | — | 0 |
| **csrsrv** | `subsystems/csr/csrsrv/` | ~5,400 | 7 | `csrsrv.dll` | nativedll | `csrsrv.spec` 35 | 8 |
| **csrlib** | `subsystems/csr/csrlib/` | ~970 | 3 | static (`csrlib`) | — | `csrdll.spec` | 0 |
| **basesrv** | `subsystems/win/basesrv/` | 4,349 | ~8 | `basesrv.dll` | win32dll | `basesrv.spec` 4 | 2 |
| **winsrv** | `win32ss/user/winsrv/` | 27,179 | 38 | `winsrv.dll` | win32dll | `winsrv.spec` 3 | 8 |
| **Total** | | **~45,800** | | 5 PEs + 2 libs | | | **19** |

### SMSS — `base/system/smss/`

```
sminit.c     2,581   ← session-mgr init: pagefile, KnownDlls, registry, ObDir   (the monster)
pagefile.c   1,150   ← pagefile sizing/creation
smsubsys.c     679   ← subsystem (CSRSS) registration + boot
smss.c         583   ← entrypoint, autochk/csrss/winlogon exec (SmpExecuteImage)
smutil.c       558   ← misc helpers
smloop.c       521   ← SM API LPC server loop
smss.h         332
smsbapi.c      221   ← SM SB (subsystem) API handlers
smsessn.c      217   ← session tracking
crashdmp.c      23   ← crash-dump trigger stub
```
Native EXE: links `nt` (native CRT subset), `${PSEH_LIB}`, `smlib`;
imports only `ntdll`. No win32, no kernel-mode anything. Self-contained
once the SM-protocol crate (smlib) exists.

### CSR core — `subsystems/csr/`

```
csrsrv/  api.c 1,548  procsup.c 1,420  init.c 1,190  thredsup.c 1,086
         server.c 716  session.c 578  wait.c 436  srv.h/api.h/status.h
         → csrsrv.dll: the CSR server runtime (process/thread tracking,
           LPC API dispatch loop, sessions, server-DLL loader). 35 exports.
csrlib/  api.c 100  capture.c 353  connect.c 521  csrlib.h
         → static lib: client-side CSR API + capture-buffer marshalling.
           capture.c is the copy-in/out across the LPC boundary — the
           user-mode analog of the kernel "probe" surface.
csrss/   csrss.c 102  → csrss.exe: trivial bootstrap, links csrsrv+ntdll.
```

### basesrv — `subsystems/win/basesrv/`

```
vdm.c     1,459   ← VDM (16-bit/DOS) support API
dosdev.c  1,110   ← DefineDosDevice / drive-letter mapping
init.c      737   ← ServerDllInitialize entrypoint
nls.c       389   ← NLS / codepage API
proc.c      327   ← process-create notifications
appcompat.c  67   sndsntry.c 67
```
The Base API server. csrsrv loads it as ServerDll #1. Leaf — no
inbound deps from the rest of the stack.

### winsrv — `win32ss/user/winsrv/` (the big one, 60%)

```
consrv/            ~19,011   ← Console server
  consrv/ core      8,407
  consrv/condrv     3,276    ← console driver shim
  consrv/frontends/gui  5,343
  consrv/frontends/tui  1,262
  consrv/frontends/serial 723
usersrv/            2,508    ← USER server (NtUser* glue) — win32k-coupled
concfg/             1,740    ← console config sub-library (add_subdirectory)
init.c                183    ← ServerDllInitialize for both consrv+usersrv
```
38 .c files, 0 C++, 8 SEH sites. Links `consrv`, `usersrv`,
`win32ksys`, `csrsrv`, `gdi32`, `user32`, `libcntpr`. **usersrv is
coupled to (still-C) win32k via `NtUser*`; consrv is not.**

## Scope — and the winsrv conflict

This doc owns the full SMSS→CSRSS spine **including winsrv**, per the
decision to keep it here as Phase 5. **But:** the win32k translation
audit explicitly lists `winsrv` (user32/gdi32/imm32/winsrv) as *its*
"Phase N+1, separate audit." That is a real ownership ambiguity, not a
both-fine situation — translating it twice wastes API spend (cost
matters; see CLAUDE.md) and the two audits would fork.

**Resolution rule:** winsrv stays Phase 5 *on paper* here, but Phase 5
opens with a mandatory **pause-and-check gate**:

> Before any winsrv translation work, verify whether the win32k track
> has already translated winsrv (or started). Check `sdk/lib/rust` for
> a `winsrv-rust`/`consrv-rust` crate and `win32ss/user/winsrv/CMakeLists.txt`
> for `# replaced by Rust` lines. If win32k owns it, this Phase 5 is a
> no-op cross-reference; if not, this doc proceeds and the win32k audit's
> "Phase N+1" line should be struck. Decide ownership explicitly at the
> gate — do not start translating winsrv until that is settled.

`smss.exe`, `csrss.exe`, `csrsrv.dll`, `csrlib`, `smlib`, `basesrv.dll`
are unambiguously owned here.

## Why this is lower-risk than win32k

| Axis | win32k | this stack |
|---|---|---|
| Mode | kernel-mode | user-mode |
| Smoke coverage | up to `shell ready` only; post-init regressions invisible → needed Pass-7 input harness *first* | **every file runs before `shell ready`**; existing smoke is a complete gate, **no harness prerequisite** |
| SEH sites | 282 (Pass 0/1/1b/2/2b/3 machine) | 19 (half a page of prep) |
| Failure mode | breaks the desktop subtly (screenshot needed) | breaks the boot loudly (no `shell ready`) — trivially caught |
| LOC | ~145K kernel-mode | ~46K user-mode |
| Cost | ~$25–50 | ~$8–15 |

The one thing that is *harder*: there is no FFI substrate. win32k had
`win32k-sys` built before its translation audit was even finalized.
Here, Phase 0 has to build it from scratch.

## Substrate gap — `ntdll-sys` (the real blocker)

These processes link `ntdll` and call the **Native API** (`Nt*`,
`Zw*`, `Rtl*`, `Ldr*`, `Csr*`, `Dbg*`) plus the subsystem headers in
`sdk/include/reactos/subsys/` (SM, CSR, BASE message structs). There is
**no Rust crate for any of this today** (checked `sdk/lib/rust` —
nothing). `ntkrnl-sys` is the *kernel-internal* surface and is the
wrong crate (kernel-mode types, no `Rtl*`/`Csr*` user-mode surface).

**Decision: one `ntdll-sys` bindgen crate**, the user-mode mirror of
`win32k-sys`/`ntkrnl-sys`:

```
sdk/lib/rust/ntdll-sys/
├── Cargo.toml
├── generate_bindings.sh        ← bindgen ×3 arches (i686/x86_64/aarch64-pc-windows-gnu)
├── verify_sizes.py
└── src/
    ├── lib.rs                  ← re-exports
    ├── base.rs                 ← hand-written: NTSTATUS, UNICODE_STRING, OBJECT_ATTRIBUTES…
    ├── ntdll.rs                ← hand-written wrappers bindgen can't do (FORCEINLINE Rtl*)
    ├── bindgen_{arm64,amd64,i386}.rs
    └── size_check_{…}.rs
```

**Header root — RESOLVED 2026-05-16 (was OQ#2).** `<ndk/ntndk.h>` is
the NT-internal umbrella only (it does carry both kernel *and*
user-mode NDK via `umfuncs.h`/`umtypes.h`, good) but it does **not**
pull the SM/CSR/BASE message structs. Those live in separate subsys
headers every component here `#include`s directly. The bindgen wrapper
root is the fixed 8-header set:
`ndk/ntndk.h` + `sm/ns.h` + `sm/smmsg.h` + `csr/csr.h` +
`csr/csrmsg.h` + `csr/csrsrv.h` + `win/base.h` + `win/basemsg.h`.
One crate suffices — the protocol structs just need explicit
`#include`s in the wrapper, not a second crate. Reuse the proven
bindgen recipe from
`feedback_win32k_sys_bindgen` (`-fms-extensions` yes /
`-fms-compatibility` no; `--allowlist-var '.+'`; tagged-enum
de-prefix). Wire into rustconv `config.py` as a new `ntdll` Target +
`catalog.py` scan + `detect_target` (mirror the win32k Target).

This is the single Phase-0 prerequisite. Budget it as its own slice
(~2–3 sessions, same shape as the win32k-sys build) before any `.c`
is submitted to rustconv.

## SM/CSR LPC protocol — the dependency hub

`smlib` is the SM client library; `smloop.c` is the SM server. `csrlib`
is the CSR client library; `csrsrv` is the CSR server. Both are
matched pairs talking over LPC ports with **capture buffers**
(`csrlib/capture.c`, `csrlib/connect.c`): the client allocates a
capture buffer, marshals args into it, the server validates and
copies. This is the user-mode structural analog of the kernel probe
surface — and the place silent ABI drift will bite if one side is
translated without the other.

**Therefore the message/capture structs are translated first, as a
shared crate**, before either client or server consumes them — exactly
the role win32k's GDI/USER handle table played as the Phase-1 hub. They
mostly come *from* `ntdll-sys` (the subsys headers bindgen into it), so
in practice this is: finish `ntdll-sys` with the SM/CSR/BASE message
structs included, then translate `capture.c`/`connect.c` (the
marshalling logic) early in Phase 1 and treat that .rs as the
contract every server validates against.

## C-side prep (folded in — not a separate audit)

19 `_SEH2_TRY` sites total:

| Component | SEH sites | Files | Shape |
|---|--:|--:|---|
| SMSS | 1 | 1 | belt-and-suspenders around an LPC reply |
| csr (csrsrv) | 8 | 2 | capture-buffer access guards (`api.c`, `procsup.c`) |
| basesrv | 2 | 1 | argument validation in a server handler |
| winsrv | 8 | 4 | console-buffer / message validation |

All are user-mode try/except around an already-structured
capture-buffer access — there is **no `_SEH2_FINALLY`**, no
lock-cleanup SEH, no raw `ProbeForRead`. Under `USE_DUMMY_PSEH` the
handlers are dead code (same as everywhere else). Prep is: confirm
each site's `TRY` body is a pure capture-buffer read/validate (it is,
by construction of the CSR API), annotate the ~5 enclosing functions
with a one-line `// SEH: capture-buffer guard, swallow → STATUS_*`
note for the rustconv prompt, and move on. No Pass-0 classifier, no
mechanical rewriter, no sidecar JSON. If a site turns out structurally
ambiguous during translation, hand that one function to the LLM with
the win32k Pass-1b canonical-shape contract (`feedback_llm_c_to_c_seh_lowering`)
— but do not stand up the whole pretranslation machine for 19 sites.

## Build-artifact decision — per-binary rlib cohabitation

Same proven cohabitation pattern as `ntoskrnl-rust`/`hal-rust`, applied
per binary. One Rust workspace, **one rlib crate per PE/lib**, linked
into the existing cmake build with the corresponding `.c` commented
out (`# replaced by Rust (path)`), `--allow-multiple-definition`
covering the swap-in window, lld first-match wins. **No custom `.json`
target specs, no cdylib** — the existing user-mode Rust targets
(`i686-pc-windows-gnu`, `x86_64-pc-windows-gnu`,
`aarch64-pc-windows-gnu`) are correct here (these link `ntdll`, not the
kernel; standard PE user-mode).

```
sdk/lib/rust/
├── ntdll-sys/        ← Phase 0 substrate (above)
├── smlib-rust/       ← static lib → libsmlib parity
├── csrlib-rust/      ← static lib (capture/connect — the protocol contract)
├── csrsrv-rust/      ← rlib → csrsrv.dll (35-export spec contract)
├── smss-rust/        ← rlib → smss.exe
├── basesrv-rust/     ← rlib → basesrv.dll
└── winsrv-rust/      ← rlib → winsrv.dll  (Phase 5, gated — see Scope)
```

Each rlib needs `extern crate <name>;` in whatever umbrella links it,
or its `#[no_mangle]` symbols won't reach the static lib
(`feedback_extern_crate_for_staticlib`). Cohabitation per file is
independently revertible (the never-revert / broken-in-tree rules from
CLAUDE.md apply unchanged).

**csrss.exe (102 LOC) is the spike:** translate it whole as the
project's *first fully-Rust user-mode PE binary*. It exercises the
native entrypoint (no CRT — `nativecui`), the `csrsrv` import, and the
`ntdll-sys` FFI end-to-end in one tiny, low-risk file. Do it
immediately after Phase 0 to prove the pattern before the bulk.

## Translation order

Leaf→root, every step smoke-gated (every file is pre-`shell ready`, so
one smoke run per flip is a complete check):

**Phase 0 — substrate.** Build `ntdll-sys` (×3 arches, cargo-clean,
size-checked, rustconv-wired). The only hard prerequisite. **~2–3 sessions.**

**Phase 1 — SM/CSR protocol crates.** `smlib` (917 LOC — small,
shared, high-leverage SM client/server protocol) + `csrlib`
capture/connect (the LPC marshalling contract). These define the ABI
every server validates against. **~2 sessions.**

**Phase 2 — csrss.exe spike + csrsrv.dll.** csrss.exe (102 LOC,
fully-Rust PE milestone) then csrsrv.dll core (api/init/procsup/
thredsup/server/session/wait, ~5.4K). 35-export spec is the contract;
run `verify_abi.py`/`verify_constants.py` per file. **~3–4 sessions.**

**Phase 3 — SMSS.** smss.exe (~6.9K; `sminit.c` 2,581 LOC is the
largest single file — use `rustconv translate` not `batch`,
`feedback_rustconv_translate_vs_batch`). Boot-critical but
self-contained once smlib is Rust. **~3–4 sessions.**

**Phase 4 — basesrv.dll.** ~4.3K, leaf server (VDM/dosdev/NLS), no
inbound deps. **~2–3 sessions.**

**Phase 5 — winsrv.dll. ⚠ PAUSE-AND-CHECK FIRST (see §Scope).** If
owned here: consrv before usersrv (consrv ~19K is self-contained;
usersrv ~2.5K is win32k-coupled and should wait on or coordinate with
the win32k track). concfg is a leaf sub-lib, do it first within the
phase. **~6–9 sessions** *if* this track owns it.

**Total: ~18–25 sessions** (Phase 5 dominates and may not be ours).
Calendar risk, not cost risk — every regression is a non-boot, caught
in one smoke run.

## Rustconv configuration

The pipeline exists. Per-target additions:

1. **New `ntdll`/user-mode Target** in `tools/rustconv/rustconv/config.py`
   (mirror the `win32k` Target dataclass): `ntdll-sys` catalog,
   `system_prompt`-level note that these are *user-mode native* (link
   `ntdll`, no kernel types, `extern "system"` = stdcall on i386 still,
   capture-buffer not probe). A short `system_prompt_user.md` analog
   may be warranted; reuse most of the kernel prompt.
2. **ABI:** user-mode native still has the i386 stdcall decoration
   hazard (#1 i386 crash cause, `feedback_extern_c_abi`). Enforce
   against the 4 `.spec` files (smdll/csrsrv/basesrv/winsrv) +
   `ntdll-sys` bindgen the same deterministic way win32k does
   (`abi_map.py` + a `postprocess.enforce_*` pass keyed by
   `(name, cfg-guard)` per `feedback_postprocess_cfg_dual_def_dedup`).
3. **Verification gates per file:** `verify_abi.py`, `verify_constants.py`,
   `scan_ud2.py`, `lint_kernel.py` (the captures(none) defenses apply
   to user-mode Rust too). All 3 arches build before next file.
4. **Cost:** ~60 files × ~$0.05 × 3–5 iters ≈ **$8–15 lifetime**.

## Per-arch notes

- **csrss.exe ARM64 2 MB stack hack** (`subsystems/csr/csrss/CMakeLists.txt`):
  `-Wl,--stack,2097152` because llvm-dlltool converts delay-imports to
  regular imports → deep recursive `LdrpLoadDll` overflows the default
  1 MB/4 KB-commit stack; `SizeOfStackCommit` is patched to `0x40000`
  by `qemu/mkramdisk.sh`. **Must be preserved through translation** —
  it's a link-options + ramdisk-patch concern, orthogonal to the Rust
  source, so it survives cohabitation automatically, but the audit
  flags it so it isn't "cleaned up" as dead config.
- Native module types (`nativecui`/`nativedll`) mean no C runtime and a
  custom entrypoint (`NtProcessStartup` / `DllMain`). Rust side is
  `#![no_std]` + `#![no_main]` with a `#[no_mangle]` native entry —
  different from both kernel rlibs and ordinary user-mode Rust. Prove
  it with the csrss.exe spike before the bulk.
- Few arch ifdefs in this tree; stay arch-agnostic with
  `#[cfg(target_arch)]` gates (same pattern as ntoskrnl).

## ABI surface

Four export contracts must be preserved byte-for-byte:

| Spec | Entries | Consumed by |
|---|--:|---|
| `sdk/lib/smlib/smdll.spec` | 13 | smss + csrsrv (SM API) |
| `subsystems/csr/csrsrv/csrsrv.spec` | 35 | csrss.exe, basesrv, winsrv |
| `subsystems/win/basesrv/basesrv.spec` | 4 | csrsrv (server-DLL entry) |
| `win32ss/user/winsrv/winsrv.spec` | 3 | csrsrv (server-DLL entry) |

Plus `subsystems/csr/csrlib/csrdll.spec` (static-lib client API).
After each file flips, `nm` the resulting PE and confirm every spec
symbol is present at the expected i386/amd64 decoration.

## Definition of done

- `cargo build` produces working `smss.exe`, `csrss.exe`,
  `csrsrv.dll`, `basesrv.dll` (+ `winsrv.dll` if owned here) for all 3
  arches.
- All 3 smoke tests pass on the Rust binaries (`shell ready` reached —
  which by construction exercises this entire stack).
- `scan_ud2.py --check`, `verify_abi.py`, `verify_constants.py`,
  `lint_kernel.py` clean against the Rust output.
- The C `base/system/smss/`, `subsystems/csr/`, `subsystems/win/basesrv/`
  (and winsrv if owned) trees can be deleted (or kept as parallel CI).
- The winsrv ownership conflict with the win32k audit is explicitly
  resolved (one audit owns it, the other cross-references).

## Open questions

1. **winsrv ownership** — this doc Phase 5 vs win32k audit "Phase N+1."
   Decide at the Phase 5 gate; don't translate twice. (Primary risk.)
2. ~~**`ntdll-sys` header root**~~ **RESOLVED 2026-05-16.**
   `<ndk/ntndk.h>` does *not* pull SM/CSR/BASE message structs; the
   bindgen wrapper is the fixed 8-header set documented in §Substrate.
   One crate, no second crate needed.
3. **Fully-Rust native PE entrypoint** — csrss.exe spike must prove
   `#![no_main]` + native entry + no-CRT linking works before the
   bulk. If it doesn't, fall back to the ntoskrnl-cargo-style
   "cmake-built objects, cargo-driven link" PoC shape.
4. **usersrv ↔ win32k coupling** — usersrv calls `NtUser*` into
   still-C win32k. Translate usersrv only after consrv, and coordinate
   the syscall-surface assumptions with the win32k track.
5. **smlib vs ntdll-sys overlap** — the SM API may already be partly
   declared in the subsys headers bindgen'd into `ntdll-sys`. Decide
   whether `smlib-rust` is a thin wrapper over `ntdll-sys` types or
   owns its own message structs. Lean: own the marshalling logic,
   borrow the structs from `ntdll-sys`.

## Cross-references

- Substrate recipe: `feedback_win32k_sys_bindgen` (bindgen flags),
  `win32k-pretranslation-audit.md` §Substrate.
- Cohabitation pattern: [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md),
  CLAUDE.md §Cohabitation pattern (HAL).
- SEH residual handling: `feedback_llm_c_to_c_seh_lowering`.
- winsrv ownership counterpart: [win32k-translation-audit.md](win32k-translation-audit.md)
  §"What this audit doesn't measure → User-mode DLLs".
