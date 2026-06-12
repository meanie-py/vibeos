# RTL (sdk/lib/rtl) C→Rust Translation Audit

**Surface:** `sdk/lib/rtl/*.c` — the NT `Rtl*` run-time library that compiles
into the single `librtl.a`, which links into **both** `ntoskrnl.exe` (kernel,
`-D_NTOSKRNL_`) **and** `ntdll.dll` (user mode). One archive, two consumers.

**Created:** 2026-06-01 (consolidates the prior `project_rtl_strings_track`
memory + the `handoff-rtl-libs-next-steps.md` codex handoff into an in-repo
ledger). Sister docs: `ntoskrnl-translation-audit.md`,
`ntdll-translation-audit.md`, `static-libs-translation-audit.md`.

**Active branch:** `rtl-priv-port` (worktree `vibeos-wt-rtl-priv`).
HEAD `b77c87e0840` + uncommitted PATH-wave work (see §4).

---

## TL;DR

- The byte-diffable + translate-only string/security/process/env/path surface
  of `sdk/lib/rtl` is **flipped to Rust** (crate `sdk/lib/rust/rtl-strings/`,
  cohabitation-commented in `sdk/lib/rtl/CMakeLists.txt`). ~20 `.c` files.
- The pure-algorithm surface (bitmap, avltable, splaytree, crc32, …) was
  already carved into sibling rlibs under `rtl-rust` — tracked separately in
  `static-libs-translation-audit.md`, **not here**.
- **path.c (3040 LOC, the largest single file) is flipped as of this
  session** (27/27 fns). Flipping it advanced amd64 boot far enough to expose
  — and this session **fix** — three latent downstream bugs in the
  (already-committed) process-creation / env path. amd64 now reaches
  `[WIN32K] shell ready`; the current blocker is a **services.exe init hang**
  (post-`shell ready`), still under investigation.
- `registry.c`, `heap.c` (family), `image.c`, and a few misc files remain C.

---

## Three rtl-ish surfaces — don't confuse them

The single biggest source of "wait, which rtl?" confusion. There are three
distinct axes, and **this doc is only axis (3)**:

1. **`ntoskrnl-rust/src/rtl/`** — tiny kernel-private glue (libsupp / misc /
   prefetch). Part of the ntoskrnl port → `ntoskrnl-translation-audit.md`.
2. **`rtl-rust` umbrella + ~18 algorithm rlibs** — the pure-algorithm core of
   sdk/lib/rtl carved out (bitmap, avltable, splaytree, generictable, crc32,
   compress, encode, largeint, time, random, luid, handle, interlck, prefix,
   access, mem, byteswap, slist, utf8, uniprefix). →
   `static-libs-translation-audit.md`.
3. **The rest of `sdk/lib/rtl/*.c`** — the `Rtl*` API bodies (strings,
   security, env, process, thread, path, …) translated into the
   **`rtl-strings`** crate (`core_rs` pure + `cabi` `#[no_mangle]` shims).
   **← THIS DOC.**

`cnt-rust`/`libcntpr` (the C `_LIBCNT_` stdlib) is a *fourth*, orthogonal axis
— rtl-strings reuses its `core_rs::mbcp` CP1252 model as the one-truth NLS
core, but they are separate tracks.

---

## Cohabitation mechanism

- `librtl` has **NO `--allow-multiple-definition`** (unlike HAL). A `.c` is
  whole-file-commentable in `sdk/lib/rtl/CMakeLists.txt` **only when Rust
  covers 100% of that file's exports.** Partial flips don't link.
- `rtl-strings` is **one crate** spanning many `.c` files, so a swap is
  all-or-nothing across whatever set of files share unresolved symbols.
- Kernel vs user: `librtl` is built `-D_NTOSKRNL_`. rtl-strings mirrors this
  with a `kernel` cargo feature (e.g. `nls_tables.rs`, SxS branches gated
  `#[cfg(not(feature="kernel"))]`). `rtl-rust` enables `features=["cabi",
  "kernel"]` for the kernel `librtl`; the diff harness uses `["cabi"]` only.
- PEB/TEB/heap access from kernel-linked rtl uses **libsupp accessor shims**
  (`{ntoskrnl,dll/ntdll}/rtl/libsupp.{c,rs}` + the `RtlpGetPebEnvironment`,
  `RtlpPrcPebProcessParametersAddr`, … family) so the Rust never mirrors the
  per-arch PEB layout. **These shims are a recurring bug source — see §4.**

---

## 4. Current status (2026-06-01)

### Flipped to Rust (rtl-strings cabi) — cohabitation-commented in CMakeLists

| File | Wave | Net | Notes |
|---|---|---|---|
| unicode.c / nls.c / dos8dot3.c | RTL-1…6b | byte-diff | strings + NLS (shared CP1252 model). `5593d886` |
| sid.c | SID-1+2 | byte-diff | pointer-free, arch-neutral |
| acl.c | ACL-1+2 | byte-diff | ACL/ACE pointer-free |
| sd.c | SD-1+2+swap | byte-diff | has pointers (abs form); per-arch SD size |
| priv.c | PRIV | translate-only | pure Zw* plumbing |
| timezone.c | TZ | translate-only | Rtl{Query,Set}TimeZoneInformation |
| env.c | ENV | translate-only | env block; **RtlExpandEnvironmentStrings_U** lives here |
| security.c | SEC | byte-diff | 9 RtlHeap edges |
| resource.c | RES | translate-only | SRW-style locks |
| thread.c | THR | translate-only | **RtlCreateUserThread** lives here |
| process.c | PRC | translate-only | **RtlCreateUserProcess / RtlpInitEnvironment** |
| atom.c | ATM | layout-mirror | `d54a073a4bf` (was the deferred "rabbit hole", now done) |
| **path.c** | **PATH** | **translate + smoke** | **THIS SESSION — 27/27 fns. Uncommitted.** |

(Algorithm crates — bitmap/bitmap64, avltable, splaytree, unicodeprefix,
crc32, compress, encode, largeint, time, random, luid, handle, interlck,
prefix, access, mem, byteswap, slist, utf8 — also commented out here but are
tracked in `static-libs-translation-audit.md`.)

### Still C in sdk/lib/rtl

> **SUPERSEDED 2026-06-09 — rtl is ~fully Rust now: registry.c (fc4563c30ba), image.c (rtl-diag), heap.c+heapdbg (rtl-heap), heappage/heapuser translated; only heappage/heapuser/arch-asm remain active C in CMakeLists.**
>
> **SUPERSEDED AGAIN 2026-06-11 — the per-arch exception/unwind tail is now Rust
> too** (`sdk/lib/rust/rtl-except/`): `amd64/{unwind,except,stubs,dynfntbl}.c`,
> `i386/{except,thread}.c`, `arm64/except.c` (7 files, ~3.1K LOC — including the
> x64 RtlVirtualUnwind/RtlUnwindEx unwinder state machine). Smoke-verified ×3.
> The only active C left in librtl is `dbgprint_variadic.c` (deliberate
> C-variadic FFI glue) + the `.s/.S` asm (trampolines/capture-context) +
> `rtl_vista_stub.c` (placeholder). **NOTE the stale-fat-archive trap found
> during this flip:** the cmake `ar` rule appends without truncating, so
> `librtl.a` had been accumulating ZOMBIE objects of every previously-flipped
> `.c` (heappage.c.obj, thread.c.obj, …) which silently kept satisfying symbols
> (`RtlpPageHeapEnabled`, `NtCurrentTeb`, `RtlFillMemoryUlong`, `_swprintf`).
> If a flip "links clean" suspiciously easily, `rm librtl.a && ninja` and check
> `ar t` for members that should be gone. Fixes landed with this wave:
> rtl-heappage `#[no_mangle]` on RtlpPageHeapEnabled/RtlpDphGlobalFlags +
> inline NtCurrentTeb, mem crate RtlFillMemoryUlong enabled on amd64,
> ntdll-cargo compiles `_swprintf.c`.

| File | LOC | Why still C |
|---|---|---|
| **registry.c** | 1351 | RtlQueryRegistryValues etc. Translate-only candidate. **Calls the Rust RtlExpandEnvironmentStrings_U — see §4 bug #3.** |
| heap.c (+ heapdbg/heappage/heapuser) | large | stateful allocator (locks/VM/free-lists) — **DEFERRED to the behavioral/Kani track**, never byte-diff. |
| image.c | — | shared PE format primitives (`RtlImageNtHeader`, …); still in librtl. |
| misc remainder | — | bootdata, generic small files. |

### Boot state (amd64, with the PATH flip + this session's 3 fixes)

> **Point-in-time snapshot (2026-06-01); the services.exe-hang described below is a state of that session, not current HEAD.**

`./vibeos-cli smoke --arch amd64` reaches **`[WIN32K] shell ready`** — the
full kernel → smss → csrss → win32k chain boots with path.c live. Winlogon
then starts, switches desktop, and launches `services.exe`, which **hangs
during init** (PID 128 loading apphelp/setupapi/newdev; no msgqueue spam;
clean hang) → smoke verdict `HANG_TIMEOUT`. **This is the current amd64
blocker.** i386 and arm64 **not yet re-validated** with the 3 fixes.

---

## The PATH wave (this session) — flip exposed 3 latent bugs

Flipping path.c was *correct*: instrumented boot confirmed
`RtlGetFullPathName_Ustr` / `RtlDosPathNameToNtPathName_U` convert paths
perfectly (`X:\reactos\winsxs\manifests` → `\??\X:\reactos\winsxs\manifests`).
The flip simply let amd64 boot reach the (committed) user-process-creation and
env code paths — which had **never been exercised on amd64** before — and
those carried three latent bugs. All three are fixed this session; all three
are all-arch:

1. **`RtlpPrcPebProcessParametersAddr` kernel stub returned NULL**
   (`ntoskrnl-rust/src/rtl/libsupp.rs`). It only computes
   `&((PPEB)peb_base)->ProcessParameters` (pointer arithmetic on the *child*
   PEB user-VA — no deref, valid in kernel). The stub returned NULL →
   `RtlpInitEnvironment`'s 3rd `ZwWriteVirtualMemory` wrote 8 bytes to address
   0 → uncatchable under DUMMY_PSEH → **BUGCHECK 0x1E**. Diagnosed via CR2=0 +
   page-table walk (`monitor xp/1gx`; VA 0 P=0). Fix = real pointer arithmetic
   on the bindgen `_PEB`.

2. **`RtlCreateUserThread` passed `&initial_teb` instead of
   `initial_teb.StackBase`** to `RtlInitializeContext` (`rtl-strings/cabi`).
   The C `RtlInitializeContext` 5th param is mistyped `PINITIAL_TEB` but used
   as `(ULONG_PTR)StackBase`; the original C caller (`sdk/lib/rtl/thread.c`)
   passes `InitialTeb.StackBase`. The Rust passed the struct address → new
   thread's user `Rsp` = a kernel-stack address → `KiInitializeUserApc` probe
   raised AV → **BUGCHECK 0x1E**. Confirmed via trap frame (Rip = smss entry
   ✓, Rsp = kernel stack ✗). Fix = pass `initial_teb.StackBase`.

3. **`RtlExpandEnvironmentStrings_U` ignored the "NULL = current process
   environment" contract** (`rtl-strings/cabi`, env wave). C passes
   `Environment` to `RtlQueryEnvironmentVariable_U`, which resolves NULL to
   `NtCurrentPeb()->ProcessParameters->Environment`. The Rust reimplemented
   the lookup via the pure `expand_env_strings(env_slice)` and mapped NULL →
   empty `&[]`. `RtlQueryRegistryValues` (still-C) expands `REG_EXPAND_SZ` by
   calling it with NULL → empty env → smss's KnownDll `DllDirectory` value
   `%SystemRoot%\system32` was left **unexpanded** → `NtOpenFile` returned
   `c000003a OBJECT_PATH_NOT_FOUND` → `SmpInit` failed → smss (critical
   process) died → **BUGCHECK 0x4C**. Fix = NULL → `RtlpGetPebEnvironment()`.
   (smss's env *did* contain `SystemRoot=X:\reactos`; expansion was simply
   skipped.)

> **Historical correction:** the 2026-05-28 handoff attributed a
> `c000003a`-family boot failure to an "upstream PnP / uniata IDE-binding
> bug." At least the smss-KnownDll variant was, in fact, bug #3 above (a cabi
> translation defect), **not** upstream. The old 0x6B ArcName/CdRom symptom no
> longer reproduces on current HEAD; boot now progresses to services.exe.

---

## Methodology

- **Byte-diff waves** (strings, sid, acl, sd, sec, atom): the `rtl-difftest`
  crate compiles the *genuine* `.c` (own ref TU in `build.rs`, with a SAL
  neutralization block + `scrape_protos()` for call-before-definition) and
  fuzzes Rust output byte-for-byte against it. `core_rs` is `forbid(unsafe)`,
  Kani-friendly. ~86 cases today.
- **Translate-only waves** (priv, tz, env, res, thr, prc, path): pure syscall
  plumbing (chains of `Zw*`) — wrong shape for byte-diff; smoke is the only
  net. Single block in `cabi/mod.rs`, comment in `CMakeLists.txt`, ship.
- **Heap is its own track** (behavioral / property / Kani) — never byte-diff.

---

## Open issues / debt

- **amd64 services.exe init hang** (current blocker, this session). Boot
  reaches `shell ready` + winlogon; `services.exe` hangs loading
  apphelp/setupapi/newdev. Next: attach to PID 128, find the blocking
  syscall/path/registry call.
- **i386 + arm64 not re-validated** with the 3 PATH-wave fixes. All three fixes
  are all-arch (FIX#1 kernel libsupp, FIX#2/#3 cabi). Note i386
  `RtlInitializeContext` has a different ABI/arg contract — re-verify FIX#2
  there before claiming green.
- **amd64 SD-wave msgqueue.split2.c:87 spam** (pre-PATH) — diff-coverage gap,
  needs `--timeout 600`; extend `mk_relative_sd`/`mk_absolute_sd`.
- **i386 idle-cpu regression** on continue-rust-port HEAD — pre-existing,
  bisect-owed, **unrelated** to any rtl wave (verified via stash revert).
- `registry.c`, `image.c`, `heap.c` family remain C (see §4 table).

---

## Conventions (must-follow)

- **Worktree path discipline** — all edits use the
  `vibeos-wt-rtl-priv/...` absolute path; repo-root paths leak into main.
- **Never revert a `.rs` to C** to pass a build/smoke; **never delete a
  translated `.rs`** without explicit approval; a broken-but-integrated
  translation visible in smoke is the correct state (see AGENTS.md).
- **Never weaken smoke markers.** Boot bar is `shell ready` (x86).
- **Stage commits explicitly by path; never `git add -A`.** Always-excluded
  pre-existing mods: `AGENTS.md`, `boot/bootdata/livecd.ini`,
  `drivers/input/i8042prt/*.c`, `ntoskrnl-rust/.../pnpmgr/devaction.rs`,
  `arm64input-rust/`, `win32k-(kernel-)?rust`, `win32k-sys`, the
  `sdk/lib/rust/Cargo.toml` workspace edit.
- **`core_rs` is `forbid(unsafe_code)`** — inline asm + raw FFI live in
  `cabi/mod.rs` only.

---

## Cross-references

- Tracker memory: `project_rtl_strings_track`, `project_rtl_path_wave_wip`.
- Methodology: `cnt_rust_prep` (diff-harness), `smoke-tests`.
- Sister audits: `static-libs-translation-audit.md` (rtl algo crates),
  `ntdll-translation-audit.md` (ntdll DLL + libsupp boundary),
  `ntoskrnl-translation-audit.md`.
- Superseded handoff: `handoff-rtl-libs-next-steps.md` (codex session handoff;
  its "upstream PnP blocker" framing is corrected above).
